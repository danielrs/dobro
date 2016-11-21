//! This example is a port of the example found in libao's documentation:
//! https://www.xiph.org/ao/doc/ao_example.c

extern crate ao;

use ao::*;
use std::f32::consts::PI;

fn main() {
    let ao = Ao::new();
    let driver = Driver::new().unwrap();
    let format = Format::new();
    let device = Device::new(&driver, &format, None).unwrap();
    let freq = 440.0;

    // Create PCM data formatted as 2 channels
    // of 16 bits each (Time1, Channel1; Time2, Channel2...).
    let buff_size = format.bits / 8 * format.channels * format.rate;
    let mut buffer: Vec<i8> = vec![0; buff_size as usize];
    for (i, chunk) in buffer.chunks_mut(4).enumerate() {
        let sin = (2.0 * PI * freq * (i as f32) / (format.rate as f32)).sin();
        let sample = (0.75 * 32768.0 * sin) as i16;
        chunk[0] = (sample & 0xff) as i8; chunk[2] = chunk[0];
        chunk[1] = ((sample >> 8) & 0xff) as i8; chunk[3] = chunk[1];
    }

    device.play(&buffer);
}
