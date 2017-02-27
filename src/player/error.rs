use std::error::Error as StdError;

use ao::error::Error as AoError;
use earwax::error::Error as EarwaxError;
use pandora::error::Error as PandoraError;

/// Composite error type for the player.
#[derive(Debug)]
pub enum Error {
    Ao(AoError),
    Earwax(EarwaxError),
    Pandora(PandoraError),
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Ao(ref e) => e.description(),
            Error::Earwax(ref e) => e.description(),
            Error::Pandora(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Ao(ref e) => Some(e),
            Error::Earwax(ref e) => Some(e),
            Error::Pandora(ref e) => Some(e),
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<AoError> for Error {
    fn from(error: AoError) -> Error {
        Error::Ao(error)
    }
}

impl From<EarwaxError> for Error {
    fn from(error: EarwaxError) -> Error {
        Error::Earwax(error)
    }
}

impl From<PandoraError> for Error {
    fn from(error: PandoraError) -> Error {
        Error::Pandora(error)
    }
}
