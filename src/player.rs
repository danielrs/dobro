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
    ao: ao::Ao,

    // Sender and receiver for external users.
    sender: Option<Sender<PlayerAction>>,
    receiver: Option<Receiver<PlayerAction>>,

    // Handles.
    player_handle: Option<JoinHandle<()>>,

    // State for pausing.
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
            pause_pair: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    pub fn receiver(&self) -> &Option<Receiver<PlayerAction>> {
        &self.receiver
    }

    /// Starts playing the given track in a separate thread; stopping
    /// any previously started threads.
    pub fn play(&mut self, track: Track) {
        if let Some(audio) = track.track_audio {
            if let Ok(mut earwax) = Earwax::new(&audio.high_quality.audio_url) {
                // Stops any previously running thread.
                self.stop();

                let (sender, external_receiver) = channel();
                let (external_sender, receiver) = channel();

                self.sender = Some(external_sender);
                self.receiver = Some(external_receiver);

                let pause_pair = self.pause_pair.clone();
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
                            paused = cvar.wait(paused).unwrap();
                        }

                        // Messages.
                         if let Ok(action) = receiver.try_recv() {
                             match action {
                                 PlayerAction::Stop => break,
                                 _ => (),
                             }
                             sender.send(action);
                         }
                         // Plays chunk.
                         device.play(chunk.data);
                    }

                    sender.send(PlayerAction::Stop);
                }));
            }
        }
    }

    /// Stops the audio thread.
    pub fn stop(&mut self) {
        if let Some(ref sender) = self.sender {
            sender.send(PlayerAction::Stop);
        }

        if let Some(player_handle) = self.player_handle.take() {
            player_handle.join();
        }

        self.sender = None;
        self.receiver = None;
        self.player_handle = None;
    }

    /// Pauses the audio thread.
    pub fn pause(&mut self) {
        let &(ref lock, ref cvar) = &*self.pause_pair;
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
            let &(ref lock, ref cvar) = &*self.pause_pair;
            let mut paused = lock.lock().unwrap();
            is_paused = *paused;
        }
        if is_paused {
            self.unpause();
        }
        else {
            self.pause();
        }
    }
}

#[derive(Debug)]
pub enum PlayerAction {
    Play,
    Pause,
    Stop,
}
