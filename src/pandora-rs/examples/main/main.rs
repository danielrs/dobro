extern crate pandora;
extern crate serde_json;

use pandora::Pandora;

fn main() {
    let res = Pandora::new("john.doe@gmail.com", "johndoe"); // <- real login here
    match res {
        Ok(pandora) => {
            for station in pandora.stations().list().unwrap() {
                println!("{:?}", station);
            }
        },
        Err(e) => println!("Err: {:?}", e),
    }
}
