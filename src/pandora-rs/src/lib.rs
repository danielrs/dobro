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
pub mod music;
pub mod playlist;
pub mod request;
mod response;
pub mod stations;

pub use auth::Credentials;

use error::Result;
use method::Method;
use music::Music;
use request::request;
use stations::Stations;

// External imports.
use hyper::client::Client;
use hyper::method::Method as HttpMethod;
use serde::Deserialize;
use serde_json::value::Value;

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
    client: Client,
    endpoint: Endpoint<'a>,
    credentials: Credentials,
}

impl<'a> Pandora<'a> {
    /// Creates a new Pandora instance from the given credentials.
    pub fn with_credentials(credentials: Credentials) -> Self {
        Pandora {
            client: Client::new(),
            endpoint: DEFAULT_ENDPOINT,
            credentials: credentials,
        }
    }

    /// Returns [Music](struct.Music.html) struct for different music related
    /// methods.
    pub fn music(&self) -> Music {
        Music::new(self)
    }

    /// Returns a handler to Stations.
    pub fn stations(&self) -> Stations {
        Stations::new(self)
    }

    /// Proxy method for GET requests.
    pub fn get<T>(&self, method: Method, body: Option<Value>) -> Result<T>
    where T: Deserialize {
        request(
            &self.client,
            HttpMethod::Get,
            self.endpoint,
            method,
            body,
            Some(&self.credentials),
        )
    }

    /// Proxy method for POST requests.
    pub fn post<T>(&self, method: Method, body: Option<Value>) -> Result<T>
    where T: Deserialize {
        request(
            &self.client,
            HttpMethod::Post,
            self.endpoint,
            method,
            body,
            Some(&self.credentials),
        )
    }
}
