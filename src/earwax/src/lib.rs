#![feature(unique)]

extern crate libc;

pub mod error;
pub mod ffi;

use error::{Error, Result};

use libc::c_char;
use std::ffi::CString;
use std::ptr;
use std::ptr::Unique;

pub struct Earwax {
    earwax_context: Unique<ffi::EarwaxContext>,
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
                Ok(Earwax { earwax_context: Unique::new(earwax_context) })
            }
            else {
                Err(Error::FFI(res.into()))
            }
        }
    }

    pub fn spit<F>(callback: F) where F: FnOnce(&[u8]) {
        unsafe {
            let mut data: *mut c_char = ptr::null_mut();
            let mut len = 0;
            while ffi::earwax_spit(&mut data, &mut len) >= 0 {
                println!("Data.");
            }
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
