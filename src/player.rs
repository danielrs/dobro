use ao;
use earwax::Earwax;
use pandora::Track;

use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{channel, Sender, Receiver};

/// Player for playing audio in a separate thread, with a channel
/// for communication.
pub struct Player {
    // Ao initialization state.
    #[allow(dead_code)]
    ao: ao::Ao,

    // Sender for notifying the thread to stop. And receiver
    // for receiving status changes as soon as it happens.
    sender: Option<Sender<()>>,
    receiver: Option<Receiver<PlayerStatus>>,

    // Thread.
    player_handle: Option<JoinHandle<()>>,
    player_status: Arc<Mutex<PlayerStatus>>,

    // Condition variable for pausing.
    pause_pair: Arc<(Mutex<bool>, Condvar)>,
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Player {
    /// Creates a new Player.
    pub fn new() -> Self {
        Player {
            ao: ao::Ao::new(),
            sender: None,
            receiver: None,
            player_handle: None,
            player_status: Arc::new(Mutex::new(PlayerStatus::Stopped)),
            pause_pair: Arc::new((Mutex::new(false), Condvar::new())),
        }
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

    /// Starts playing the given track in a separate thread; stopping
    /// any previously started threads.
    pub fn play(&mut self, track: Track) {
        if let Some(audio) = track.track_audio {
            if let Ok(mut earwax) = Earwax::new(&audio.high_quality.audio_url) {
                // Stops any previously running thread.
                self.stop();

                let (external_sender, receiver) = channel();
                let (sender, external_receiver) = channel();

                let player_status = self.player_status.clone();
                let pause_pair = self.pause_pair.clone();

                self.sender = Some(external_sender);
                self.receiver = Some(external_receiver);

                *player_status.lock().unwrap() = PlayerStatus::Playing;
                sender.send(PlayerStatus::Playing);
                self.player_handle = Some(thread::spawn(move || {
                    // TODO: Format should replicate earwax format.
                    let driver = ao::Driver::new().unwrap();
                    let format = ao::Format::new();
                    let device = ao::Device::new(&driver, &format, None).unwrap();

                    while let Some(chunk) = earwax.spit() {
                        // Pauses.
                        let &(ref lock, ref cvar) = &*pause_pair;
                        let mut paused = lock.lock().unwrap();
                        while *paused {
                            *player_status.lock().unwrap() = PlayerStatus::Paused;
                            sender.send(PlayerStatus::Paused);
                            paused = cvar.wait(paused).unwrap();
                            *player_status.lock().unwrap() = PlayerStatus::Playing;
                            sender.send(PlayerStatus::Playing);
                        }

                        // Stop signal message.
                         if let Ok(_) = receiver.try_recv() {
                             break;
                         }

                         // Plays chunk.
                         device.play(chunk.data);
                    }
                    *player_status.lock().unwrap() = PlayerStatus::Stopped;
                    sender.send(PlayerStatus::Stopped);
                }));
            }
        }
    }

    /// Stops the audio thread.
    #[allow(unused_must_use)]
    pub fn stop(&mut self) {
        // Thread needs to be running to receive the stop signal,
        // so we unpause it.
        self.unpause();

        // Notifies the thread to stop.
        if let Some(ref sender) = self.sender {
            sender.send(());
        }

        // Waits for the thread to stop.
        if let Some(player_handle) = self.player_handle.take() {
            player_handle.join().unwrap();
        }

        self.sender = None;
        self.receiver = None;
        self.player_handle = None;
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

/// Enumeration type for showing player status.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PlayerStatus {
    Playing,
    Paused,
    Stopped,
}
