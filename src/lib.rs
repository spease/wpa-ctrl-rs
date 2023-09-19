#![deny(missing_docs)]
#![deny(dead_code)]
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
/// enables syncronous operation of this crate
#[cfg(feature = "sync")]
pub mod sync;
pub use crate::wpactrl::{Client, ClientAttached, ClientBuilder};
use async_trait::async_trait;

pub use crate::error::Error;

/// A `Result` alias where the `Err` case is `wpactrl::Error`
pub type Result<T> = ::std::result::Result<T, Error>;

/// this trait is a way to abstract implementation between ClientAttached and Client
#[async_trait]
pub trait WPAClient {
    /// Send a command to `wpa_supplicant` / `hostapd`.
    ///
    /// Commands are generally identical to those used in `wpa_cli`,
    /// except all uppercase (eg `LIST_NETWORKS`, `SCAN`, etc)
    ///
    /// # Examples
    ///
    /// ```
    /// let mut wpa = wpactrl::Client::builder().open().unwrap();
    /// assert_eq!(wpa.request("PING").unwrap(), "PONG\n");
    /// ```
    ///
    /// # Errors
    ///
    /// * [`Error::Io`] - Low-level I/O error
    /// * [`Error::Utf8ToStr`] - Corrupted message or message with non-UTF8 characters
    /// * [`Error::Wait`] - Failed to wait on underlying Unix socket
    async fn request(&mut self, cmd: &str) -> Result<String>;
}