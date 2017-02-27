use std::error::Error as StdError;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
    Unknown,

    NoDriver,
    NotFile,
    NotLive,

    BadOption,

    OpenDevice,
    OpenFile,
    FileExists,

    BadFormat,
}

impl Error {
    pub fn from_errno() -> Self {
        match IoError::last_os_error().raw_os_error() {
            Some(e) => Self::from_u32(e as u32),
            None => Error::Unknown,
        }
    }

    pub fn from_u32(code: u32) -> Self {
        match code {
            1 => Error::NoDriver,
            2 => Error::NotFile,
            3 => Error::NotLive,
            4 => Error::BadOption,
            5 => Error::OpenDevice,
            6 => Error::OpenFile,
            7 => Error::FileExists,
            8 => Error::BadFormat,
            _ => Error::Unknown,
        }
    }

    pub fn to_str(&self) -> &str {
        match *self {
            Error::Unknown => "Unknown",

            Error::NoDriver => "No driver",
            Error::NotFile => "No file",
            Error::NotLive => "Not live",

            Error::BadOption => "Bad option",

            Error::OpenDevice => "Can't open device",
            Error::OpenFile => "Cant' open file",
            Error::FileExists => "File exists",

            Error::BadFormat => "Bad format",
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        self.to_str()
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<u32> for Error {
    fn from(code: u32) -> Self {
        Error::from_u32(code)
    }
}
