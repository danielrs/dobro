/// Struct for deserializing Pandora API responses.

/// Enum for checking Pandora API responses of success (ok) or error (fail).
#[derive(Debug, Serialize, Deserialize)]
pub enum Stat {
    #[serde(rename="ok")]
    Ok,
    #[serde(rename="fail")]
    Fail,
}

/// Type for deserializing a Pandora API reponse.
#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub stat: Stat,
    pub result: Option<T>,
    pub message: Option<String>,
    pub code: Option<u32>,
}

impl<T> Response<T> {
    pub fn from_result(result: T) -> Self {
        Response {
            stat: Stat::Ok,
            result: Some(result),
            message: None,
            code: None
        }
    }

    pub fn from_error(message: String, code: u32) -> Self {
        Response {
            stat: Stat::Fail,
            result: None,
            message: Some(message),
            code: Some(code),
        }
    }
}
