/// Struct for deserializing Pandora API responses.

#[derive(Debug, Serialize, Deserialize)]
pub enum Stat {
    #[serde(rename="ok")]
    Ok,
    #[serde(rename="fail")]
    Fail,
}

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
