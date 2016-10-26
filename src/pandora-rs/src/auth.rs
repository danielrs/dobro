use super::{Endpoint, Credentials};
use error::Result;
use method::Method;

use std::io::Read;
use serde_json;
use hyper::client::Client;

/// Partner login information.
struct PartnerLogin<'a> {
    username: &'a str,
    password: &'a str,
    #[serde(rename="deviceModel")]
    device_model: &'a str,
    version: &'a str,
}

const PARTNERS : [PartnerLogin<'static>; 2] = [
    PartnerLogin {
        username: "android",
        password: "AC7IBG09A3DTSYM4R41UJWL07VLN8JI7",
        device_model: "android-generic",
        version: "5",
    },
    PartnerLogin {
        username: "iphone",
        password: "P2E4FC0EAD3*878N92B2CDp34I0B1@388137C",
        device_model: "IP0",
        version: "5",
    },
];
const DEFAULT_PARTNER : PartnerLogin<'static> = PARTNERS[0];

// /// User login information.
// struct UserLogin<'a>

pub fn login(endpoint: Endpoint, user: String, password: String) -> Result<Credentials> {
    let method = format!("method={}", Method::AuthPartnerLogin.to_string());
    let url = format!("{}?{}", endpoint.to_string(), method);
    // let client = Client::new().post(url)
    unimplemented!()
}
