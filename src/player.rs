use ao;
use earwax::Earwax;

use pandora::{Pandora, StationItem, Track};

use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, MutexGuard, Condvar};
use std::sync::mpsc::{channel, Sender, Receiver};

/// Player type for playing audio in a separate thread, with a channel
/// for communication.
pub struct Player {
    #[allow(dead_code)]
    ao: ao::Ao,
    pandora: Arc<Pandora>,
    player_handle: Option<JoinHandle<()>>,

    // Player state.
    state: Arc<Mutex<PlayerState>>,
    pause_pair: Arc<(Mutex<bool>, Condvar)>,

    // Sender for notifying the player thread of different actions.
    // Receiver for getting player status.
    sender: Option<Sender<PlayerAction>>,
    receiver: Option<Receiver<PlayerStatus>>,
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Player {
    /// Creates a new Player.
    pub fn new(pandora: &Arc<Pandora>) -> Self {
        Player {
            ao: ao::Ao::new(),
            pandora: pandora.clone(),
            player_handle: None,

            state: Arc::new(Mutex::new(PlayerState::new())),
            pause_pair: Arc::new((Mutex::new(false), Condvar::new())),

            sender: None,
            receiver: None,
        }
    }

    /// Returns the player state. Note that this function is synchronized with the player
    /// thread, meaning that it blocks until "state" is available.
    pub fn state(&self) -> MutexGuard<PlayerState> {
        self.state.lock().unwrap()
    }

    /// Starts playing the given station in a separate thread; stopping
    /// any previously started threads.
    pub fn play(&mut self, station: StationItem) {
        // Stops any previously running thread.
        self.stop();

        let (external_sender, receiver) = channel();
        let (sender, external_receiver) = channel();
        let (event_sender, event_receiver) = channel();

        self.state.lock().unwrap().station = Some(station.clone());

        // Receiver and sender for communicating with main thread.
        self.sender = Some(external_sender);
        self.receiver = Some(external_receiver);

        // Thread is dedicated to receive the events from the main thread and
        // forward player events to the player thread.
        let event_handle = {
            let state = self.state.clone();
            let sender = sender.clone();
            thread::spawn(move || {
                while let Ok(action) = receiver.recv() {
                    match action {
                        PlayerAction::Report => {
                            sender.send(state.lock().unwrap().status.clone()).unwrap();
                        },
                        PlayerAction::Stop => {
                            event_sender.send(PlayerAction::Stop).unwrap();
                            break;
                        },
                        action => {
                            event_sender.send(action).unwrap();
                        }
                    }
                }
            })
        };

        // Channel for checking initialization of thread.
        let (start_sender, start_receiver) = channel();

        // Player thread, it fetches songs from the given stations and receives
        // events from the event thread.
        let pandora = self.pandora.clone();
        let state = self.state.clone();
        let pause_pair = self.pause_pair.clone();
        self.player_handle = Some(thread::spawn(move || {
            let driver = ao::Driver::new().unwrap();

            let set_status = |status: PlayerStatus| {
                state.lock().unwrap().status = status.clone();
                sender.send(status).unwrap();
            };

            // Set start status and notify the main thread so the parent process can
            // return the function.
            set_status(PlayerStatus::Start(station.clone()));
            let _ = start_sender.send(());

            'track_loop: while let Ok(tracklist) = {
                set_status(PlayerStatus::Fetching(station.clone()));
                pandora.stations().playlist(&station).list()
            } {
                for track in tracklist {
                    if track.is_ad() { continue; }

                    state.lock().unwrap().track = Some(track.clone());
                    set_status(PlayerStatus::Playing(track.clone()));
                    if let Some(ref audio) = track.track_audio {
                        if let Ok(mut earwax) = Earwax::new(&audio.high_quality.audio_url) {
                            // TODO: Format should replicate earwax format.
                            let format = ao::Format::new();
                            let device = ao::Device::new(&driver, &format, None).unwrap();
                            let duration = earwax.info().duration.seconds();

                            while let Some(chunk) = earwax.spit() {
                                state.lock().unwrap().progress = Some((chunk.time.seconds(), duration));
                                // Pauses.
                                let &(ref lock, ref cvar) = &*pause_pair;
                                let mut paused = lock.lock().unwrap();
                                while *paused {
                                    set_status(PlayerStatus::Paused(track.clone()));
                                    paused = cvar.wait(paused).unwrap();
                                    set_status(PlayerStatus::Playing(track.clone()));
                                }

                                // Message handler.
                                 if let Ok(action) = event_receiver.try_recv() {
                                     match action {
                                         PlayerAction::Skip => break,
                                         PlayerAction::Stop => {
                                            set_status(PlayerStatus::Stopped(track.clone()));
                                            break 'track_loop;
                                         }
                                         _ => (),
                                     }
                                 }

                                 // Plays chunk.
                                 device.play(chunk.data);
                            }
                            set_status(PlayerStatus::Stopped(track.clone()));
                        }
                    }
                }
            }
            set_status(PlayerStatus::Shutdown);
            event_handle.join().unwrap();
        }));

        // Block until player thread changes status to start.
        let _ = start_receiver.recv();
    }

    /// Requests the player to send an event reporting its current status.
    pub fn report(&self) {
        if let Some(ref sender) = self.sender {
            sender.send(PlayerAction::Report).unwrap();
        }
    }

    /// Skips the current track (if any is playing).
    pub fn skip(&mut self) {
        self.unpause();

        if let Some(ref sender) = self.sender {
            sender.send(PlayerAction::Skip).unwrap();
        }
    }

    /// Stops the audio thread.
    #[allow(unused_must_use)]
    pub fn stop(&mut self) {
        // Thread needs to be running to receive a message
        // so we need to unpause it.
        self.unpause();

        // Notifies the thread to stop.
        if let Some(ref sender) = self.sender {
            sender.send(PlayerAction::Stop);
        }

        // Waits for the thread to stop.
        if let Some(player_handle) = self.player_handle.take() {
            player_handle.join().unwrap();
        }

        self.player_handle = None;
        *self.state.lock().unwrap() = PlayerState::new();
        self.sender = None;
        self.receiver = None;
    }

    /// Pauses the audio thread.
    pub fn pause(&mut self) {
        let &(ref lock, _) = &*self.pause_pair;
        let mut paused = lock.lock().unwrap();
        *paused = true;
    }

    /// Unpauses the audio thread.
    pub fn unpause(&mut self) {
        let &(ref lock, ref cvar) = &*self.pause_pair;
        let mut paused = lock.lock().unwrap();
        *paused = false;
        cvar.notify_one();
    }

    /// Toggles pause / unpause.
    #[allow(unused_assignments)]
    pub fn toggle_pause(&mut self) {
        let mut is_paused = false;
        {
            let &(ref lock, _) = &*self.pause_pair;
            is_paused = *lock.lock().unwrap();
        }

        if is_paused {
            self.unpause();
        }
        else {
            self.pause();
        }
    }

    /// Returns true if the player is shutdown.
    pub fn is_shutdown(&self) -> bool {
        self.state.lock().unwrap().status.is_shutdown()
    }

    /// Returns true if the player is playing audio.
    pub fn is_playing(&self) -> bool {
        self.state.lock().unwrap().status.is_playing()
    }

    /// Returns true if the player is stopped.
    pub fn is_stopped(&self) -> bool {
        self.state.lock().unwrap().status.is_stopped()
    }

    /// Returns true if the player is paused.
    pub fn is_paused(&self) -> bool {
        self.state.lock().unwrap().status.is_paused()
    }

    /// Returns the most recent status from the player.
    ///
    /// # Returns
    /// * Some(PlayerStatus) when the thread is running and emitting
    /// messages.
    /// * None when the thread is not running and there are no recent statuses.
    pub fn next_status(&self) -> Option<PlayerStatus> {
        let mut status = None;
        if let Some(ref receiver) = self.receiver {
            if let Ok(s) = receiver.try_recv() {
                status = Some(s);
            }
        }
        status
    }
}

/// Container for the player state.
pub struct PlayerState {
    pub station: Option<StationItem>,
    pub track: Option<Track>,
    pub progress: Option<(i64, i64)>,
    pub status: PlayerStatus,
}

impl PlayerState {
    /// Returns a new PlayerState.
    pub fn new() -> Self {
        PlayerState {
            station: None,
            track: None,
            progress: None,
            status: PlayerStatus::Shutdown,
        }
    }
}

/// Enumeration type for sending player actions.
pub enum PlayerAction {
    Report,
    Skip,
    Stop,
}

/// Enumeration type for showing player status.
#[derive(Debug, Clone)]
pub enum PlayerStatus {
    Start(StationItem),
    Fetching(StationItem),
    Shutdown,

    Playing(Track),
    Paused(Track),
    Stopped(Track),
}

impl PlayerStatus {
    pub fn is_shutdown(&self) -> bool {
        match *self {
            PlayerStatus::Shutdown => true,
            _ => false,
        }
    }

    pub fn is_playing(&self) -> bool {
        match *self {
            PlayerStatus::Playing(_) => true,
            _ => false,
        }
    }

    pub fn is_paused(&self) -> bool {
        match *self {
            PlayerStatus::Paused(_) => true,
            _ => false,
        }
    }

    pub fn is_stopped(&self) -> bool {
        match *self {
            PlayerStatus::Stopped(_) => true,
            _ => false,
        }
    }
}
