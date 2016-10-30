extern crate pandora;
extern crate serde_json;

use pandora::Pandora;

fn main() {
    // Replace the user and login with real credentials to get
    // a valid response.
    let res = Pandora::new("john.doe@gmail.com", "johndoe");
    match res {
        Ok(res) => println!("Ok: {:?}", res),
        Err(e) => println!("Err: {:?}", e),
    }
}
