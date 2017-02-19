#![feature(proc_macro)]

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
pub use stations::Station;

//////////
// Module
/////////

use error::{Error, Result};
use method::Method;
use music::Music;
use request::request;
use stations::Stations;

use hyper::client::Client;
use hyper::method::Method as HttpMethod;
use serde::Deserialize;
use serde_json::value::Value;

use std::sync::{Mutex};
use std::cell::RefCell;

/// Main interface for interacting with the Pandora API.
/// A Pandora instance is thread-safe, since it doesn't
/// really uses any state; only the credentials, which
/// are protected by a Mutex.
#[derive(Debug)]
pub struct Pandora {
    client: Client,
    endpoint: Endpoint<'static>,
    credentials: Mutex<RefCell<Credentials>>,
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
            credentials: Mutex::new(RefCell::new(credentials)),
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

    /// Proxy method for GET requests that do not return data.
    pub fn get_noop(&self, method: Method, body: Option<Value>) -> Result<()> {
        self.request::<()>(HttpMethod::Get, method, body)
    }

    /// Proxy method for POST requests.
    pub fn post<T>(&self, method: Method, body: Option<Value>) -> Result<T>
    where T: Deserialize {
        self.request(HttpMethod::Post, method, body)
    }

    /// Proxy method for POST requests that do not return data.
    pub fn post_noop(&self, method: Method, body: Option<Value>) -> Result<()> {
        self.request_noop(HttpMethod::Post, method, body)
    }

    fn request<T>(&self, http_method: HttpMethod, method: Method, body: Option<Value>)
    -> Result<T> where T: Deserialize {
        let credentials = self.credentials.lock().unwrap();

        let req = request(
            &self.client,
            &http_method,
            self.endpoint,
            method,
            body.clone(),
            Some(&credentials.borrow()),
        );

        // Checks response and tries to revalidate possibly expired
        // credentials once.
        match req {
            Ok(res) => Ok(res),
            Err(err) => {
                // Update credentials.
                if credentials.borrow_mut().refresh().is_err() {
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
                    Some(&credentials.borrow()),
                )
            }
        }
    }

    fn request_noop(&self, http_method: HttpMethod, method: Method, body: Option<Value>)
    -> Result<()> {
        let credentials = self.credentials.lock().unwrap();

        let req = request::<()>(
            &self.client,
            &http_method,
            self.endpoint,
            method,
            body.clone(),
            Some(&credentials.borrow()),
        );

        // Checks response and tries to revalidate possibly expired
        // credentials once.
        match req {
            Ok(_) | Err(Error::Codec(_)) => Ok(()),
            Err(err) => {
                // Update credentials.
                if credentials.borrow_mut().refresh().is_err() {
                    // If there was an error updating credentials
                    // return first error.
                    return Err(err)
                }

                // Try request again with updated credentials.
                let req = request::<()>(
                    &self.client,
                    &http_method,
                    self.endpoint,
                    method,
                    body,
                    Some(&credentials.borrow()),
                );

                match req {
                    Ok(_) | Err(Error::Codec(_)) => Ok(()),
                    Err(err) => Err(err),
                }
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
