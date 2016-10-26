/// Enumerated errors for this API

use std::error::Error as StdError;
use hyper::error::Error as HttpError;
use serde_json::error::Error as CodecError;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Codec(CodecError),
    Http(HttpError),
    Fail { message: String, code: u32 },
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Codec(ref e) => e.description(),
            Error::Http(ref e) => e.description(),
            Error::Fail { ref message, .. } => message.as_str(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Codec(ref e) => Some(e),
            Error::Http(ref e) => Some(e),
            _ => None,
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<CodecError> for Error {
    fn from(error: CodecError) -> Error {
        Error::Codec(error)
    }
}

impl From<HttpError> for Error {
    fn from(error: HttpError) -> Error {
        Error::Http(error)
    }
}
