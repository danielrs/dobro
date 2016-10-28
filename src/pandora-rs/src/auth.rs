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
#[derive(Debug)]
pub struct Credentials {
    // Encryption / Decryption information.
    encrypt_key: String,
    decrypt_key: String,
    // Partner info.
    partner_id: Option<String>,
    partner_auth_token: Option<String>,
    sync_time: Option<u64>,
    // User info.
    user_id: Option<String>,
    user_auth_token: Option<String>,
}

impl Credentials {
    pub fn new(partner: &Partner) -> Self {
        Credentials {
            encrypt_key: partner.encrypt_password.clone(),
            decrypt_key: partner.decrypt_password.clone(),

            partner_id: None,
            partner_auth_token: None,
            sync_time: None,

            user_id: None,
            user_auth_token: None,
        }
    }

    /// Consumes PartnerLogin and sets the required information
    /// in the credentials.
    pub fn set_partner_login(&mut self, partner_login: &PartnerLogin) {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let sync_time_bytes: Vec<u8> = decrypt(self.decrypt_key(), &partner_login.sync_time)
            .as_os_str().as_bytes().iter().skip(4).cloned().collect();
        let sync_time_string = OsStr::from_bytes(&sync_time_bytes)
            .to_owned().into_string().unwrap_or("0".to_owned());
        let sync_time = sync_time_string.parse::<u64>().unwrap_or(0);


        self.partner_id = Some(partner_login.partner_id.clone());
        self.partner_auth_token = Some(partner_login.partner_auth_token.clone());
        self.sync_time = Some(sync_time);
    }

    /// Consumes UserLogin and sets the required information
    /// in the credentials.
    pub fn set_user_login(&mut self, user_login: &UserLogin) {
        self.user_id = user_login.user_id.clone();
        self.user_auth_token = Some(user_login.user_auth_token.clone());
    }

    /// Returns the encryption key.
    pub fn encrypt_key(&self) -> &str {
        &self.encrypt_key
    }

    /// Returns the decryption key.
    pub fn decrypt_key(&self) -> &str {
        &self.decrypt_key
    }


    /// Returns a Vector of query pairs from the credentials.
    pub fn query_pairs<'a>(&'a self) -> Vec<(&'a str, &'a str)> {
        let mut pairs = Vec::new();

        if let Some(ref partner_auth_token) = self.partner_auth_token {
            pairs.push(("auth_token", partner_auth_token.as_str()));
        }

        if let Some(ref user_auth_token) = self.user_auth_token {
            pairs.push(("auth_token", user_auth_token.as_str()));
        }

        if let Some(ref partner_id) = self.partner_id {
            pairs.push(("partner_id", partner_id.as_str()));
        }

        if let Some(ref user_id) = self.user_id {
            pairs.push(("user_id", user_id.as_str()));
        }

        pairs
    }

    pub fn sync_time<'a>(&'a self) -> Option<&'a u64> {
        match self.sync_time {
            Some(ref sync_time) => Some(&sync_time),
            None => None
        }
    }

    pub fn user_auth_token<'a>(&'a self) -> Option<&'a str> {
        match self.user_auth_token {
            Some(ref user_auth_token) => Some(user_auth_token.as_str()),
            None => None
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
    #[serde(rename="partnerId")]
    pub partner_id: String,
    #[serde(rename="partnerAuthToken")]
    pub partner_auth_token: String,
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
    #[serde(rename="userId")]
    pub user_id: Option<String>,
    #[serde(rename="userAuthToken")]
    pub user_auth_token: String,
}

pub fn login(endpoint: Endpoint, username: &str, password: &str) -> Result<Credentials> {

    let client = Client::new();
    let partner = Partner::default();
    let mut credentials = Credentials::new(&partner);

    let partner_login : PartnerLogin = try!(request(
        &client,
        HttpMethod::Post,
        endpoint,
        Method::AuthPartnerLogin,
        Some(serde_json::to_value(&partner)),
    ));
    credentials.set_partner_login(&partner_login);

    let user_login_body = serde_json::to_value(
        &UserLoginRequest::new(username.to_owned(), password.to_owned(), &partner_login)
    );
    let user_login : UserLogin = try!(request_with_credentials(
        &client,
        HttpMethod::Post,
        endpoint,
        Method::AuthUserLogin,
        Some(user_login_body),
        Some(&credentials),
    ));
    credentials.set_user_login(&user_login);

    Ok(credentials)
}
