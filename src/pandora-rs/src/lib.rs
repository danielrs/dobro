#![feature(proc_macro)]
#![feature(custom_attribute)]

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate hyper;

mod auth;
mod error;
mod method;
// mod request;
// pub use method::*;
// pub use request::*;

/// Endpoint of the Pandora API
pub struct Endpoint<'a>(&'a str);

impl<'a> ToString for Endpoint<'a> {
    fn to_string(&self) -> String {
        let Endpoint(url) = *self;
        url.to_owned()
    }
}

const ENDPOINTS : [Endpoint<'static>; 4] = [
    Endpoint("http://tuner.pandora.com/services/json/"),
    Endpoint("https://tuner.pandora.com/services/json/"),
    Endpoint("http://internal-tuner.pandora.com/services/json/"),
    Endpoint("https://internal-tuner.pandora.com/services/json/"),
];
const DEFAULT_ENDPOINT : Endpoint<'static> = ENDPOINTS[0];

/// The authentication details.
pub enum Credentials {
    None,
    Token {
        auth_token: String,
        partner_id: String,
        user_id: String,
    }
}

impl Default for Credentials {
    fn default() -> Credentials {
        Credentials::None
    }
}

/// Main interface for interacting with the Pandora API
pub struct Pandora<'a> {
    endpoint: Endpoint<'a>,
    credentials: Credentials,
}

impl<'a> Pandora<'a> {
    /// Creates a new Pandora instance.
    pub fn new(credentials: Credentials) -> Self {
        Pandora::new_with_endpoint(DEFAULT_ENDPOINT, credentials)
    }

    /// Creates a new Pandora instance with the given endponit.
    pub fn new_with_endpoint(endpoint: Endpoint<'a>, credentials: Credentials) -> Self {
        Pandora {
            endpoint: endpoint,
            credentials: credentials,
        }
    }
}
