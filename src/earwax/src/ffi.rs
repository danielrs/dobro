use libc::{c_char, c_int, size_t, int64_t};
use std::ptr;

#[derive(Debug)]
pub enum EarwaxErrorCode {
    Io = 100,
    AudioStreamNotFound,
    DecoderNotFound,
    UnableToOpenDecoder,
    Unknown,
}

impl EarwaxErrorCode {
    pub fn from_c_int(code: c_int) -> Self {
        match code {
            100 => EarwaxErrorCode::Io,
            101 => EarwaxErrorCode::AudioStreamNotFound,
            102 => EarwaxErrorCode::DecoderNotFound,
            103 => EarwaxErrorCode::UnableToOpenDecoder,
            _ => EarwaxErrorCode::Unknown,
        }
    }
}

impl From<c_int> for EarwaxErrorCode {
    fn from(code: c_int) -> EarwaxErrorCode {
        EarwaxErrorCode::from_c_int(code)
    }
}

pub enum EarwaxContext {}

#[repr(C)]
pub struct EarwaxInfo {
    pub bitrate: c_int,
    pub sample_rate: c_int,
    pub duration: int64_t,
}

#[repr(C)]
pub struct EarwaxChunk {
    pub data: *mut c_char,
    pub size: size_t,
    pub time: int64_t,
}

impl EarwaxChunk {
    pub fn new() -> Self {
        EarwaxChunk {
            data: ptr::null_mut(),
            size: 0,
            time: 0,
        }
    }
}

#[link(name="earwax")]
#[link(name="avcodec")]
#[link(name="avformat")]
#[link(name="swresample")]
#[link(name="avutil")]
#[link(name="pthread")]
extern {
    pub fn earwax_init();
    pub fn earwax_shutdown();
    pub fn earwax_new(ctx: *mut *mut EarwaxContext, url: *const c_char) -> c_int;
    pub fn earwax_drop(ctx: *mut *mut EarwaxContext);
    pub fn earwax_get_info(ctx: *mut EarwaxContext, info: *mut EarwaxInfo);
    pub fn earwax_spit(ctx: *mut EarwaxContext, chunk: *mut EarwaxChunk) -> c_int;
}
