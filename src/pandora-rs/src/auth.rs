use super::Endpoint;
use error::Result;
use method::Method;
use request::{request, request_with_credentials};

use hyper::client::Client;
use hyper::method::Method as HttpMethod;

use serde_json;
use serde_json::value::Value;

/// The authentication details.
pub struct Credentials {
    // Goes on query
    pub auth_token: Option<String>,
    pub partner_id: Option<String>,
    pub user_id: Option<String>,
    // Goes in request body
    pub sync_time: Option<String>,
    pub user_auth_token: Option<String>,
}

impl Default for Credentials {
    fn default() -> Credentials {
        Credentials {
            auth_token: None,
            partner_id: None,
            user_id: None,
            sync_time: None,
            user_auth_token: None,
        }
    }
}

impl Credentials {
    pub fn with_partner(partner: PartnerLogin) -> Self {
        Credentials {
            auth_token: Some(partner.partner_auth_token),
            partner_id: Some(partner.partner_id),
            sync_time: Some(partner.sync_time),
            .. Credentials::default()
        }
    }

    pub fn with_user(user: UserLogin) -> Self {
        Credentials {
            auth_token: Some(user.user_auth_token.clone()),
            user_auth_token: Some(user.user_auth_token),
            .. Credentials::default()
        }
    }

    pub fn with_user_partner(user: UserLogin, partner: PartnerLogin) -> Self {
        Credentials {
            auth_token: Some(partner.partner_auth_token),
            partner_id: Some(partner.partner_id),
            sync_time: Some(partner.sync_time),
            .. Credentials::with_user(user)
        }
    }
}

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

impl UserLoginRequest {
    pub fn new(username: String, password: String, partner: &PartnerLogin) -> Self {
        UserLoginRequest {
            login_type: "user".to_owned(),
            username: username,
            password: password,
            partner_auth_token: partner.partner_auth_token.clone(),
            sync_time: partner.sync_time.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UserLogin {
    #[serde(rename="userAuthToken")]
    pub user_auth_token: String,
}

pub fn login(endpoint: Endpoint, user: &str, password: &str) -> Result<UserLogin> {
    let client = Client::new();
    let body = serde_json::to_value(&PartnerLoginRequest::new_android());
    let partner = try!(request(
            &client,
            HttpMethod::Post,
            endpoint,
            Method::AuthPartnerLogin,
            Some(body)));

    let body = serde_json::to_value(
        &UserLoginRequest::new(user.to_owned(), password.to_owned(), &partner)
    );
    let creds = Credentials::with_partner(partner);

    request_with_credentials(
        &client,
        HttpMethod::Post,
        endpoint,
        Method::AuthUserLogin,
        Some(body),
        &creds,
    )
}
