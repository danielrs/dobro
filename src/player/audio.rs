use super::error::Error;

use ao;
use earwax::{Earwax, Timestamp};

/// Type for streaming audio playback that hides the details earwax and ao-rs handling.
pub struct Audio {
    earwax: Earwax,
    driver: ao::Driver,
    format: ao::Format,
    device: ao::Device,
}

impl Audio {
    /// Tries to initialize a new stream for the given URL.
    pub fn new(url: &str) -> Result<Self, Error> {
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

    /// Plays the next chunk of the stream to the default audio device. The passed
    /// closure consumes the chunk (after being played).
    pub fn play<F>(&mut self, closure: F) -> Result<(), ()> where F: FnOnce(Timestamp, Timestamp) {
        let duration = self.earwax.info().duration;
        if let Some(chunk) = self.earwax.spit() {
            self.device.play(chunk.data);
            closure(chunk.time, duration);
            Ok(())
        }
        else {
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
