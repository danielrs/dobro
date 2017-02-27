//! This module contains the ffi bindings for libao. Check the
//! [official docs](https://www.xiph.org/ao/doc/) for more
//! information.

use libc::{c_int, c_char, uint32_t};

/// Opaque structure for libao's ao_device.
#[derive(Debug)]
pub enum AoDevice {}

/// Structure for libao's ao_sample_format.
#[repr(C)]
pub struct AoFormat {
    pub bits: c_int,
    pub rate: c_int,
    pub channels: c_int,
    pub byte_format: c_int,
    pub matrix: *mut c_char,
}

/// Structure for libao's ao_option.
#[repr(C)]
pub struct AoOption {
    key: *mut c_char,
    value: *mut c_char,
    next: *mut AoOption,
}

#[link(name="ao")]
extern {
    pub fn ao_initialize();
    pub fn ao_shutdown();

    pub fn ao_driver_id(short_name: *const c_char) -> c_int;
    pub fn ao_default_driver_id() -> c_int;
    pub fn ao_open_live(driver_id: c_int, format: *const AoFormat, options: *const AoOption) -> *mut AoDevice;
    pub fn ao_play(ao_device: *const AoDevice, output_samples: *const c_char, num_bytes: uint32_t) -> c_int;
    pub fn ao_close(ao_device: *mut AoDevice) -> c_int;

    pub fn ao_append_option(options: *mut *mut AoOption, key: *const c_char, value: *const c_char);
    pub fn ao_free_options(options: *mut AoOption);
}
