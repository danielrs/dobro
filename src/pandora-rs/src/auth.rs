use super::Endpoint;
use crypt::{decrypt};
use error::Result;
use method::Method;
use request::{request, request_with_credentials};

use hyper::client::Client;
use hyper::method::Method as HttpMethod;

use serde_json;
use serde_json::value::Value;

/// The authentication details.
pub struct Credentials {
    // Holds partner and encrypt / decrypt key
    pub partner: Partner,
    // Auth information in query string
    pub auth_token: Option<String>,
    pub partner_id: Option<String>,
    pub user_id: Option<String>,
    // Auth information in body
    pub sync_time: Option<String>,
    pub user_auth_token: Option<String>,
}

impl Default for Credentials {
    fn default() -> Credentials {
        Credentials {
            partner: Partner::default(),
            auth_token: None,
            partner_id: None,
            user_id: None,
            sync_time: None,
            user_auth_token: None,
        }
    }
}

impl Credentials {
    pub fn new() -> Self {
        Credentials::default()
    }

    pub fn with_partner(partner: Partner) -> Self {
        Credentials {
            partner: partner,
            auth_token: None,
            partner_id: None,
            user_id: None,
            sync_time: None,
            user_auth_token: None,
        }

    }

    // pub fn set_partner_login(&mut self, partner_login: PartnerLogin) {
    //     self.auth_token: Some(partner_login.partner_auth_token),
    //     self.partner_id: Some(partner_login.partner_id);
    //     self.sync_time: Some(
    // }

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
pub struct Partner {
    username: String,
    password: String,
    #[serde(rename="deviceModel")]
    device_model: String,
    version: String,
    #[serde(rename="encryptPassword")]
    encrypt_password: String,
    #[serde(rename="decryptPassword")]
    decrypt_password: String,
}

impl Default for Partner {
    fn default() -> Self {
        Partner {
            username: "android".to_owned(),
            password: "AC7IBG09A3DTSYM4R41UJWL07VLN8JI7".to_owned(),
            device_model: "android-generic".to_owned(),
            version: "5".to_owned(),
            encrypt_password: "6#26FRL$ZWD".to_owned(),
            decrypt_password: "R=U!LH$O2B#".to_owned(),
        }
    }
}

impl Partner {
    pub fn new(username: String, password: String, device_model: String, version: String,
               encrypt_password: String, decrypt_password: String) -> Self {
        Partner {
            username: username,
            password: password,
            device_model: device_model,
            version: version,
            encrypt_password: encrypt_password,
            decrypt_password: decrypt_password,
        }
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

pub fn login(endpoint: Endpoint, user: &str, password: &str) -> Result<PartnerLogin> {

    unimplemented!()
    // let client = Client::new();
    // let body = serde_json::to_value(&Partner::default());
    // let partner : PartnerLogin = try!(request(
    //         &client,
    //         HttpMethod::Post,
    //         endpoint,
    //         Method::AuthPartnerLogin,
    //         Some(body)));
    // println!("USING: {}", Partner::default().decrypt_password);
    // println!("{} : {:?}", partner.sync_time, decrypt(&Partner::default().decrypt_password, &partner.sync_time));
    // Ok(partner)

    // let body = serde_json::to_value(
    //     &UserLoginRequest::new(user.to_owned(), password.to_owned(), &partner)
    // );
    // let creds = Credentials::with_partner(partner);

    // request_with_credentials(
    //     &client,
    //     HttpMethod::Post,
    //     endpoint,
    //     Method::AuthUserLogin,
    //     Some(body),
    //     &creds,
    // )
}
