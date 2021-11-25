#![deny(missing_docs)]

use std::{io, str};

/// Error type used for some library functions
#[derive(Debug)]
pub enum WpaError {
    /// Represents all cases of `std::io::Error`.
    Io(io::Error),

    /// Represents all cases of `nix::Error`.
    Nix(nix::Error),

    /// Represents a failure to interpret a sequence of u8 as a string slice.
    Utf8ToStr(str::Utf8Error),

    /// Represents a failed `ATTACH` request to wpasupplicant.
    Attach,

    /// Represents a failed `DETACH` request to wpasupplicant.
    Detach,
}

impl std::error::Error for WpaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            WpaError::Attach => None,
            WpaError::Detach => None,
            WpaError::Io(_) => None,
            WpaError::Nix(_) => None,
            WpaError::Utf8ToStr(_) => None,
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
            WpaError::Io(ref err) => {
                write!(f, "Failed to execute the specified command: {}", err)
            }
            WpaError::Nix(ref err) => {
                write!(f, "Failed to execute the specified command: {}", err)
            }
            WpaError::Utf8ToStr(ref err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for WpaError {
    fn from(err: std::io::Error) -> WpaError {
        WpaError::Io(err)
    }
}

impl From<nix::Error> for WpaError {
    fn from(err: nix::Error) -> WpaError {
        WpaError::Nix(err)
    }
}

impl From<str::Utf8Error> for WpaError {
    fn from(err: str::Utf8Error) -> WpaError {
        WpaError::Utf8ToStr(err)
    }
}
