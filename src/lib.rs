#![deny(missing_docs)]
#![deny(dead_code)]
#![deny(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic)]
#![allow(clippy::error_impl_error, clippy::implicit_return, clippy::missing_trait_methods, clippy::question_mark_used, clippy::ref_patterns, clippy::separated_literal_suffix)]
#![deny(clippy::absolute_paths, clippy::doc_markdown, clippy::exhaustive_enums, clippy::indexing_slicing, clippy::min_ident_chars, clippy::missing_docs_in_private_items, clippy::missing_inline_in_public_items, clippy::mod_module_files, clippy::multiple_unsafe_ops_per_block, clippy::panic_in_result_fn, clippy::std_instead_of_core, clippy::undocumented_unsafe_blocks, clippy::unseparated_literal_suffix, clippy::unused_trait_names, clippy::unwrap_used)]
// #![deny(arbitrary_source_item_ordering)]
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

extern crate alloc;

/// Error-handling
mod error;

/// enables syncronous operation of this crate
#[cfg(feature = "sync")]
pub mod sync;

/// Primary implementation
mod wpactrl;

use core::future::Future;

pub use crate::{
    error::Error,
    wpactrl::{Client, ClientAttached, ClientBuilder}
};

/// A `Result` alias where the `Err` case is `wpactrl::Error`
#[expect(clippy::absolute_paths, reason = "make certain core is used")]
pub type Result<T> = ::core::result::Result<T, Error>;

/// this trait is a way to abstract implementation between `ClientAttached` and `Client`
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
    fn request(&mut self, cmd: &str) -> impl Future<Output = Result<String>>;
}
