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
pub use playlist::Track;
pub use stations::StationItem;

//////////
// Module
/////////

use error::Result;
use method::Method;
use music::Music;
use request::request;
use stations::Stations;

use hyper::client::Client;
use hyper::method::Method as HttpMethod;
use serde::Deserialize;
use serde_json::value::Value;

use std::cell::RefCell;

/// Main interface for interacting with the Pandora API
#[derive(Debug)]
pub struct Pandora {
    client: Client,
    endpoint: Endpoint<'static>,
    credentials: RefCell<Credentials>,
}

impl Pandora {
    pub fn new(username: &str, password: &str) -> Result<Self> {
        let credentials = try!(Credentials::new(username, password));
        Ok(Pandora::with_credentials(credentials))
    }

    /// Creates a new Pandora instance from the given credentials.
    pub fn with_credentials(credentials: Credentials) -> Self {
        Pandora {
            client: Client::new(),
            endpoint: DEFAULT_ENDPOINT,
            credentials: RefCell::new(credentials),
        }
    }

    /// Returns an instance of [Music](struct.Music.html).
    pub fn music(&self) -> Music {
        Music::new(self)
    }

    /// Returns an instance of [Stations](struct.Stations.html).
    pub fn stations(&self) -> Stations {
        Stations::new(self)
    }

    /// Proxy method for GET requests.
    pub fn get<T>(&self, method: Method, body: Option<Value>) -> Result<T>
    where T: Deserialize {
        self.request(HttpMethod::Get, method, body)
    }

    /// Proxy method for POST requests.
    pub fn post<T>(&self, method: Method, body: Option<Value>) -> Result<T>
    where T: Deserialize {
        self.request(HttpMethod::Post, method, body)
    }

    fn request<T>(&self, http_method: HttpMethod, method: Method, body: Option<Value>)
    -> Result<T> where T: Deserialize {
        let req = request(
            &self.client,
            &http_method,
            self.endpoint,
            method,
            body.clone(),
            Some(&self.credentials.borrow()),
        );

        // Checks response and tries to revalidate possibly expired
        // credentials once.
        match req {
            Ok(res) => Ok(res),
            Err(err) => {
                // Update credentials.
                if self.credentials.borrow_mut().refresh().is_err() {
                    // If there was an error updating credentials
                    // return first error.
                    return Err(err)
                }
                // Try request again with updated credentials.
                request(
                    &self.client,
                    &http_method,
                    self.endpoint,
                    method,
                    body,
                    Some(&self.credentials.borrow()),
                )
            }
        }
    }
}

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
