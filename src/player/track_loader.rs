use pandora::Track;
use super::audio::Audio;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;

/// TrackLoader type for loading tracks in the background.
pub struct TrackLoader {
    tracklist: Arc<Mutex<VecDeque<Track>>>,
    next: Arc<Mutex<Option<(Track, Audio)>>>,
    fetching: Arc<(Mutex<bool>, Condvar)>,
}

impl TrackLoader {
    /// Creates a new TrackLoader form the given tracklist.
    pub fn new(tracklist: VecDeque<Track>) -> Self {
        let mut track_loader = TrackLoader {
            tracklist: Arc::new(Mutex::new(tracklist)),
            next: Arc::new(Mutex::new(None)),
            fetching: Arc::new((Mutex::new(false), Condvar::new())),
        };
        // track_loader.fetch();
        track_loader
    }

    /// Returns the next track and audio, `None` is no more
    /// items available.
    pub fn next(&mut self) -> Option<(Track, Audio)> {
        // // Wait until we are done fetching.
        // {
        //     let &(ref lock, ref cvar) = &*self.fetching;
        //     let mut fetching = lock.lock().unwrap();
        //     while *fetching {
        //         fetching = cvar.wait(fetching).unwrap();
        //     }
        // }

        // let next = self.next.lock().unwrap().take();
        // self.fetch();
        let next = pop_tracklist(self.tracklist.clone());
        return next;
    }

    // /// Fetches the next track in the background.
    // fn fetch(&mut self) {
    //     let tracklist = self.tracklist.clone();
    //     let next = self.next.clone();
    //     let pair = self.fetching.clone();

    //     if tracklist.lock().unwrap().len() > 0 {
    //         let &(ref lock, ref cvar) = &*self.fetching;
    //         let mut fetching = lock.lock().unwrap();
    //         *fetching = true;

    //         thread::spawn(move || {
    //             if next.lock().unwrap().is_none() {
    //                 *next.lock().unwrap() = pop_tracklist(tracklist);
    //             }
    //             let &(ref lock, ref cvar) = &*pair;
    //             let mut fetching = lock.lock().unwrap();
    //             *fetching = false;
    //             cvar.notify_one();
    //         });
    //     }
    // }
}

/// Pops the next track from the tracklist and returns it along
/// with the audio.
fn pop_tracklist(tracklist: Arc<Mutex<VecDeque<Track>>>) -> Option<(Track, Audio)> {
    if let Some((track, audio)) =
        tracklist
            .lock()
            .unwrap()
            .pop_front()
            .and_then(|track| track.track_audio.clone().map(|audio| (track, audio))) {
        let audio = match Audio::new(&audio.high_quality.audio_url) {
            Ok(audio) => audio,
            Err(e) => {
                return None;
            }
        };
        return Some((track, audio));
    }
    None
}
