//! Most of the structs in this module are only used for generating
//! the required [Credentials](struct.Credentials.html) needed to
//! start using [Pandora](../struct.Pandora.html).

use super::{DEFAULT_ENDPOINT};
use crypt::{decrypt};
use error::Result;
use method::Method;
use request::request;

use hyper::client::Client;
use hyper::method::Method as HttpMethod;

use serde_json;

/// Authentication details used in each request. Remember that Pandora puts
/// an expiration date on a set of credentials, so they need to be
/// created again regularly.
///
/// Most fields are optional due to the way Pandora API works, they have an
/// authentication process with 2 phases:
///
/// 1. Login device (a.k.a. Partner).
/// 2. Login user.
///
/// Some http requests to the API use the partial credentials created in step
/// one to get the full credentials.
#[derive(Debug)]
pub struct Credentials {
    // Username and password
    username: String,
    password: String,

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
    /// Creates new credentials from the given user and password.
    pub fn new(username: &str, password: &str) -> Result<Self> {
        let client = Client::new();
        let partner = Partner::default();
        let mut credentials = Credentials {
            username: username.to_owned(),
            password: password.to_owned(),

            encrypt_key: partner.encrypt_password.clone(),
            decrypt_key: partner.decrypt_password.clone(),

            partner_id: None,
            partner_auth_token: None,
            sync_time: None,

            user_id: None,
            user_auth_token: None,
        };

        let partner_login : PartnerLogin = try!(request(
            &client,
            &HttpMethod::Post,
            DEFAULT_ENDPOINT,
            Method::AuthPartnerLogin,
            Some(serde_json::to_value(&partner)),
            None,
        ));
        credentials.set_partner_login(partner_login);

        let user_login_body = serde_json::to_value(
            &UserLoginRequest::new(username.to_owned(), password.to_owned())
        );
        let user_login : UserLogin = try!(request(
            &client,
            &HttpMethod::Post,
            DEFAULT_ENDPOINT,
            Method::AuthUserLogin,
            Some(user_login_body),
            Some(&credentials),
        ));
        credentials.set_user_login(user_login);

        // At this point we can assume credentials are correct.
        Ok(credentials)
    }

    /// Refreshes the expiration time of the credentials.
    pub fn refresh(&mut self) -> Result<()> {
        match Credentials::new(&self.username, &self.password) {
            Ok(new_credentials) => {
                *self = new_credentials;
                Ok(())
            },
            Err(e) => {
                Err(e)
            },
        }
    }

    /// Returns a reference to the username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns a reference to the password.
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Returns a reference to the encryption key.
    pub fn encrypt_key(&self) -> &str {
        &self.encrypt_key
    }

    /// Returns a reference to the decryption key.
    pub fn decrypt_key(&self) -> &str {
        &self.decrypt_key
    }

    /// Returns a reference to the partner id.
    pub fn partner_id<'a>(&'a self) -> Option<&'a str> {
        match self.partner_id {
            Some(ref partner_id) => Some(partner_id.as_str()),
            None => None
        }
    }

    /// Returns a reference to the partner authorization token.
    pub fn partner_auth_token<'a>(&'a self) -> Option<&'a str> {
        match self.partner_auth_token {
            Some(ref partner_auth_token) => Some(partner_auth_token.as_str()),
            None => None
        }
    }

    /// Returns a reference to the synchronization time.
    pub fn sync_time<'a>(&'a self) -> Option<&'a u64> {
        match self.sync_time {
            Some(ref sync_time) => Some(&sync_time),
            None => None
        }
    }

    /// Returns a reference to the user id.
    pub fn user_id<'a>(&'a self) -> Option<&'a str> {
        match self.user_id {
            Some(ref user_id) => Some(user_id.as_str()),
            None => None
        }
    }

    /// Returns a reference to the user authorization token.
    pub fn user_auth_token<'a>(&'a self) -> Option<&'a str> {
        match self.user_auth_token {
            Some(ref user_auth_token) => Some(user_auth_token.as_str()),
            None => None
        }
    }

    /// Consumes PartnerLogin and sets the required information
    /// in the credentials.
    fn set_partner_login(&mut self, partner_login: PartnerLogin) {
        use std::str;
        use std::os::unix::ffi::OsStrExt;

        let sync_time_bytes: Vec<u8> = decrypt(self.decrypt_key(), &partner_login.sync_time)
            .as_os_str().as_bytes().iter().skip(4).cloned().collect();
        let sync_time_str = str::from_utf8(&sync_time_bytes).unwrap_or("0");
        let sync_time = sync_time_str.parse::<u64>().unwrap_or(0);

        self.partner_id = Some(partner_login.partner_id.clone());
        self.partner_auth_token = Some(partner_login.partner_auth_token.clone());
        self.sync_time = Some(sync_time);
    }

    /// Consumes UserLogin and sets the required information
    /// in the credentials.
    fn set_user_login(&mut self, user_login: UserLogin) {
        self.user_id = user_login.user_id.clone();
        self.user_auth_token = Some(user_login.user_auth_token.clone());
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

/// User login.
#[derive(Debug, Deserialize)]
pub struct UserLogin {
    #[serde(rename="userId")]
    pub user_id: Option<String>,
    #[serde(rename="userAuthToken")]
    pub user_auth_token: String,
}

////////////////////
// Request structs
////////////////////

/// User login information.
#[derive(Serialize)]
struct UserLoginRequest {
    #[serde(rename="loginType")]
    login_type: String,
    username: String,
    password: String,
}

impl UserLoginRequest {
    pub fn new(username: String, password: String) -> Self {
        UserLoginRequest {
            login_type: "user".to_owned(),
            username: username,
            password: password,
        }
    }
}
