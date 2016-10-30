#![feature(proc_macro)]
#![feature(custom_attribute)]

extern crate crypto;

extern crate hyper;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate url;

pub mod auth;
pub mod crypt;
pub mod error;
pub mod method;
pub mod request;
mod response;

pub use auth::Credentials;

// pub use method::*;
// pub use request::*;

/// Endpoint of the Pandora API
#[derive(Debug, Copy, Clone)]
pub struct Endpoint<'a>(&'a str);

impl<'a> ToString for Endpoint<'a> {
    fn to_string(&self) -> String {
        let Endpoint(url) = *self;
        url.to_owned()
    }
}

pub const ENDPOINTS : [Endpoint<'static>; 4] = [
    Endpoint("http://tuner.pandora.com/services/json/"),
    Endpoint("https://tuner.pandora.com/services/json/"),
    Endpoint("http://internal-tuner.pandora.com/services/json/"),
    Endpoint("https://internal-tuner.pandora.com/services/json/"),
];
pub const DEFAULT_ENDPOINT : Endpoint<'static> = ENDPOINTS[0];

/// Main interface for interacting with the Pandora API
#[derive(Debug)]
pub struct Pandora<'a> {
    endpoint: Endpoint<'a>,
    credentials: Credentials,
}

impl<'a> Pandora<'a> {
    /// Creates a new Pandora instance from the given credentials.
    pub fn with_credentials(credentials: Credentials) -> Self {
        Pandora {
            endpoint: DEFAULT_ENDPOINT,
            credentials: credentials,
        }
    }
}
