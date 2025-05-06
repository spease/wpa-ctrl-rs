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
//! # tokio_test::block_on(async {
//! use wpactrl::WPAClient;
//!
//! let mut wpa = wpactrl::Client::builder().open().await?;
//! println!("{}", wpa.request("LIST_NETWORKS").await?);
//! # Ok::<_,wpactrl::Error>(())
//! # })?;
//! # Ok::<_,wpactrl::Error>(())
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

pub use crate::error::Error;

/// A `Result` alias where the `Err` case is `wpactrl::Error`
pub type Result<T> = ::std::result::Result<T, Error>;

/// this trait is a way to abstract implementation between ClientAttached and Client
pub trait WPAClient {
    /// Send a command to `wpa_supplicant` / `hostapd`.
    ///
    /// Commands are generally identical to those used in `wpa_cli`,
    /// except all uppercase (eg `LIST_NETWORKS`, `SCAN`, etc)
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use wpactrl::WPAClient;
    ///
    /// let mut wpa = wpactrl::Client::builder().open().await?;
    /// assert_eq!(wpa.request("PING").await?, "PONG\n");
    /// # Ok::<_,wpactrl::Error>(())
    /// # })?;
    /// # Ok::<_,wpactrl::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// * [`Error::Io`] - Low-level I/O error
    /// * [`Error::Utf8ToStr`] - Corrupted message or message with non-UTF8 characters
    /// * [`Error::Wait`] - Failed to wait on underlying Unix socket
    fn request(&mut self, cmd: &str) -> impl std::future::Future<Output = Result<String>>;
}
