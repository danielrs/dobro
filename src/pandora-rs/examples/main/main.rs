extern crate pandora;
extern crate serde_json;

use pandora::*;

fn main() {

    // println!("Hex: {}", encrypt_bytes("secret".as_bytes(), "String 'Online Domain Tools' encrypted with BLOWFISH (EBC mode)".as_bytes()));

    let res = login(DEFAULT_ENDPOINT, "john.doe@gmail.com", "johndoe");
    match res {
        Ok(res) => println!("Ok: {:?}", res),
        Err(e) => println!("Err: {:?}", e),
    }
}
