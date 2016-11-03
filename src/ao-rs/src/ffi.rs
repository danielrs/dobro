//! This module contains the ffi bindings for libao
extern crate libc;

use std::ptr;
use std::f32::consts::{PI};
use libc::{c_int, c_char};

// bitflags! {
//     flags AoByteFormat: c_int {
//         const AO_FMT_LITTLE = 1,
//         const AO_FMT_BIG = 2,
//         const AO_FMT_NATIVE = 4,
//     }
// }

const AO_FMT_LITTLE: c_int = 1;
const AO_FMT_BIG: c_int = 2;
const AO_FMT_NATIVE: c_int = 4;

// Opaque structures for c inter-operation.

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

impl Default for AoFormat {
    fn default() -> Self {
        AoFormat {
            bits: 16,
            rate: 44100,
            channels: 2,
            byte_format: AO_FMT_LITTLE,
            matrix: ptr::null_mut(),
        }
    }
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
    static mut errno: c_int;

    pub fn ao_initialize();
    pub fn ao_driver_id(short_name: *const c_char) -> c_int;
    pub fn ao_default_driver_id() -> c_int;
    pub fn ao_open_live(driver_id: c_int, format: *const AoFormat, options: *const AoOption) -> *mut AoDevice;
    pub fn ao_play(ao_device: *const AoDevice, output_samples: *const c_char, num_bytes: u32) -> c_int;
    pub fn ao_close(ao_device: *const AoDevice) -> c_int;
    pub fn ao_shutdown();
}

// fn main() {
//     let driver = unsafe {
//         ao_initialize();
//         ao_default_driver_id()
//     };

//     let freq = 440.0;
//     let mut format = AoFormat::default();

//     let device = unsafe { ao_open_live(driver, &mut format, ptr::null_mut()) };

//     // Creates buffer.
//     let buff_size = (format.bits / 8 * format.channels * format.rate) as u32;
//     let mut buffer: Vec<i8> = Vec::with_capacity(buff_size as usize);
//     unsafe { buffer.set_len(buff_size as usize); }
//     for (i, chunk) in buffer.chunks_mut(4).enumerate() {
//         let sample = (0.75 * 32768.0 * (2.0 * PI * freq * (i as f32) / (format.rate as f32)).sin()) as i16;
//         chunk[0] = (sample & 0xff) as i8;
//         chunk[2] = (sample & 0xff) as i8;
//         chunk[1] = ((sample >> 8) & 0xff) as i8;
//         chunk[3] = ((sample >> 8) & 0xff) as i8;
//         println!("{:?}", chunk);
//     }

//     let mut ret = -10;
//     unsafe {
//         ret = ao_play(device, buffer.as_mut_ptr(), buff_size);
//         ao_close(device);
//         ao_shutdown();
//     }

//     println!("Device: {:?}", device);
//     println!("Ret: {:?}", buff_size);
// }
