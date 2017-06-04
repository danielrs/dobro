use super::audio::Audio;
use super::error::Error;
use super::track_loader::TrackLoader;
use super::PlayerAction;
use super::state::{PlayerState, PlayerStatus};

use ao;
use pandora::{Pandora, Station, Track};

use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{channel, Sender, Receiver};

/// This function starts the event and player thread.
pub fn spawn_player(
    pandora: &Arc<Pandora>,
    main_state: &Arc<Mutex<PlayerState>>,
    main_sender: Sender<Result<PlayerStatus, Error>>,
    main_receiver: Receiver<PlayerAction>) -> JoinHandle<()> {

    // The Condvar used for pausing the player thread.
    let main_pause_pair: Arc<(Mutex<bool>, Condvar)> = Arc::new((Mutex::new(false), Condvar::new()));

    // Sender and receiver for communication between the event thread and
    // the player thread.
    let (event_sender, event_receiver) = channel();

    // Thread is dedicated to receive the events from the main thread and
    // forward player events to the player thread.
    let state = main_state.clone();
    let pause_pair = main_pause_pair.clone();
    let receiver = main_receiver;
    let sender = main_sender.clone();

    let event_handle = {
        thread::Builder::new().name("event".to_string()).spawn(move || {
            while let Ok(action) = receiver.recv() {
                match action {
                    PlayerAction::Pause => {
                        let &(ref lock, _) = &*pause_pair;
                        let mut paused = lock.lock().unwrap();
                        *paused = true;
                    },
                    PlayerAction::Unpause => {
                        let &(ref lock, ref cvar) = &*pause_pair;
                        let mut paused = lock.lock().unwrap();
                        *paused = false;
                        cvar.notify_one();
                    },

                    PlayerAction::Report => {
                        sender.send(Ok(state.lock().unwrap().status().clone())).unwrap();
                    },

                    PlayerAction::Exit => {
                        event_sender.send(PlayerAction::Exit).unwrap();
                        break;
                    },

                    action => {
                        event_sender.send(action).unwrap();
                    }
                }
            }
        }).unwrap()
    };

    // The 'player' thread runs while the Player is in scope. It plays the given station
    // and takes care of fetching the tracks. All the events this thread receives are
    // the events forwarded from the 'event' thread.
    let pandora = pandora.clone();
    let state = main_state.clone();
    let pause_pair = main_pause_pair.clone();
    let sender = main_sender.clone();

    thread::Builder::new().name("player".to_string()).spawn(move || {
        // Context of our player.
        let mut ctx = ThreadContext {
            pandora: pandora,
            state: state,
            pause_pair: pause_pair,
            sender: sender,
            receiver: event_receiver,
        };

        // Finite state machine loop.
        let mut fsm = ThreadFSM::new();
        ctx.send_status(PlayerStatus::Standby);
        while !fsm.is_shutdown() {
            fsm = fsm.update(&mut ctx);
        }

        event_handle.join().unwrap();
    }).unwrap()
}

// ----------------
// Finite State Machine
// ----------------

/// Context struct for our finite state machine.
struct ThreadContext {
    pub pandora: Arc<Pandora>,
    pub state: Arc<Mutex<PlayerState>>,
    pub pause_pair: Arc<(Mutex<bool>, Condvar)>,
    pub sender: Sender<Result<PlayerStatus, Error>>,
    pub receiver: Receiver<PlayerAction>,
}

impl ThreadContext {
    /// Sends a status through the sender channel.
    pub fn send_status(&mut self, status: PlayerStatus) {
        self.state.lock().unwrap().set_status(status.clone());
        self.sender.send(Ok(status)).unwrap();
    }

    /// Sends an error through the sender channel.
    pub fn send_error(&self, error: Error) {
        self.sender.send(Err(error)).unwrap();
    }

    /// Blocks the current thread and returns the next available
    /// action.
    pub fn action(&mut self) -> Option<PlayerAction> {
        self.receiver.recv().ok()
    }

    /// Checks for a pending action without blocking the current
    /// thread.
    pub fn try_action(&mut self) -> Option<PlayerAction> {
        self.receiver.try_recv().ok()
    }
}

/// Finite state machine for the thread.
enum ThreadFSM {
    Shutdown,

    Standby,

    Station {
        station: Station
    },

    Track {
        station: Station,
        track_loader: TrackLoader,
    },

    Playing {
        station: Station,
        track_loader: TrackLoader,
        track: Track,
        audio: Audio,
    },
}

impl ThreadFSM {
    /// Creates a new state machine in Standby state.
    pub fn new() -> ThreadFSM {
        ThreadFSM::Standby
    }

    /// Returns true if the current state is Standby state.
    pub fn is_standby(&self) -> bool {
        match *self {
            ThreadFSM::Standby => true,
            _ => false,
        }
    }

    /// Returns true if the current state is Shutdown state.
    pub fn is_shutdown(&self) -> bool {
        match *self {
            ThreadFSM::Shutdown => true,
            _ => false,
        }
    }

    // ----------------
    // Updating.
    // ----------------

    /// Consumes the current state and returns a new state.
    pub fn update(self, ctx: &mut ThreadContext) -> ThreadFSM {
        match self {
            ThreadFSM::Standby =>
                Self::update_standby(ctx),

            ThreadFSM::Station { station } =>
                Self::update_station(ctx, station),

            ThreadFSM::Track { station, track_loader } =>
                Self::update_track(ctx, station, track_loader),

            ThreadFSM::Playing { station, track_loader, track, audio } =>
                Self::update_playing(ctx, station, track_loader, track, audio),

            _ =>
                self,
        }
    }

    fn update_standby(ctx: &mut ThreadContext) -> ThreadFSM {
        if let Some(PlayerAction::Play(station)) = ctx.action() {
            ctx.state.lock().unwrap().set_station(station.clone());
            ctx.send_status(PlayerStatus::Started(station.clone()));
            return Self::new_station(station);
        }

        // Stay in Standby state.
        Self::new()
    }

    fn update_station(ctx: &mut ThreadContext, station: Station) -> ThreadFSM {
        ctx.send_status(PlayerStatus::Fetching(station.clone()));
        match ctx.pandora.stations().playlist(&station).list() {
            Ok(tracklist) => {
                Self::new_track(station, TrackLoader::new(tracklist.into_iter().collect()))
            },
            Err(e) => {
                ctx.send_error(e.into());
                Self::new_station(station)
            },
        }
    }

    fn update_track(
        ctx: &mut ThreadContext,
        station: Station,
        mut track_loader: TrackLoader,
    ) -> ThreadFSM {
        if let Some((track, audio)) = track_loader.next() {
            ctx.state.lock().unwrap().set_track(track.clone());
            ctx.send_status(PlayerStatus::Playing(track.clone()));
            return Self::new_playing(station, track_loader, track, audio);
        }
        Self::new_station(station)
    }

    fn update_playing(
        ctx: &mut ThreadContext,
        station: Station,
        track_loader: TrackLoader,
        track: Track,
        mut audio: Audio,
    ) -> ThreadFSM {
        // Pauses.
        {
            let &(ref lock, ref cvar) = &*ctx.pause_pair.clone();
            let mut paused = lock.lock().unwrap();
            while *paused {
                ctx.send_status(PlayerStatus::Paused(track.clone()));
                paused = cvar.wait(paused).unwrap();
                ctx.send_status(PlayerStatus::Playing(track.clone()));
            }
        }

        // Actions.
        if let Some(action) = ctx.try_action() {
            match action {
                PlayerAction::Play(new_station) => {
                    ctx.state.lock().unwrap().clear_info();
                    ctx.state.lock().unwrap().set_station(new_station.clone());
                    ctx.send_status(PlayerStatus::Finished(track.clone()));
                    ctx.send_status(PlayerStatus::Stopped(station.clone()));
                    ctx.send_status(PlayerStatus::Started(new_station.clone()));
                    return Self::new_station(new_station);
                },
                PlayerAction::Stop => {
                    ctx.state.lock().unwrap().clear_info();
                    ctx.send_status(PlayerStatus::Finished(track.clone()));
                    ctx.send_status(PlayerStatus::Stopped(station.clone()));
                    ctx.send_status(PlayerStatus::Standby);
                    return Self::new();
                },

                PlayerAction::Skip => {
                    ctx.state.lock().unwrap().clear_track();
                    ctx.state.lock().unwrap().clear_progress();
                    ctx.send_status(PlayerStatus::Finished(track.clone()));
                    return Self::new_track(station, track_loader);
                },

                PlayerAction::Exit => {
                    ctx.state.lock().unwrap().clear_info();
                    ctx.send_status(PlayerStatus::Finished(track.clone()));
                    ctx.send_status(PlayerStatus::Stopped(station.clone()));
                    ctx.send_status(PlayerStatus::Shutdown);
                    return Self::new_shutdown();
                },

                _ => (),
            }
        }

        // Playback.
        if let Ok((current, duration)) = audio.play() {
            ctx.state.lock().unwrap().set_progress(current.seconds(), duration.seconds());
        }
        else {
            ctx.state.lock().unwrap().clear_track();
            ctx.state.lock().unwrap().clear_progress();
            ctx.send_status(PlayerStatus::Finished(track.clone()));
            return Self::new_track(station, track_loader);
        }

        return Self::new_playing(station, track_loader, track, audio);
    }

    // ----------------
    // Creation of different states.
    // ----------------

    fn new_shutdown() -> ThreadFSM {
        ThreadFSM::Shutdown
    }

    fn new_station(station: Station) -> ThreadFSM {
        ThreadFSM::Station {
            station: station,
        }
    }

    fn new_track(station: Station, track_loader: TrackLoader) -> ThreadFSM {
        ThreadFSM::Track {
            station: station,
            track_loader: track_loader,
        }
    }

    fn new_playing(
        station: Station,
        track_loader: TrackLoader,
        track: Track,
        audio: Audio,
    ) -> ThreadFSM {
        ThreadFSM::Playing {
            station: station,
            track_loader: track_loader,
            track: track,
            audio: audio,
        }
    }
}
