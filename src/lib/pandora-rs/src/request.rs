//! Common functions for interacting with the unofficial Pandora API.

use super::Endpoint;
use auth::Credentials;
use crypt;
use error::{Error, Result};
use method::Method;
use response::{Stat, Response};

use std::io::Read;

use hyper::client::{RequestBuilder, Client};
use hyper::header::ContentLength;
use hyper::method::Method as HttpMethod;

use serde::Deserialize;
use serde::ser::Error as SerdeError;
use serde_json;
use serde_json::value::{Value};
use serde_json::error::Error as JsonError;

use url::Url;


pub fn request<T>(
    client: &Client, http_method: &HttpMethod, endpoint: Endpoint, method: Method, body: Option<Value>,
    credentials: Option<&Credentials>) -> Result<T> where T: Deserialize {

    let mut body = try!(serde_json::to_string(&authenticate_body(body, credentials)));
    if method.is_encrypted() {
        if let Some(credentials) = credentials {
            body = crypt::encrypt(credentials.encrypt_key(), &body);
        }
    }

    let builder = authenticate(client, http_method, endpoint, method, credentials);

    let mut res = try!(builder.body(&body).send());
    let mut body = match res.headers.clone().get::<ContentLength>() {
        Some(&ContentLength(len)) => String::with_capacity(len as usize),
        None => String::new(),
    };
    try!(res.read_to_string(&mut body));

    debug!("== Received response ==\nStatus: {:?}\nHeaders: {:?}\nBody: {:?}", res.status, res.headers, body);

    let res: Response<T> = try!(serde_json::from_str(&body));
    match res {
        Response { stat: Stat::Ok, result: Some(result), .. } => {
            Ok(result)
        },
        Response { stat: Stat::Ok, result: None, .. } => {
            Err(Error::Codec(JsonError::custom("Nothing to deserialize")))
        },
        Response { stat: Stat::Fail, .. } => {
            Err(Error::Api { message: res.message.unwrap(), code: res.code.unwrap().into() })
        },
    }
}

/// Returns a RequestBuilder with the HTTP method and URL set. The URL query string
/// will include the auth information if credentials were provided.
fn authenticate<'a>(
    client: &'a Client, http_method: &HttpMethod, endpoint: Endpoint, method: Method,
    credentials: Option<&Credentials>) -> RequestBuilder<'a> {

    let url = format!("{}?method={}", endpoint.to_string(), method.to_string());
    let mut url = Url::parse(&url).unwrap();

    if let Some(credentials) = credentials {
        use std::collections::BTreeMap;
        let mut query_pairs: BTreeMap<&str, &str> = BTreeMap::new();
        if let Some(partner_auth_token) = credentials.partner_auth_token() {
            query_pairs.insert("auth_token", partner_auth_token);
        }
        if let Some(user_auth_token) = credentials.user_auth_token() {
            query_pairs.insert("auth_token", user_auth_token);
        }
        if let Some(partner_id) = credentials.partner_id() {
            query_pairs.insert("partner_id", partner_id);
        }
        if let Some(user_id) = credentials.user_id() {
            query_pairs.insert("user_id", user_id);
        }
        url.query_pairs_mut().extend_pairs(query_pairs);
    }

    debug!("== URL ==\n{:?}", url);
    client.request(http_method.clone(), url)
}

/// Returns the authenticated body.
///
/// # Arguments
/// * `body` - If no body is provided  a new object is created instead. If a body is provided
/// but is not an object, then the function does nothing and returns the same body.
/// * `credentials` - The credentials to use when adding the auth information to the body.
fn authenticate_body(body: Option<Value>, credentials: Option<&Credentials>) -> Value {
    let mut body = match body {
        Some(body) => body,
        None => serde_json::to_value(serde_json::Map::<String, Value>::new()),
    };

    if let Some(credentials) = credentials {
        if let Some(obj) = body.as_object_mut() {
            if let Some(partner_auth_token) = credentials.partner_auth_token() {
                obj.insert("partnerAuthToken".to_owned(), Value::String(partner_auth_token.to_owned()));
            }
            if let Some(sync_time) = credentials.sync_time() {
                obj.insert("syncTime".to_owned(), Value::U64(sync_time.clone()));
            }
            if let Some(user_auth_token) = credentials.user_auth_token() {
                obj.insert("userAuthToken".to_owned(), Value::String(user_auth_token.to_owned()));
            }
        }
    }

    debug!("== Body created ==\n{:?}", body);
    body
}
