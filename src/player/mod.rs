mod state;

pub use self::state::*;

use ao;
use earwax::Earwax;

use pandora::{Pandora, Station};

use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, MutexGuard, Condvar};
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
    receiver: Receiver<PlayerStatus>,
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

        let main_state = Arc::new(Mutex::new(PlayerState::new()));
        let main_pause_pair: Arc<(Mutex<bool>, Condvar)> = Arc::new((Mutex::new(false), Condvar::new()));

        let (external_sender, receiver) = channel();
        let (sender, external_receiver) = channel();
        let (event_sender, event_receiver) = channel();

        // Thread is dedicated to receive the events from the main thread and
        // forward player events to the player thread.
        let event_handle = {
            let state = main_state.clone();
            let pause_pair = main_pause_pair.clone();
            let sender = sender.clone();
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
                            sender.send(state.lock().unwrap().status().clone()).unwrap();
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
        let player_handle = thread::Builder::new().name("player".to_string()).spawn(move || {
            let driver = ao::Driver::new().unwrap();

            let set_status = |status: PlayerStatus| {
                state.lock().unwrap().set_status(status.clone());
                sender.send(status).unwrap();
            };

            let mut current_station = None;
            'main_loop: loop {

                // Stand-by, waiting to play a station.
                if current_station.is_none() {
                    set_status(PlayerStatus::Standby);
                    while let Ok(action) = event_receiver.recv() {
                        match action {
                            PlayerAction::Play(new_station) => {
                                current_station = Some(new_station);
                                break;
                            },
                            PlayerAction::Exit => break 'main_loop,
                            _ => (),
                        }
                    }
                }

                // Playing a station.
                if let Some(ref station) = current_station.clone() {
                    state.lock().unwrap().set_station(station.clone());
                    set_status(PlayerStatus::Started(station.clone()));

                    'station_loop: while let Ok(tracklist) = {
                        set_status(PlayerStatus::Fetching(station.clone()));
                        pandora.stations().playlist(station).list()
                    } {
                        for track in tracklist {
                            if track.is_ad() { continue; }

                            state.lock().unwrap().set_track(track.clone());
                            set_status(PlayerStatus::Playing(track.clone()));

                            if let Some(ref audio) = track.track_audio {
                                if let Ok(mut earwax) = Earwax::new(&audio.high_quality.audio_url) {
                                    // TODO: Format should replicate earwax format.
                                    let format = ao::Format::new();
                                    let device = ao::Device::new(&driver, &format, None).unwrap();
                                    let duration = earwax.info().duration.seconds();

                                    while let Some(chunk) = earwax.spit() {
                                        state.lock().unwrap().set_progress(chunk.time.seconds(), duration);

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
                                                PlayerAction::Play(new_station) => {
                                                    current_station = Some(new_station);
                                                    set_status(PlayerStatus::Finished(track.clone()));
                                                    break 'station_loop;
                                                },
                                                PlayerAction::Stop => {
                                                    current_station = None;
                                                    set_status(PlayerStatus::Finished(track.clone()));
                                                    break 'station_loop;
                                                },

                                                PlayerAction::Skip => break,

                                                PlayerAction::Exit => {
                                                    set_status(PlayerStatus::Finished(track.clone()));
                                                    set_status(PlayerStatus::Stopped(station.clone()));
                                                    break 'main_loop;
                                                }
                                                _ => (),
                                            }
                                        }

                                        // Plays chunk.
                                        device.play(chunk.data);
                                    }
                                    set_status(PlayerStatus::Finished(track.clone()));
                                }
                            }
                        }
                    }
                    state.lock().unwrap().clear_info();
                    set_status(PlayerStatus::Stopped(station.clone()));
                }
            } // 'main-loop
            *state.lock().unwrap() = PlayerState::new();
            set_status(PlayerStatus::Shutdown);
            event_handle.join().unwrap();
        }).unwrap();

        Player {
            ao: ao,
            player_handle: Some(player_handle),

            state: main_state,

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
    /// * Some(PlayerStatus) when the thread is running and emitting
    /// messages.
    /// * None when the thread is not running and there are no recent statuses.
    pub fn next_status(&self) -> Option<PlayerStatus> {
        let mut status = None;
        if let Ok(s) = self.receiver.try_recv() {
            status = Some(s);
        }
        status
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
