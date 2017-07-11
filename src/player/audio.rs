use super::error::Error;

use ao;
use earwax::{Earwax, Timestamp, LogLevel};

use std::sync::{Once, ONCE_INIT};
static START: Once = ONCE_INIT;

/// Type for audio streaming audio that hides the details of earwax and ao-rs handling.
pub struct Audio {
    earwax: Earwax,
    driver: ao::Driver,
    format: ao::Format,
    device: ao::Device,
}

impl Audio {
    /// Tries to initialize a new stream for the given URL.
    pub fn new(url: &str) -> Result<Self, Error> {
        // #[cfg(not(debug_assertions))]
        START.call_once(|| { Earwax::set_log_level(LogLevel::Error); });

        let earwax = try!(Earwax::new(url));
        let driver = try!(ao::Driver::new());
        let format = ao::Format::new();
        let device = try!(ao::Device::new(&driver, &format, None));

        Ok(Audio {
               earwax: earwax,
               driver: driver,
               format: format,
               device: device,
           })
    }

    /// Plays the next chunk of the stream to the default audio device.
    /// # Returns
    /// If playback was successful (at getting next stream chunk), the value returned
    /// is a tuple where the first element is the current timestamp, and the second
    /// element is the total timestamp.
    pub fn play(&mut self) -> Result<(Timestamp, Timestamp), ()> {
        let duration = self.earwax.info().duration;
        if let Some(chunk) = self.earwax.spit() {
            self.device.play(chunk.data);
            Ok((chunk.time, duration))
        } else {
            Err(())
        }
    }

    /// Plays all the chunks remaining in the stream to the default audio the device.
    pub fn play_all(&mut self) {
        while let Some(chunk) = self.earwax.spit() {
            self.device.play(chunk.data);
        }
    }
}
