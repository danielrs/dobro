use super::{Endpoint, Credentials};
use error::Result;
use method::Method;
use request::request;

use hyper::client::Client;
use hyper::method::Method as HttpMethod;

use serde_json;

/// Partner login request information.
#[derive(Serialize)]
struct PartnerLoginRequest {
    username: String,
    password: String,
    #[serde(rename="deviceModel")]
    device_model: String,
    version: String,
}

impl PartnerLoginRequest {
    pub fn new(username: String, password: String, device_model: String) -> Self {
        PartnerLoginRequest {
            username: username,
            password: password,
            device_model: device_model,
            version: "5".to_owned(),
        }
    }

    pub fn new_android() -> Self {
        Self::new(
            "android".to_owned(),
            "AC7IBG09A3DTSYM4R41UJWL07VLN8JI7".to_owned(),
            "android-generic".to_owned(),
        )
    }

    pub fn new_iphone() -> Self {
        Self::new(
            "iphone".to_owned(),
            "P2E4FC0EAD3*878N92B2CDp34I0B1@388137C".to_owned(),
            "IP0".to_owned(),
        )
    }
}

/// Partner login.
#[derive(Debug, Deserialize)]
pub struct PartnerLogin {
    #[serde(rename="partnerAuthToken")]
    pub partner_auth_token: String,
    #[serde(rename="partnerId")]
    pub partner_id: String,
    #[serde(rename="syncTime")]
    pub sync_time: String,
}

/// User login information.
#[derive(Serialize)]
pub struct UserLoginRequest {
    #[serde(rename="loginType")]
    login_type: String,
    username: String,
    password: String,
    #[serde(rename="partnerAuthToken")]
    partner_auth_token: String,
    #[serde(rename="syncTime")]
    sync_time: String,
}

#[derive(Debug, Deserialize)]
pub struct UserLogin {
    #[serde(rename="userAuthToken")]
    user_auth_token: String,
}

impl UserLoginRequest {
    pub fn new(username: String, password: String, partner: PartnerLogin) -> Self {
        UserLoginRequest {
            login_type: "user".to_owned(),
            username: username,
            password: password,
            partner_auth_token: partner.partner_auth_token,
            sync_time: partner.sync_time,
        }
    }
}

// /// User login information.
// struct UserLogin<'a>

pub fn login(endpoint: Endpoint, user: &str, password: &str) -> Result<UserLogin> {
    let client = Client::new();
    let body = try!(serde_json::to_string(&PartnerLoginRequest::new_android()));
    let partner = try!(request(
            &client,
            HttpMethod::Post,
            &endpoint,
            Method::AuthPartnerLogin,
            Some(body)));
    let body = try!(serde_json::to_string(&UserLoginRequest::new(user.to_owned(), password.to_owned(), partner)));
    request(
        &client,
        HttpMethod::Post,
        &endpoint,
        Method::AuthUserLogin,
        Some(body)
    )
}
