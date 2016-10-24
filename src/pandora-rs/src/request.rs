use method::Method;
use serde_json;
use serde_json::value::Value;

#[derive(Debug)]
pub struct Endpoint {
    pub url: &'static str,
    pub is_https: bool,
}

const endpoints : [Endpoint; 4] = [
    Endpoint {
        url: "http://tuner.pandora.com/services/json/",
        is_https: false,
    },
    Endpoint {
        url: "https://tuner.pandora.com/services/json/",
        is_https: true,
    },
    Endpoint {
        url: "http://internal-tuner.pandora.com/services/json/",
        is_https: false,
    },
    Endpoint {
        url: "https://internal-tuner.pandora.com/services/json/",
        is_https: true,
    },
];
const default_endpoint : Endpoint = endpoints[0];

#[derive(Debug)]
pub struct PandoraRequest {
    endpoint: Endpoint,
    method: Method,
    auth_token: Option<String>,
    partner_id: Option<String>,
    user_id: Option<String>,
    body: Value,
}

impl PandoraRequest {
    pub fn new(method: Method) -> Self {
        PandoraRequest {
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
}

impl PandoraRequest {
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
