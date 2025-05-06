#![deny(missing_docs)]

use core::{error, fmt, str};
use core::ffi::c_int;
use core::fmt::{Display, Formatter};
use std::io;

/// The errors that may occur using `wpactrl`
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Represents a failed `ATTACH` request to wpasupplicant.
    Attach,

    /// Represents a failed `DETACH` request to wpasupplicant.
    Detach,

    /// Represents all cases of `std::io::Error`.
    Io(io::Error),

    /// Represents an invalid buffer length reported
    UnexpectedMessageSize(usize),

    /// Represents a failure to interpret a sequence of u8 as a string slice.
    Utf8ToStr(str::Utf8Error),

    /// Error waiting for a response
    Wait(c_int)
}

impl Display for Error {
    #[inline]
    #[expect(clippy::min_ident_chars, reason = "f is default var")]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Self::Attach => {
                write!(f, "Failed to attach to wpasupplicant")
            }
            Self::Detach => {
                write!(f, "Failed to detach from wpasupplicant")
            }
            Self::Io(ref err) => {
                write!(f, "Failed to execute the specified command: {err}")
            }
            Self::UnexpectedMessageSize(size) => {
                write!(f, "Unexpected message size {size}")
            }
            Self::Utf8ToStr(ref err) => {
                write!(f, "Failed to parse UTF8 to string: {err}")
            }
            Self::Wait(err) => {
                write!(f, "Unable to wait for response from wpasupplicant: {err}")
            }
        }
    }
}

impl From<io::Error> for Error {
    #[inline]
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<str::Utf8Error> for Error {
    #[inline]
    fn from(err: str::Utf8Error) -> Self {
        Self::Utf8ToStr(err)
    }
}

impl error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Self::Attach|Self::Detach|Self::UnexpectedMessageSize(_)|Self::Wait(_) => None,
            Self::Io(ref source) => Some(source),
            Self::Utf8ToStr(ref source) => Some(source),
        }
    }
}
