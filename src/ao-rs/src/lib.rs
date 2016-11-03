extern crate libc;

mod ffi;

use std::ffi::CString;
use std::ptr;

use libc::{c_int, c_char};

/// Opaque struct for Ao handling. Make sure only one instance of this
/// type is created.
pub struct Ao;
impl Ao {
    pub fn new() -> Self {
        unsafe { ffi::ao_initialize(); }
        Ao
    }

    pub fn reload(&mut self) {
        unsafe {
            ffi::ao_shutdown();
            ffi::ao_initialize();
        }
    }
}

impl Drop for Ao {
    fn drop(&mut self) {
        unsafe { ffi::ao_shutdown(); }
    }
}

/// Ao driver.
pub struct Driver {
    driver_id: i32,
}

impl Driver {
    /// Creates and returns (if-any) the default driver.
    pub fn new() -> Option<Self> {
        let driver_id = unsafe { ffi::ao_default_driver_id() };
        if driver_id >= 0 { Some(Driver { driver_id: driver_id }) }
        else { None }
    }

    /// Tries to find a driver with the given name.
    ///
    /// # Panics
    /// If the given name contains 0 bytes.
    pub fn with_name(short_name: &str) -> Option<Self> {
        let short_name = CString::new(short_name).unwrap();
        let driver_id = unsafe { ffi::ao_driver_id(short_name.as_ptr()) };
        if driver_id >= 0 { Some(Driver { driver_id: driver_id }) }
        else { None }
    }

    /// Returns the driver id.
    pub fn driver_id(&self) -> i32 {
        self.driver_id
    }
}

/// Ao device.
pub struct Device {
    device: *mut ffi::AoDevice,
}

impl Device {
    /// Creates a new device using the given driver, format, and settings.
    pub fn new(driver: &Driver, format: &Format, settings: Option<&Settings>) -> Option<Self> {
        let ao_device = unsafe {
            ffi::ao_open_live(driver.driver_id(), &format.to_ao_format(), ptr::null())
        };

        if ao_device.is_null() {
            None
        }
        else {
            Some(Device { device: ao_device })
        }
    }

    /// Plays the given PCM data using the specified format.
    pub fn play(&self, buffer: &[i8]) {
        unsafe {
            ffi::ao_play(self.device, buffer.as_ptr(), buffer.len() as u32);
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            if !self.device.is_null() {
                ffi::ao_close(self.device);
            }
        }
    }
}

/// Ao sample format.
pub struct Format {
    pub bits: u32,
    pub rate: u32,
    pub channels: u32,
    pub byte_format: ByteFormat,
    // TODO: Implement macros for creating channel formats
    pub channel_format: Option<String>,
}

impl Format {
    /// Creates a new default format.
    pub fn new() -> Self {
        Format::default()
    }

    /// Returns reference as AoFormat.
    pub fn to_ao_format(&self) -> ffi::AoFormat {
        ffi::AoFormat {
            bits: self.bits.clone() as c_int,
            rate: self.rate.clone() as c_int,
            channels: self.channels.clone() as c_int,
            byte_format: self.byte_format.clone() as c_int,
            matrix: ptr::null_mut(),
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Format {
            bits: 16,
            rate: 44100,
            channels: 2,
            byte_format: ByteFormat::Little,
            channel_format: None,
        }
    }
}

/// Byte format, can either by little-endian, bit-endian, or native (inherits from system).
#[derive(Copy, Clone)]
pub enum ByteFormat {
    Little = 1,
    Big = 2,
    Native = 4,
}

/// Ao settings.
pub struct Settings;
