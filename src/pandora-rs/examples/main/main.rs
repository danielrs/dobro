extern crate pandora;
extern crate serde_json;

use pandora::Pandora;

fn main() {
    // Replace the username and password with real credentials to get
    // a valid response.
    let res = Pandora::new("john.doe@gmail.com", "johndoe");
    match res {
        Ok(res) => {
            let stations = res.stations();
            let station_list = stations.list().unwrap();
            println!("{:?}", station_list);
        },
        Err(e) => println!("Err: {:?}", e),
    }
}
