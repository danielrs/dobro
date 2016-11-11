use ao;
use earwax::Earwax;
use pandora::Track;

use std::thread;
use std::sync::mpsc::{Sender, Receiver};

pub struct Player {
    ao: ao::Ao,
    driver: ao::Driver,
    sender: Option<Sender<PlayerAction>>,
    receiver: Option<Receiver<PlayerAction>>,
}

impl Player {
    /// Creates a new Player.
    pub fn new() -> Self {
        Player {
            ao: ao::Ao::new(),
            driver: ao::Driver::new().unwrap(),
            sender: None,
            receiver: None,
        }
    }

    /// Creates a new player with the given receiver. Useful
    /// for creating the player in a different thread and
    /// playing music from there.
    pub fn with_channel(sender: Sender<PlayerAction>, receiver: Receiver<PlayerAction>) -> Self {
        Player {
            ao: ao::Ao::new(),
            driver: ao::Driver::new().unwrap(),
            sender: Some(sender),
            receiver: Some(receiver),
        }
    }

    /// Starts playing the given track. Returns when the track is over.
    pub fn play(&self, track: Track) {
        if let Some(audio) = track.track_audio {
            if let Ok(mut earwax) = Earwax::new(&audio.high_quality.audio_url) {
                // TODO: Format should replicate earwax format.
                let format = ao::Format::new();
                let device = ao::Device::new(&self.driver, &format, None).unwrap();

                while let Some(chunk) = earwax.spit() {

                    // Process message
                    if let Some(ref receiver) = self.receiver {
                        if let Ok(action) = receiver.try_recv() {
                            match action {
                                PlayerAction::Stop => break,
                                _ => (),
                            }

                            if let Some(ref sender) = self.sender {
                                sender.send(action);
                            }
                        }
                    }

                    device.play(chunk.data);
                }
            }
        }
        if let Some(ref sender) = self.sender {
            sender.send(PlayerAction::Stop);
        }
    }
}

#[derive(Debug)]
pub enum PlayerAction {
    Play,
    Pause,
    Stop,
}
