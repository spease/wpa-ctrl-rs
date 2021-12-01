#![deny(missing_docs)]

use std::{io, str};

/// Error type used for some library functions
#[derive(Debug)]
pub enum WpaError {
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

impl std::error::Error for WpaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            WpaError::Attach => None,
            WpaError::Detach => None,
            WpaError::Wait => None,
            WpaError::Io(ref source) => Some(source),
            WpaError::Utf8ToStr(ref source) => Some(source),
        }
    }
}

impl std::fmt::Display for WpaError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            WpaError::Attach => {
                write!(f, "Failed to attach to wpasupplicant")
            }
            WpaError::Detach => {
                write!(f, "Failed to detach from wpasupplicant")
            }
            WpaError::Wait => {
                write!(f, "Unable to wait for response from wpasupplicant")
            }
            WpaError::Io(ref err) => {
                write!(f, "Failed to execute the specified command: {}", err)
            }
            WpaError::Utf8ToStr(ref err) => {
                write!(f, "Failed to parse UTF8 to string: {}", err)
            }
        }
    }
}

impl From<std::io::Error> for WpaError {
    fn from(err: std::io::Error) -> WpaError {
        WpaError::Io(err)
    }
}

impl From<str::Utf8Error> for WpaError {
    fn from(err: str::Utf8Error) -> WpaError {
        WpaError::Utf8ToStr(err)
    }
}
