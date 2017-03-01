use super::error::Error;
use super::PlayerAction;
use super::state::{PlayerState, PlayerStatus};

use ao;
use earwax::Earwax;
use pandora::Pandora;

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
        /// Sets current player status and sends status event to main thread.
        let set_status = |status: PlayerStatus| {
            state.lock().unwrap().set_status(status.clone());
            sender.send(Ok(status)).unwrap();
        };

        /// This macro tries the given Result, and if an Err is given sends the error using
        /// the sender in the current context and executes the specified action on Error.
        macro_rules! try_or {
            ($e:expr, $action:expr) => {
                match $e {
                    Ok(val) => val,
                    Err(err) => {
                        sender.send(Err(err.into())).unwrap();
                        $action;
                    }
                }
            }
        }

        // TODO: Correct handling of event thread if we return here.
        let driver = try_or!(ao::Driver::new(), return);

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

                'station_loop: loop {
                    set_status(PlayerStatus::Fetching(station.clone()));
                    let tracklist = try_or!(pandora.stations().playlist(station).list(), continue);

                    for track in tracklist {
                        if track.is_ad() { continue; }

                        state.lock().unwrap().set_track(track.clone());
                        set_status(PlayerStatus::Playing(track.clone()));

                        if let Some(ref audio) = track.track_audio {
                            if let Ok(mut earwax) = Earwax::new(&audio.high_quality.audio_url) {
                                // TODO: Format should replicate earwax format.
                                let format = ao::Format::new();
                                let device = try_or!(ao::Device::new(&driver, &format, None), continue);
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
    }).unwrap()
}
