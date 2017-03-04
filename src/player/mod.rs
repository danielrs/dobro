mod error;
mod state;
mod thread;

use self::error::Error;
pub use self::state::{PlayerState, PlayerStatus};
use self::thread::spawn_player;

use ao;
use pandora::{Pandora, Station};

use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::{channel, Sender, Receiver};

/// Player type for playing audio in a separate thread, with a channel
/// for communication.
pub struct Player {
    #[allow(dead_code)]
    ao: ao::Ao,
    player_handle: Option<JoinHandle<()>>,

    // Player state.
    state: Arc<Mutex<PlayerState>>,

    // Sender for notifying the player thread of different actions.
    // Receiver for getting player status.
    sender: Sender<PlayerAction>,
    receiver: Receiver<Result<PlayerStatus, Error>>,
}

impl Drop for Player {
    fn drop(&mut self) {
        // Thread needs to be running to receive a message
        // so we need to unpause it.
        self.unpause();

        // Notifies the thread to exit.
        self.sender.send(PlayerAction::Exit).unwrap();

        // Waits for the thread to stop.
        if let Some(player_handle) = self.player_handle.take() {
            player_handle.join().unwrap();
        }
    }
}

impl Player {
    /// Creates a new Player.
    pub fn new(pandora: &Arc<Pandora>) -> Self {
        // Initialize AO before anything else.
        let ao = ao::Ao::new();

        let state = Arc::new(Mutex::new(PlayerState::new()));

        let (external_sender, receiver) = channel();
        let (sender, external_receiver) = channel();
        let player_handle = spawn_player(pandora, &state, sender, receiver);

        Player {
            ao: ao,
            player_handle: Some(player_handle),

            state: state,

            sender: external_sender,
            receiver: external_receiver,
        }
    }

    /// Returns the player state. Note that this function is synchronized with the player
    /// thread, meaning that it blocks until "state" is available.
    pub fn state(&self) -> MutexGuard<PlayerState> {
        self.state.lock().unwrap()
    }

    //
    // Player control functions
    //

    /// Starts playing the given station.
    pub fn play(&mut self, station: Station) {
        self.unpause();
        self.sender.send(PlayerAction::Play(station)).unwrap();
    }

    /// Stops the current station.
    pub fn stop(&mut self) {
        self.sender.send(PlayerAction::Stop).unwrap();
    }

    /// Pauses the audio thread.
    pub fn pause(&mut self) {
        self.sender.send(PlayerAction::Pause).unwrap();;
    }

    /// Unpauses the audio thread.
    pub fn unpause(&mut self) {
        self.sender.send(PlayerAction::Unpause).unwrap();
    }

    /// Skips the current track (if any is playing).
    pub fn skip(&mut self) {
        self.unpause();
        self.sender.send(PlayerAction::Skip).unwrap();
    }

    /// Toggles pause / unpause.
    pub fn toggle_pause(&mut self) {
        if self.is_paused() {
            self.unpause();
        }
        else {
            self.pause();
        }
    }

    /// Requests the player to send an event reporting its current status.
    pub fn report(&self) {
        self.sender.send(PlayerAction::Report).unwrap();
    }

    //
    // PLayer state functions
    //

    /// Returns true if the player is starting.
    pub fn is_started(&self) -> bool {
        self.state.lock().unwrap().status().is_started()
    }

    /// Returns true if the player is stopped (waiting for a Play action).
    pub fn is_stopped(&self) -> bool {
        self.state.lock().unwrap().status().is_stopped()
    }

    /// Returns true if the player is fetching tracks.
    pub fn is_fetching(&self) -> bool {
        self.state.lock().unwrap().status().is_fetching()
    }

    /// Returns true if the player is playing audio.
    pub fn is_playing(&self) -> bool {
        self.state.lock().unwrap().status().is_playing()
    }

    /// Returns true if the player has just finished audio.
    pub fn is_finished(&self) -> bool {
        self.state.lock().unwrap().status().is_finished()
    }

    /// Returns true if the player is paused.
    pub fn is_paused(&self) -> bool {
        self.state.lock().unwrap().status().is_paused()
    }

    /// Returns true if the player is shutdown.
    pub fn is_shutdown(&self) -> bool {
        self.state.lock().unwrap().status().is_shutdown()
    }

    /// Returns the most recent status from the player.
    ///
    /// # Returns
    /// * Result(PlayerStatus) when the thread is running and emitting
    /// messages.
    /// * Error(err) when the thread sent an error instead.
    pub fn next_status(&self) -> Result<PlayerStatus, Error> {
        self.receiver.recv().unwrap()
    }

    /// Returns the most recent status from the player without blocking.
    ///
    /// # Returns
    /// * Result(PlayerStatus) when the thread is running and emitting
    /// messages.
    /// * Error(err) when the thread sent an error instead.
    pub fn try_next_status(&self) -> Option<Result<PlayerStatus, Error>> {
        if let Ok(s) = self.receiver.try_recv() {
            Some(s)
        }
        else {
            None
        }
    }
}

/// Enumeration type for sending player actions.
pub enum PlayerAction {
    // Station related actions.
    Play(Station),
    Stop,

    // Track related actions.
    Pause,
    Unpause,
    Skip,

    // Misc actions.
    Report,
    Exit,
}
