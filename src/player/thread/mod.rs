use super::error::Error;
use super::PlayerAction;
use super::state::{PlayerState, PlayerStatus};

use ao;
use earwax::Earwax;
use pandora::{Pandora, Station, Track};

use std::collections::VecDeque;
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

        // Initializes driver.
        let driver = match ao::Driver::new() {
            Ok(driver) => driver,
            Err(e) => {
                sender.send(Err(e.into())).unwrap();
                sender.send(Ok(PlayerStatus::Shutdown)).unwrap();
                event_handle.join().unwrap();
                return;
            }
        };

        // Context of our player.
        let mut ctx = ThreadContext {
            pandora: pandora,
            state: state,
            pause_pair: pause_pair,
            sender: sender,
            driver: driver,
            action: None,
        };

        // Finite state machine loop.
        let mut fsm = ThreadState::new();
        ctx.send_status(PlayerStatus::Standby);
        while !fsm.is_shutdown() {
            // Action.
            ctx.action = if fsm.is_standby() {
                event_receiver.recv().ok()
            }
            else {
                event_receiver.try_recv().ok()
            };

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
    pub driver: ao::Driver,

    pub action: Option<PlayerAction>,
}

impl ThreadContext {
    pub fn send_status(&mut self, status: PlayerStatus) {
        self.state.lock().unwrap().set_status(status.clone());
        self.sender.send(Ok(status)).unwrap();
    }

    pub fn send_error(&self, error: Error) {
        self.sender.send(Err(error)).unwrap();
    }

    pub fn action(&mut self) -> Option<PlayerAction> {
        self.action.take()
    }
}

/// Finite state machine for the thread.
enum ThreadState {
    Shutdown,

    Standby,

    Station {
        station: Station
    },

    Track {
        station: Station,
        tracklist: VecDeque<Track>,
    },

    Playing {
        station: Station,
        tracklist: VecDeque<Track>,
        track: Track,
        earwax: Earwax,
        device: ao::Device,
    },
}

impl ThreadState {
    /// Creates a new state machine in Standby state.
    pub fn new() -> ThreadState {
        ThreadState::Standby
    }

    /// Returns true if the current state is Standby state.
    pub fn is_standby(&self) -> bool {
        match *self {
            ThreadState::Standby => true,
            _ => false,
        }
    }

    /// Returns true if the current state is Shutdown state.
    pub fn is_shutdown(&self) -> bool {
        match *self {
            ThreadState::Shutdown => true,
            _ => false,
        }
    }

    // ----------------
    // Updating.
    // ----------------

    /// Consumes the current state and returns a new state.
    pub fn update(self, ctx: &mut ThreadContext) -> ThreadState {
        match self {
            ThreadState::Standby =>
                Self::update_standby(ctx),

            ThreadState::Station { station } =>
                Self::update_station(ctx, station),

            ThreadState::Track { station, tracklist } =>
                Self::update_track(ctx, station, tracklist),

            ThreadState::Playing { station, tracklist, track, earwax, device} =>
                Self::update_playing(ctx, station, tracklist, track, earwax, device),

            _ =>
                self,
        }
    }

    fn update_standby(ctx: &mut ThreadContext) -> ThreadState {
        if let Some(PlayerAction::Play(station)) = ctx.action() {
            ctx.send_status(PlayerStatus::Started(station.clone()));
            return Self::new_station(station);
        }

        Self::new()
    }

    fn update_station(ctx: &mut ThreadContext, station: Station) -> ThreadState {
        ctx.send_status(PlayerStatus::Fetching(station.clone()));
        match ctx.pandora.stations().playlist(&station).list() {
            Ok(tracklist) => {
                Self::new_track(station, tracklist.into_iter().collect())
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
        mut tracklist: VecDeque<Track>
    ) -> ThreadState {
        if let Some(track) = tracklist.pop_front() {
            if let Some(ref audio) = track.clone().track_audio {
                match Earwax::new(&audio.high_quality.audio_url) {
                    Ok(earwax) => {
                        let format = ao::Format::new();
                        let device = ao::Device::new(&ctx.driver, &format, None).unwrap();

                        ctx.send_status(PlayerStatus::Playing(track.clone()));
                        return Self::new_playing(station, tracklist, track, earwax, device);
                    },
                    Err(e) => {
                        ctx.send_error(e.into());
                    }
                }
            }
        }
        else {
            return Self::new_station(station);
        }

        Self::new_track(station, tracklist)
    }

    fn update_playing(
        ctx: &mut ThreadContext,
        station: Station,
        tracklist: VecDeque<Track>,
        track: Track,
        mut earwax: Earwax,
        device: ao::Device,
    ) -> ThreadState {
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

        // Actions
        if let Some(action) = ctx.action() {
            match action {
                PlayerAction::Play(new_station) => {
                    ctx.send_status(PlayerStatus::Finished(track.clone()));
                    ctx.send_status(PlayerStatus::Stopped(station.clone()));
                    ctx.send_status(PlayerStatus::Started(new_station.clone()));
                    return Self::new_station(new_station);
                },
                PlayerAction::Stop => {
                    ctx.send_status(PlayerStatus::Finished(track.clone()));
                    ctx.send_status(PlayerStatus::Stopped(station.clone()));
                    ctx.send_status(PlayerStatus::Standby);
                    return Self::new();
                },

                PlayerAction::Skip => {
                    ctx.send_status(PlayerStatus::Finished(track.clone()));
                    return Self::new_track(station, tracklist);
                },

                PlayerAction::Exit => {
                    ctx.send_status(PlayerStatus::Finished(track.clone()));
                    ctx.send_status(PlayerStatus::Stopped(station.clone()));
                    ctx.send_status(PlayerStatus::Shutdown);
                    return Self::new_shutdown();
                },

                _ => (),
            }
        }

        let duration = earwax.info().duration.seconds();
        if let Some(chunk) = earwax.spit() {
            ctx.state.lock().unwrap().set_progress(chunk.time.seconds(), duration);
            device.play(chunk.data);
        }
        else {
            ctx.send_status(PlayerStatus::Finished(track.clone()));
            return Self::new_track(station, tracklist);
        }

        return Self::new_playing(station, tracklist, track, earwax, device);
    }

    // ----------------
    // Creation of different states.
    // ----------------

    fn new_shutdown() -> ThreadState {
        ThreadState::Shutdown
    }

    fn new_station(station: Station) -> ThreadState {
        ThreadState::Station {
            station: station,
        }
    }

    fn new_track(station: Station, tracklist: VecDeque<Track>) -> ThreadState {
        ThreadState::Track {
            station: station,
            tracklist: tracklist,
        }
    }

    fn new_playing(station: Station, tracklist: VecDeque<Track>, track: Track, earwax: Earwax, device: ao::Device)
    -> ThreadState {
        ThreadState::Playing {
            station: station,
            tracklist: tracklist,
            track: track,
            earwax: earwax,
            device: device,
        }
    }
}
