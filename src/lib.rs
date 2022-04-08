#![deny(missing_docs)]
//! A pure-Rust lowlevel library for controlling wpasupplicant remotely
//!
//! Note that in order to connect to wpasupplicant, you may need
//! elevated permissions (eg run as root)
//!
//! # Example
//!
//! ```
//! let mut wpa = wpactrl::Client::builder().open().unwrap();
//! println!("{}", wpa.request("LIST_NETWORKS").unwrap());
//! ```
//!
//! The library currently only supports UNIX sockets, but additional
//! connection methods (eg UDP or pipes) may be added in the future.

mod error;
mod wpactrl;
pub use crate::wpactrl::{Client, ClientAttached, ClientBuilder};

pub use crate::error::Error;

/// A `Result` alias where the `Err` case is `wpactrl::Error`
pub type Result<T> = ::std::result::Result<T, Error>;
