extern crate pandora;
extern crate serde_json;

use pandora::*;
use pandora::crypt::*;

fn main() {

    let res = login(DEFAULT_ENDPOINT, "john.doe@gmail.com", "johndoe");
    match res {
        Ok(res) => println!("Ok: {:?}", res),
        Err(e) => println!("Err: {:?}", e),
    }

    // let key = "R=U!LH$O2B#".to_owned();
    // let message = "è.<Ú1477631903".to_owned();
    // let encrypted = encrypt(&key, &message);
    // let mut decrypted = decrypt(&key, &encrypted);
    // println!("Message: {:?}", message);
    // println!("Encrypted: {:?}", encrypted);
    // println!("Decrypted: {:?}", decrypted);
}
