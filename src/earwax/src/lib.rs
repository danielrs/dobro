#![feature(unique)]

extern crate libc;
extern crate num;

pub mod error;
pub mod ffi;

use error::{Error, Result};

use libc::c_char;
use num::rational::Rational64;

use std::ffi::CString;
use std::ptr;
use std::ptr::Unique;

/**
 * Earwax context. This struct contains the stream data
 * and main methods getting and seeking data.
 */
pub struct Earwax {
    earwax_context: Unique<ffi::EarwaxContext>,
    info: Info,
}



impl Earwax {
    /// Creates a new Earwax instance from the given url.
    /// # Returns
    /// Some(Earwax) if everything went fine.
    /// None if something went wrong with ffmpeg.
    pub fn new(url: &str) -> Result<Self> {
        let url = try!(CString::new(url));
        let mut earwax_context = ptr::null_mut();
        unsafe {
            ffi::earwax_init();
            let res = ffi::earwax_new(&mut earwax_context, url.as_ptr());
            if res == 0 {
                let mut info = ffi::EarwaxInfo::new();
                ffi::earwax_get_info(earwax_context, &mut info);
                let time_base = Rational64::new(info.time_base.num, info.time_base.den);

                Ok(Earwax {
                    earwax_context: Unique::new(earwax_context),
                    info: Info {
                        bitrate: info.bitrate,
                        sample_rate: info.sample_rate,
                        start_time: Timestamp::from_pts(time_base, info.start_time),
                        duration: Timestamp::from_pts(time_base, info.duration),
                        time_base: time_base,
                    }
                })
            }
            else {
                Err(Error::FFI(res.into()))
            }
        }
    }

    /// Returns the information for this Earwax stream.
    pub fn info(&self) -> &Info {
        &self.info
    }

    /// Returns the next decoded chunk of PCM data. None if
    /// the end of the stream was reached.
    pub fn spit(&mut self) -> Option<Chunk> {
        unsafe {
            let mut chunk = ffi::EarwaxChunk::new();
            if ffi::earwax_spit(*self.earwax_context, &mut chunk) > 0 {
                let slice = std::slice::from_raw_parts(chunk.data, chunk.size);
                Some(Chunk {
                    data: slice,
                    time: Timestamp::from_pts(self.info().time_base, chunk.time),
                })
            }
            else {
                None
            }
        }
    }

    /// Seeks to the given seconds.
    pub fn seek(&mut self, seconds: i64) {
        let time_base = self.info.time_base;
        self.seek_pts(seconds * time_base.denom() / time_base.numer())
    }

    /// Seeks to the given pts.
    pub fn seek_pts(&mut self, pts: i64) {
        unsafe {
            ffi::earwax_seek(*self.earwax_context, pts);
        }
    }
}

impl Drop for Earwax {
    fn drop(&mut self) {
        unsafe {
            let mut ctx = self.earwax_context.as_mut().unwrap() as *mut ffi::EarwaxContext;
            ffi::earwax_drop(&mut ctx);
            ffi::earwax_shutdown();
        }
    }
}

/// A chunk represents a piece of decoded PCM data in 16-bit signed
/// format.
#[derive(Debug)]
pub struct Chunk<'a> {
    pub data: &'a [i8],
    pub time: Timestamp,
}

/// Information about the Earwax context it is attached to.
#[derive(Debug)]
pub struct Info {
    pub bitrate: i32,
    pub sample_rate: i32,
    pub start_time: Timestamp,
    pub duration: Timestamp,
    pub time_base: Rational64,
}

/// Used for representing points in time of the audio
/// stream.
#[derive(Debug, Copy, Clone)]
pub struct Timestamp {
    time_base: Rational64,
    pts: i64,
}

impl Timestamp {
    pub fn new(time_base: Rational64) -> Self {
        Timestamp {
            time_base: time_base,
            pts: 0,
        }
    }

    pub fn from_seconds(time_base: Rational64, seconds: i64) -> Self {
        let pts = seconds * time_base.denom() * time_base.numer();
        Timestamp {
            time_base: time_base,
            pts: pts,
        }
    }

    pub fn from_pts(time_base: Rational64, pts: i64) -> Self {
        Timestamp {
            time_base: time_base,
            pts: pts,
        }
    }

    pub fn seconds(&self) -> i64 {
        (self.pts * self.time_base.numer()) / self.time_base.denom()
    }

    pub fn set_seconds(&mut self, seconds: i64) {
        self.pts = seconds * self.time_base.denom() * self.time_base.numer();
    }

    pub fn pts(&self) -> i64 {
        self.pts
    }

    pub fn set_pts(&mut self, pts: i64) {
        self.pts = pts
    }
}
