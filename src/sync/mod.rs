use crate::Result;
use crate::{ClientBuilder};
use futures::executor::block_on;
/// A connection to `wpa_supplicant` / `hostapd`
pub struct Client(crate::Client);

impl Client {
    /// Creates a builder for a `wpa_supplicant` / `hostapd` connection
    ///
    /// # Examples
    ///
    /// ```
    /// let wpa = wpactrl::Client::builder().open().unwrap();
    /// ```
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Register as an event monitor for control interface messages
    ///
    /// # Examples
    ///
    /// ```
    /// let mut wpa = wpactrl::Client::builder().open().unwrap();
    /// let wpa_attached = wpa.attach().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// * [`Error::Attach`] - Unexpected (non-OK) response
    /// * [`Error::Io`] - Low-level I/O error
    /// * [`Error::Utf8ToStr`] - Corrupted message or message with non-UTF8 characters
    /// * [`Error::Wait`] - Failed to wait on underlying Unix socket
    pub fn attach(self) -> Result<ClientAttached> {
        Ok(ClientAttached(block_on(self.0.attach())?))
    }

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
    pub fn request(&mut self, cmd: &str) -> Result<String> {
        block_on(self.0.request(cmd))
    }
}

/// A connection to `wpa_supplicant` / `hostapd` that receives status messages
pub struct ClientAttached(crate::ClientAttached);

impl ClientAttached {
    /// Stop listening for and discard any remaining control interface messages
    ///
    /// # Examples
    ///
    /// ```
    /// let mut wpa = wpactrl::Client::builder().open().unwrap().attach().unwrap();
    /// wpa.detach().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// * [`Error::Detach`] - Unexpected (non-OK) response
    /// * [`Error::Io`] - Low-level I/O error
    /// * [`Error::Utf8ToStr`] - Corrupted message or message with non-UTF8 characters
    /// * [`Error::Wait`] - Failed to wait on underlying Unix socket
    pub fn detach(self) -> Result<Client> {
        Ok(Client(block_on(self.0.detach())?))
    }

    /// Receive the next control interface message.
    ///
    /// Note that multiple control interface messages can be pending;
    /// call this function repeatedly until it returns None to get all of them.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut wpa = wpactrl::Client::builder().open().unwrap().attach().unwrap();
    /// assert_eq!(wpa.recv().unwrap(), None);
    /// ```
    ///
    /// # Errors
    ///
    /// * [`Error::Io`] - Low-level I/O error
    /// * [`Error::Utf8ToStr`] - Corrupted message or message with non-UTF8 characters
    /// * [`Error::Wait`] - Failed to wait on underlying Unix socket
    pub fn recv(&mut self) -> Result<Option<String>> {
        block_on(self.0.recv())
    }

    /// Send a command to `wpa_supplicant` / `hostapd`.
    ///
    /// Commands are generally identical to those used in `wpa_cli`,
    /// except all uppercase (eg `LIST_NETWORKS`, `SCAN`, etc)
    ///
    /// Control interface messages will be buffered as the command
    /// runs, and will be returned on the next call to recv.
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
    pub fn request(&mut self, cmd: &str) -> Result<String> {
        block_on(self.0.request(cmd))
    }
}