use super::avutil::{AVClass, AVIOContext};

use libc::{c_int, c_uint, uint8_t, uint64_t, c_char, c_void};

#[repr(C)]
pub struct AVFormatContext {
    class: *const AVClass,
    iformat: *mut AVInputFormat,
    oformat: *mut AVOutputFormat,
    priv_data: *mut c_void,
    pb: *mut AVIOContext,
    ctx_flags: c_int,
    nb_streams: c_uint,
    streams: *mut *mut AVStream,
    filename: *mut c_char,
    start_time: uint64_t,
    duration: uint64_t,
    bit_rate: uint64_t,
}

#[repr(C)]
pub struct AVStream {}

pub enum AVInputFormat {}
pub enum AVOutputFormat {}
pub enum AVDictionary {}

#[link(name="avformat")]
extern {
    pub fn avformat_network_init();
    pub fn avformat_network_deinit();

    pub fn av_register_all();
    pub fn avformat_open_file(ctx: *mut *mut AVFormatContext, url: *const c_char, fmt: *mut AVInputFormat, options: *mut *mut AVDictionary);
}
