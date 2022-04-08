#![deny(missing_docs)]

use std::{io, str};

/// The errors that may occur using `wpactrl`
#[derive(Debug)]
pub enum Error {
    /// Represents all cases of `std::io::Error`.
    Io(io::Error),

    /// Represents a failure to interpret a sequence of u8 as a string slice.
    Utf8ToStr(str::Utf8Error),

    /// Represents a failed `ATTACH` request to wpasupplicant.
    Attach,

    /// Represents a failed `DETACH` request to wpasupplicant.
    Detach,

    /// Error waiting for a response
    Wait
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::Attach|Self::Detach|Self::Wait => None,
            Self::Io(ref source) => Some(source),
            Self::Utf8ToStr(ref source) => Some(source),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Attach => {
                write!(f, "Failed to attach to wpasupplicant")
            }
            Self::Detach => {
                write!(f, "Failed to detach from wpasupplicant")
            }
            Self::Wait => {
                write!(f, "Unable to wait for response from wpasupplicant")
            }
            Self::Io(ref err) => {
                write!(f, "Failed to execute the specified command: {}", err)
            }
            Self::Utf8ToStr(ref err) => {
                write!(f, "Failed to parse UTF8 to string: {}", err)
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Self {
        Self::Utf8ToStr(err)
    }
}
