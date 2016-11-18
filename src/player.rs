use ao;
use earwax::Earwax;

use pandora::{Pandora, StationItem, Track};

use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{channel, Sender, Receiver};

/// Player for playing audio in a separate thread, with a channel
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
    pub fn new(pandora: Arc<Pandora>) -> Self {
        Player {
            ao: ao::Ao::new(),
            pandora: pandora,
            player_handle: None,

            state: Arc::new(Mutex::new(PlayerState::new())),
            pause_pair: Arc::new((Mutex::new(false), Condvar::new())),

            sender: None,
            receiver: None,
        }
    }

    /// Returns the player state.
    pub fn state(&self) -> &Arc<Mutex<PlayerState>> {
        &self.state
    }

    /// Starts playing the given station in a separate thread; stopping
    /// any previously started threads.
    pub fn play(&mut self, station: StationItem) {
        // Stops any previously running thread.
        self.stop();

        let pandora  = self.pandora.clone();
        let state = self.state.clone();
        let pause_pair = self.pause_pair.clone();

        let (external_sender, receiver) = channel();
        let (sender, external_receiver) = channel();

        state.lock().unwrap().station = Some(station.clone());
        self.sender = Some(external_sender);
        self.receiver = Some(external_receiver);

        self.player_handle = Some(thread::spawn(move || {
            let driver = ao::Driver::new().unwrap();

            let set_status = |status: PlayerStatus| {
                state.lock().unwrap().status = status;
                sender.send(status);
            };

            set_status(PlayerStatus::Start);
            while let Ok(tracklist) = pandora.stations().playlist(&station).list() {
                for track in tracklist {
                    if track.is_ad() { continue; }
                    if let Some(ref audio) = track.track_audio {
                        if let Ok(mut earwax) = Earwax::new(&audio.high_quality.audio_url) {
                            // TODO: Format should replicate earwax format.
                            let format = ao::Format::new();
                            let device = ao::Device::new(&driver, &format, None).unwrap();
                            let duration = earwax.info().duration.seconds();

                            state.lock().unwrap().track = Some(track.clone());
                            set_status(PlayerStatus::Playing);
                            while let Some(chunk) = earwax.spit() {
                                state.lock().unwrap().progress = Some((chunk.time.seconds(), duration));
                                // Pauses.
                                let &(ref lock, ref cvar) = &*pause_pair;
                                let mut paused = lock.lock().unwrap();
                                while *paused {
                                    set_status(PlayerStatus::Paused);
                                    paused = cvar.wait(paused).unwrap();
                                    set_status(PlayerStatus::Unpaused);
                                }

                                // Stop signal message.
                                 if let Ok(action) = receiver.try_recv() {
                                     match action {
                                         PlayerAction::Skip => break,
                                         PlayerAction::Stop => {
                                            set_status(PlayerStatus::Stopped);
                                            return;
                                         }
                                     }
                                 }

                                 // Plays chunk.
                                 device.play(chunk.data);
                            }
                            set_status(PlayerStatus::Stopped);
                        }
                    }
                }
            }
            set_status(PlayerStatus::Shutdown);
        }));
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

    /// Returns true if the player is stopped.
    pub fn is_stopped(&self) -> bool {
        self.state.lock().unwrap().status == PlayerStatus::Stopped
    }

    /// Skips the current track (if any is playing).
    pub fn skip(&mut self) {
        self.unpause();

        if let Some(ref sender) = self.sender {
            sender.send(PlayerAction::Skip);
        }
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

    /// Returns true if the player is paused.
    pub fn is_paused(&self) -> bool {
        let &(ref lock, _) = &*self.pause_pair;
        *lock.lock().unwrap()
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
            while let Ok(s) = receiver.try_recv() {
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
            status: PlayerStatus::Stopped,
        }
    }
}

/// Enumeration type for sending player actions.
pub enum PlayerAction {
    Skip,
    Stop,
}

/// Enumeration type for showing player status.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PlayerStatus {
    Start,
    Shutdown,
    Playing,
    Paused,
    Unpaused,
    Stopped,
}
