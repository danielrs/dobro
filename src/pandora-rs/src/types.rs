use serde_json;

pub trait Json {
    fn json(&self) -> String;
}

pub enum Response<T> {
    Ok(T),
    Fail { message: String, code: u32 },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PartnerLogin {
    pub username: String,
    pub password: String,
    #[serde(rename="deviceModel")]
    pub device_model: String,
    pub version: String,
    #[serde(rename="includeUrls")]
    pub include_urls: bool,
}

impl Json for PartnerLogin {
    fn json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
