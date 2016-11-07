extern crate earwax;
extern crate ao;

use earwax::Earwax;
use ao::*;

fn main() {
    let ao = Ao::new();
    let driver = Driver::new().unwrap();
    let format = Format::new();
    let device = Device::new(&driver, &format, None).unwrap();

    let mut earwax = Earwax::new("./tracks/Pachelbel - Canon in D Major.mp3").unwrap();
    earwax.spit(|data| {
        device.play(data);
    });
}
