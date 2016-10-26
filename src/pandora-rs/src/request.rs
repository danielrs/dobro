use method::Method;

use serde_json;
use serde_json::value::Value;

use hyper::client::Client;
use hyper::error::Error;

#[derive(Debug)]
pub enum Response<T> {
    Ok(T),
    Fail { message: String, code: u32 },
}

#[derive(Debug)]
pub struct Request {
    endpoint: Endpoint,
    method: Method,
    auth_token: Option<String>,
    partner_id: Option<String>,
    user_id: Option<String>,
    body: Value,
}

impl Request {
    pub fn new(method: Method) -> Self {
        Request {
            endpoint: default_endpoint,
            method: method,
            auth_token: None,
            partner_id: None,
            user_id: None,
            body: Value::Null,
        }
    }

    pub fn with_endpoint(mut self, endpoint: Endpoint) -> Self {
        self.endpoint = endpoint;
        self
    }

    pub fn with_auth_token(mut self, auth_token: String) -> Self {
        self.auth_token = Some(auth_token);
        self
    }

    pub fn with_partner_id(mut self, partner_id: String) -> Self {
        self.partner_id = Some(partner_id);
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_body(mut self, body: Value) -> Self {
        self.body = body;
        self
    }

    pub fn send<T>(&self) -> Result<Response<T>, Error> {
        use std::io::Read;

        let client = Client::new();
        let mres = client
            .post(&self.get_url())
            .body(&self.get_body())
            .send();

        match mres {
            Ok(mut res) => {
                let mut body = String::new();
                res.read_to_string(&mut body);
                // TODO: Add serde decoding
                unimplemented!()
            },
            Err(err) => Err(err),
        }
    }
}

impl Request {
    pub fn get_url(&self) -> String {
        let mut string = String::new();
        string.push_str(self.endpoint.url);
        string.push_str(&format!("?method={}", self.method.to_string()));
        if let Some(ref auth_token) = self.auth_token {
            string.push_str(&format!("&auth_token={}", auth_token));
        }
        if let Some(ref partner_id) = self.partner_id {
            string.push_str(&format!("&partner_id={}", partner_id));
        }
        if let Some(ref user_id) = self.user_id {
            string.push_str(&format!("&user_id={}", user_id));
        }
        string
    }

    pub fn get_body(&self) -> String {
        serde_json::to_string(&self.body).unwrap()
    }
}
