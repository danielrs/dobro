extern crate earwax;

use earwax::ffi::earwax_init;

fn main() {
    unsafe { earwax_init(); }
}
