use super::Endpoint;
use auth::Credentials;
use crypt::encrypt;
use error::{Error, Result};
use method::Method;
use response::{Stat, Response};

use std::io::Read;
use std::ffi::OsString;

use hyper::client::{RequestBuilder, Client};
use hyper::header::ContentLength;
use hyper::method::Method as HttpMethod;

use serde_json;
use serde::Deserialize;
use serde_json::value::{Value};

use url::Url;

pub fn request<T>(
    client: &Client, http_method: HttpMethod, endpoint: Endpoint, method: Method, body: Option<Value>)
    -> Result<T> where T: Deserialize {
    request_with_credentials(client, http_method, endpoint, method, body, None)
}

pub fn request_with_credentials<T>(
    client: &Client, http_method: HttpMethod, endpoint: Endpoint, method: Method, body: Option<Value>,
    credentials: Option<&Credentials>) -> Result<T> where T: Deserialize {

    let mut body = try!(serde_json::to_string(&authenticate_body(body, credentials)));
    if method.is_encrypted() {
        if let Some(credentials) = credentials {
            body = encrypt(credentials.encrypt_key(), &body);
        }
    }

    let builder = authenticate(client, http_method, endpoint, method, credentials);

    // println!("{:?}", body);

    let mut res = try!(builder.body(&body).send());
    let mut body = match res.headers.clone().get::<ContentLength>() {
        Some(&ContentLength(len)) => String::with_capacity(len as usize),
        None => String::new(),
    };
    try!(res.read_to_string(&mut body));

    println!("received response {:?} {:?} {:?}", res.status, res.headers, body);

    let res: Response<T> = try!(serde_json::from_str(&body));
    match res {
        Response { stat: Stat::Ok, .. } => {
            Ok(res.result.unwrap())
        },
        Response { stat: Stat::Fail, ..} => {
            Err(Error::Fail { message: res.message.unwrap(), code: res.code.unwrap() })
        },
    }
}

fn authenticate<'a>(
    client: &'a Client, http_method: HttpMethod, endpoint: Endpoint, method: Method,
    credentials: Option<&Credentials>) -> RequestBuilder<'a> {

    let url = format!("{}?method={}", endpoint.to_string(), method.to_string());
    let mut url = Url::parse(&url).unwrap();
    if let Some(credentials) = credentials {
        url.query_pairs_mut().extend_pairs(credentials.query_pairs());
    }

    client.request(http_method, url)
}

fn authenticate_body(body: Option<Value>, credentials: Option<&Credentials>) -> Value {

    let mut body = match body {
        Some(body) => body,
        None => serde_json::to_value("{}"),
    };

    if let Some(credentials) = credentials {
        if let Some(body_map) = body.as_object_mut() {
            if let Some(sync_time) = credentials.sync_time() {
                body_map.insert("syncTime".to_owned(), Value::U64(sync_time.clone()));
            }
            if let Some(user_auth_token) = credentials.user_auth_token() {
                body_map.insert("userAuthToken".to_owned(), Value::String(user_auth_token.to_owned()));
            }
        }
    }

    println!("BODY: {}", body);
    body
}
