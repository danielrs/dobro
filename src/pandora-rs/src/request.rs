use super::Endpoint;
use method::Method;
use response::{Stat, Response};
use error::{Error, Result};

use std::io::Read;

use serde::Deserialize;
use serde_json;
use serde_json::value::Value;

use hyper::client::Client;
use hyper::header::ContentLength;
use hyper::method::Method as HttpMethod;

pub fn request<T>(client: &Client,
                  http_method: HttpMethod,
                  endpoint: &Endpoint,
                  method: Method,
                  body: Option<String>) -> Result<T> where T: Deserialize {
    let url = format!("{}?method={}", endpoint.to_string(), method.to_string());
    let builder = client.request(http_method, &url);
    let mut res = try!(match body {
        Some(ref body) => builder.body(body).send(),
        _ => builder.send(),
    });
    let mut body = match res.headers.clone().get::<ContentLength>() {
        Some(&ContentLength(len)) => String::with_capacity(len as usize),
        None => String::new(),
    };
    try!(res.read_to_string(&mut body));

    debug!("received response {:?} {:?} {:?}", res.status, res.headers, body);

    let res: Response<T> = try!(serde_json::from_str(&body));

    match res {
        Response { stat: Stat::Ok, .. } => Ok(res.result.unwrap()),
        Response { stat: Stat::Fail, ..} => Err(Error::Fail { message: res.message.unwrap(), code: res.code.unwrap() }),
    }
}
