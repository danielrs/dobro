use libc::{c_char, c_int, size_t};

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

#[link(name="earwax")]
#[link(name="avcodec")]
#[link(name="avformat")]
#[link(name="avutil")]
#[link(name="pthread")]
extern {
    pub fn earwax_init();
    pub fn earwax_shutdown();
    pub fn earwax_new(ctx: *mut *mut EarwaxContext, url: *const c_char) -> c_int;
    pub fn earwax_drop(ctx: *mut *mut EarwaxContext);
    pub fn earwax_spit(data: *mut *mut c_char, len: *mut size_t) -> c_int;
}
