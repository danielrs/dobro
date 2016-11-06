extern crate earwax;

use earwax::Earwax;
use std::thread;

fn main() {
    let handles: Vec<_> = (0..10).into_iter().map(|i| {
        thread::spawn(move || {
            for i in 0..10 {
                match Earwax::new("track.mp4") {
                    Ok(_) => println!("DONE"),
                    Err(_) => unreachable!(),
                }
            }
        })
    }).collect();

    for h in handles {
        h.join().unwrap();
    }
}
