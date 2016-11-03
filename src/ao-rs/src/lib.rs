extern crate libc;

mod ffi;

use ffi::*;

use std::ffi::CString;

/// Opaque struct for Ao handling. Make sure only one instance of this
/// type is created.
pub struct Ao;
impl Ao {
    pub fn new() -> Self {
        unsafe { ao_initialize(); }
        Ao
    }

    pub fn reload(&mut self) {
        unsafe {
            ao_shutdown();
            ao_initialize();
        }
    }
}

impl Drop for Ao {
    fn drop(&mut self) {
        unsafe { ao_shutdown(); }
    }
}

/// Ao driver.
pub struct Driver {
    driver_id: i32,
}

impl Driver {
    /// Creates and returns (if-any) the default driver.
    pub fn new() -> Option<Self> {
        let driver_id = unsafe { ao_default_driver_id() };
        if driver_id >= 0 { Some(Driver { driver_id: driver_id }) }
        else { None }
    }

    /// Tries to find a driver with the given name.
    ///
    /// # Panics
    /// If the given name contains 0 bytes.
    pub fn with_name(short_name: &str) -> Option<Self> {
        let short_name = CString::new(short_name).unwrap();
        let driver_id = unsafe { ao_driver_id(short_name.as_ptr()) };
        if driver_id >= 0 { Some(Driver { driver_id: driver_id }) }
        else { None }
    }

    /// Returns the driver id.
    pub fn driver_id() -> i32 {
        driver_id
    }
}

/// Ao device.
pub struct Device {
    device: *mut AoDevice,
}

impl Device {
    pub fn new(driver: &Driver, format: &Format, settings: Option<&Settings>) {
        let ao_device = unsafe {
            ao_open_live(driver.driver_id(), format, settings)
        };
    }
}

/// Ao format.
pub struct Format {
    pub bits: u8,
    pub rate: u32,
    pub channels: u8,
    pub byte_format: ByteFormat,
    // TODO: Implement macros for creating channel formats
    pub channel_format: Option<String>,
}

impl Default for Format {
    fn default() -> Self {
        Format {
            bits: 16,
            rate: 44100,
            channels: 2,
            byte_format: ByteFormat::Little,
            matrix: None,
        }
    }
}

/// Byte format, can either by little-endian, bit-endian, or native (inherits from system).
pub enum ByteFormat {
    Little = 1,
    Big = 2,
    Native = 4,
}

/// Ao settings.
pub struct Settings;
