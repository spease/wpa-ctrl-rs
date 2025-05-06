use crate::Result;
use crate::WPAClient as _;
use futures::executor::block_on;
use std::path::{Path, PathBuf};

/// A connection to `wpa_supplicant` / `hostapd`
pub struct Client(crate::Client);

impl Client {
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
    #[inline]
    pub fn attach(self) -> Result<ClientAttached> {
        Ok(ClientAttached(block_on(self.0.attach())?))
    }

    /// Creates a builder for a `wpa_supplicant` / `hostapd` connection
    ///
    /// # Examples
    ///
    /// ```
    /// let wpa = wpactrl::Client::builder().open().unwrap();
    /// ```
    #[inline]
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
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
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn request(&mut self, cmd: &str) -> Result<String> {
        block_on(self.0.request(cmd))
    }
}

/// Builder object used to construct a [`Client`] session
#[derive(Default)]
pub struct ClientBuilder(crate::ClientBuilder);

impl ClientBuilder {
    /// A path-like object for this application's UNIX domain socket
    ///
    /// # Examples
    ///
    /// ```
    /// use wpactrl::sync::Client;
    /// let wpa = Client::builder()
    ///             .cli_path("/tmp")
    ///             .open()?;
    /// # Ok::<_,wpactrl::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn cli_path<I, P>(self, cli_path: I) -> Self
    where
        I: Into<Option<P>>,
        P: AsRef<Path> + Sized,
        PathBuf: From<P>,
    {
        Self(self.0.cli_path(cli_path))
    }

    /// A path-like object for the `wpa_supplicant` / `hostapd` UNIX domain sockets
    ///
    /// # Examples
    ///
    /// ```
    /// use wpactrl::sync::Client;
    /// let wpa = Client::builder()
    ///             .ctrl_path("/var/run/wpa_supplicant/wlan0")
    ///             .open()?;
    /// # Ok::<_,wpactrl::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn ctrl_path<I, P>(self, ctrl_path: I) -> Self
    where
        I: Into<Option<P>>,
        P: AsRef<Path> + Sized,
        PathBuf: From<P>,
    {
        Self(self.0.ctrl_path(ctrl_path))
    }

    /// Open a control interface to `wpa_supplicant` / `hostapd`.
    ///
    /// # Examples
    ///
    /// ```
    /// use wpactrl::sync::Client;
    /// let wpa = Client::builder().open()?;
    /// # Ok::<_,wpactrl::Error>(())
    /// ```
    /// # Errors
    ///
    /// * [[`Error::Io`]] - Low-level I/O error
    #[inline]
    pub fn open(self) -> Result<Client> {
        Ok(Client(block_on(self.0.open())?))
    }
}

#[cfg(test)]
#[expect(clippy::panic_in_result_fn, reason = "test")]
mod test {
    use core::time::Duration;
    use super::*;

    use std::sync::{LazyLock, Mutex};
    use std::thread;

    static SERIAL: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);

    #[test]
    fn attach() -> Result<()> {
        #[expect(clippy::unwrap_used, reason = "test")]
        let _lock = SERIAL.lock().unwrap();
        wpa_ctrl()?
            .attach()?
            .detach()?
            .attach()?
            .detach()?;
        Ok(())
    }

    #[test]
    fn builder() -> Result<()> {
        #[expect(clippy::unwrap_used, reason = "test")]
        let _lock = SERIAL.lock().unwrap();
        wpa_ctrl()?;
        Ok(())
    }

    #[test]
    fn detach() -> Result<()> {
        #[expect(clippy::unwrap_used, reason = "test")]
        let _lock = SERIAL.lock().unwrap();
        let wpa = wpa_ctrl()?.attach()?;
        wpa.detach()?;
        Ok(())
    }

    #[test]
    fn recv() -> Result<()> {
        #[expect(clippy::unwrap_used, reason = "test")]
        let _lock = SERIAL.lock().unwrap();
        let mut wpa = wpa_ctrl()?.attach()?;
        assert_eq!(wpa.recv()?, None);
        assert_eq!(wpa.request("SCAN")?, "OK\n");
        loop {
            match wpa.recv()? {
                Some(reply) => {
                    assert_eq!(reply.get(3..), Some("CTRL-EVENT-SCAN-STARTED "));
                    break;
                }
                None => thread::sleep(Duration::from_millis(10)),
            }
        }
        wpa.detach()?;
        Ok(())
    }

    #[test]
    fn request() -> Result<()> {
        #[expect(clippy::unwrap_used, reason = "test")]
        let _lock = SERIAL.lock().unwrap();
        let mut wpa = wpa_ctrl()?;
        assert_eq!(wpa.request("PING")?, "PONG\n");
        let mut wpa_attached = wpa.attach()?;
        // FIXME: This may not trigger the callback
        assert_eq!(wpa_attached.request("PING")?, "PONG\n");
        Ok(())
    }

    fn wpa_ctrl() -> Result<Client> {
        Client::builder().open()
    }
}
