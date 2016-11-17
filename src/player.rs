use ao;
use earwax::Earwax;

use pandora::{Pandora, StationItem};

use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{channel, Sender, Receiver};

/// Player for playing audio in a separate thread, with a channel
/// for communication.
pub struct Player {
    #[allow(dead_code)]
    ao: ao::Ao,
    player_handle: Option<JoinHandle<()>>,

    // Pandora handler.
    pandora: Arc<Mutex<Pandora>>,

    // Current station.
    station: Arc<Mutex<Option<StationItem>>>,

    // Thread.
    player_status: Arc<Mutex<PlayerStatus>>,
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
    pub fn new(pandora: Arc<Mutex<Pandora>>) -> Self {
        Player {
            ao: ao::Ao::new(),
            player_handle: None,

            pandora: pandora,
            station: Arc::new(Mutex::new(None)),
            player_status: Arc::new(Mutex::new(PlayerStatus::Stopped)),
            pause_pair: Arc::new((Mutex::new(false), Condvar::new())),

            sender: None,
            receiver: None,
        }
    }

    /// Returns the current station.
    pub fn station(&self) -> Option<StationItem> {
        self.station.lock().unwrap().clone()
    }

    /// Returns the current status of the player.
    pub fn status(&self) -> PlayerStatus {
        *self.player_status.lock().unwrap()
    }

    /// Returns true if the player is stopped.
    pub fn is_stopped(&self) -> bool {
        *self.player_status.lock().unwrap() == PlayerStatus::Stopped
    }

    /// Returns a reference to the receiver.
    ///
    /// # Returns
    /// * Some(Receiver<PlayerAction>) when the thread is running and emitting
    /// messages.
    /// * None when the thread is not running.
    pub fn receiver(&self) -> &Option<Receiver<PlayerStatus>> {
        &self.receiver
    }

    /// Starts playing the given station in a separate thread; stopping
    /// any previously started threads.
    pub fn play(&mut self, station: StationItem) {
        // Stops any previously running thread.
        self.stop();
        *self.station.lock().unwrap() = Some(station.clone());

        let pandora  = self.pandora.clone();
        // let station = self.station.clone();
        let player_status = self.player_status.clone();
        let pause_pair = self.pause_pair.clone();

        let (external_sender, receiver) = channel();
        let (sender, external_receiver) = channel();

        self.sender = Some(external_sender);
        self.receiver = Some(external_receiver);

        self.player_handle = Some(thread::spawn(move || {
            let set_status = |status: PlayerStatus| {
                *player_status.lock().unwrap() = status;
                sender.send(status);
            };

            let driver = ao::Driver::new().unwrap();
            let pandora = pandora.lock().unwrap();
            let stations = pandora.stations();
            loop {
                let playlist = stations.playlist(&station);
                let tracklist = playlist.list().unwrap();

                for track in tracklist {
                    if let Some(audio) = track.track_audio {
                        if let Ok(mut earwax) = Earwax::new(&audio.high_quality.audio_url) {
                            // TODO: Format should replicate earwax format.
                            let format = ao::Format::new();
                            let device = ao::Device::new(&driver, &format, None).unwrap();

                            set_status(PlayerStatus::Playing);
                            while let Some(chunk) = earwax.spit() {
                                // Pauses.
                                let &(ref lock, ref cvar) = &*pause_pair;
                                let mut paused = lock.lock().unwrap();
                                while *paused {
                                    set_status(PlayerStatus::Paused);
                                    paused = cvar.wait(paused).unwrap();
                                    set_status(PlayerStatus::Playing);
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
        }));
    }

    /// Stops the audio thread.
    #[allow(unused_must_use)]
    pub fn stop(&mut self) {
        // Thread needs to be running to receive the stop signal,
        // so we unpause it.
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
        *self.station.lock().unwrap() = None;
        self.sender = None;
        self.receiver = None;
    }

    /// Skips the current track (if any is playing).
    pub fn skip(&mut self) {
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
}

/// Enumeration type for sending player actions.
pub enum PlayerAction {
    Skip,
    Stop,
}

/// Enumeration type for showing player status.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PlayerStatus {
    Playing,
    Paused,
    Stopped,
}
