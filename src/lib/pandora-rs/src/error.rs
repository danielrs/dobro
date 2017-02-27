//! Enumerated errors for this API.

use std::error::Error as StdError;
use std::io::Error as IoError;

use hyper::error::Error as HttpError;

use serde_json::error::Error as CodecError;

/// Specialized result.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Composite error type for the library.
#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Codec(CodecError),
    Http(HttpError),
    Api { message: String, code: ApiErrorCode },
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref e) => e.description(),
            Error::Codec(ref e) => e.description(),
            Error::Http(ref e) => e.description(),
            Error::Api { ref message, .. } => message.as_str(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Io(ref e) => Some(e),
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

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::Io(error)
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

/// Pandora error codes.
#[derive(Debug)]
pub enum ApiErrorCode {
    Unknown,

    InternalError,
    MaintenanceMode,

    UrlParamMissingMethod,
    UrlParamMissingAuthToken,
    UrlParamMissingPartnerId,
    UrlParamMissingUserId,

    SecureProtocolRequired,
    CertifiateRequired,

    ParameterTypeMismatch,
    ParameterMissing,
    ParameterValueInvalid,

    ApiVersionNotSupported,
    LicensingRestrictions,
    InsufficientConnectivity,

    UnknownMethodName,
    WrongProtocol,

    ReadOnlyMode,
    InvalidAuthToken,
    InvalidPartnerOrUserLogin,
    ListenerNotAuthorized,
    UserNotAuthorized,

    MaxStationsReached,
    StationDoesNotExists,

    ComplimentaryPeriodAlreadyInUse,
    CallNotAllowed,
    DeviceNotFound,
    PartnerNotAuthroized,

    InvalidUsername,
    InvalidPassword,
    UsernameAlreadyExists,

    DeviceAlreadyAssociatedToAccount,
    UpgradeDeviceModelInvalid,

    ExplicitPinIncorrect,
    ExplicitPinMalformed,

    DeviceModelInvalid,

    ZipCodeInvalid,
    BirthYearInvalid,
    BirthYearTooYoung,
    InvalidCountryCode,
    InvalidGender,
    DeviceDisabled,

    DailyTrialLimitReached,
    InvalidSponsor,
    UserAlreadyUserTrial,

    PlaylistExceeded,
}

impl From<u32> for ApiErrorCode {
    fn from(code: u32) -> Self {
        match code {
            0 => ApiErrorCode::InternalError,
            1 => ApiErrorCode::MaintenanceMode,
            2 => ApiErrorCode::UrlParamMissingMethod,
            3 => ApiErrorCode::UrlParamMissingAuthToken,
            4 => ApiErrorCode::UrlParamMissingPartnerId,
            5 => ApiErrorCode::UrlParamMissingUserId,
            6 => ApiErrorCode::SecureProtocolRequired,
            7 => ApiErrorCode::CertifiateRequired,
            8 => ApiErrorCode::ParameterTypeMismatch,
            9 => ApiErrorCode::ParameterMissing,
            10 => ApiErrorCode::ParameterValueInvalid,
            11 => ApiErrorCode::ApiVersionNotSupported,
            12 => ApiErrorCode::LicensingRestrictions,
            13 => ApiErrorCode::InsufficientConnectivity,
            14 => ApiErrorCode::UnknownMethodName,
            15 => ApiErrorCode::WrongProtocol,
            1000 => ApiErrorCode::ReadOnlyMode,
            1001 => ApiErrorCode::InvalidAuthToken,
            1002 => ApiErrorCode::InvalidPartnerOrUserLogin,
            1003 => ApiErrorCode::ListenerNotAuthorized,
            1004 => ApiErrorCode::UserNotAuthorized,
            1005 => ApiErrorCode::MaxStationsReached,
            1006 => ApiErrorCode::StationDoesNotExists,
            1007 => ApiErrorCode::ComplimentaryPeriodAlreadyInUse,
            1008 => ApiErrorCode::CallNotAllowed,
            1009 => ApiErrorCode::DeviceNotFound,
            1010 => ApiErrorCode::PartnerNotAuthroized,
            1011 => ApiErrorCode::InvalidUsername,
            1012 => ApiErrorCode::InvalidPassword,
            1013 => ApiErrorCode::UsernameAlreadyExists,
            1014 => ApiErrorCode::DeviceAlreadyAssociatedToAccount,
            1015 => ApiErrorCode::UpgradeDeviceModelInvalid,
            1018 => ApiErrorCode::ExplicitPinIncorrect,
            1020 => ApiErrorCode::ExplicitPinMalformed,
            1023 => ApiErrorCode::DeviceModelInvalid,
            1024 => ApiErrorCode::ZipCodeInvalid,
            1025 => ApiErrorCode::BirthYearInvalid,
            1026 => ApiErrorCode::BirthYearTooYoung,
            1027 => ApiErrorCode::InvalidCountryCode,
            // TODO: Maybe not 1028, the code was 1027 (Repeated).
            1028 => ApiErrorCode::InvalidGender,
            1034 => ApiErrorCode::DeviceDisabled,
            1035 => ApiErrorCode::DailyTrialLimitReached,
            1036 => ApiErrorCode::InvalidSponsor,
            1037 => ApiErrorCode::UserAlreadyUserTrial,
            1039 => ApiErrorCode::PlaylistExceeded,

            _ => ApiErrorCode::Unknown,
        }
    }
}
