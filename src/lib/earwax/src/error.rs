use ffi::EarwaxErrorCode;

use std::error::Error as StdError;
use std::ffi::NulError;

/// Specialized result.
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Nul(NulError),
    FFI(EarwaxErrorCode),
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Nul(ref e) => e.description(),
            Error::FFI(_) => "FFI error.",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Nul(ref e) => Some(e),
            _ => None,
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<NulError> for Error {
    fn from(error: NulError) -> Error {
        Error::Nul(error)
    }
}
