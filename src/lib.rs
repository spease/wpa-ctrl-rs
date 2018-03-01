#![deny(missing_docs)]
//! A pure-Rust lowlevel library for controlling wpasupplicant remotely
//!
//! Note that in order to connect to wpasupplicant, you may need
//! elevated permissions (eg run as root)
//!
//! # Example
//!
//! ```
//! let mut wpa = wpactrl::WpaCtrl::new().open().unwrap();
//! println!("{}", wpa.request("LIST_NETWORKS").unwrap());
//! ```
//!
//! The library currently only supports UNIX sockets, but additional
//! connection methods (eg UDP or pipes) may be added in the future.
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate nix;

mod wpactrl;
pub use wpactrl::{WpaCtrl, WpaCtrlAttached, WpaCtrlBuilder};

use failure::Error;
/// Result type used for the library
pub type Result<T> = ::std::result::Result<T, Error>;
