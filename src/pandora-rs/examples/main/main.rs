extern crate pandora;
extern crate serde_json;

use pandora::*;

extern crate hyper;
use hyper::Url;
use hyper::client::Client;

use std::io::Read;

fn main() {
    let login = PartnerLogin {
        username: "android".to_owned(),
        password: "AC7IBG09A3DTSYM4R41UJWL07VLN8JI7".to_owned(),
        device_model: "android-generic".to_owned(),
        version: "5".to_owned(),
        include_urls: false,
    };

    let request = Request::new(Method::AuthPartnerLogin)
        .with_body(serde_json::to_value(login));

    let client = Client::new();
    let mut res = client.post(&request.get_url())
        .body(&request.get_body())
        .send().unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body);

    println!("{} says:\n{:?}", request.get_url(), body);
}
