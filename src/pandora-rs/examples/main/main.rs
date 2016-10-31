extern crate pandora;
extern crate serde_json;

use pandora::Pandora;

fn main() {
    let res = Pandora::new("john.doe@gmail.com", "johndoe"); // <- real login here
    match res {
        Ok(pandora) => {
            let station_handler = pandora.stations();
            for station in station_handler.list().unwrap() {
                println!("== Tracks for \"{}\"", station.station_name);
                let playlist = station_handler.playlist(&station);
                let tracklist = playlist.list().unwrap();

                for track in tracklist {
                    println!("{:?}", track);
                }
            }
        },
        Err(e) => println!("Err: {:?}", e),
    }
}
