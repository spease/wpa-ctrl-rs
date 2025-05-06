#![deny(missing_docs)]
use alloc::collections::VecDeque;
use core::convert::Into;
use core::str::from_utf8;
use core::sync::atomic::{AtomicUsize, Ordering};
use log::{debug, warn};
use std::io::ErrorKind;
use std::os::unix::io::{AsRawFd as _, RawFd};
use std::path::{Path, PathBuf};
use std::process;
use super::Result;
use tokio::fs;
use tokio::net::UnixDatagram;

use crate::WPAClient;
use crate::error::Error;

/// Maximum message size
const BUF_SIZE: usize = 10_240;
/// Counter to avoid using the same file when creating multiple clients.
static COUNTER: AtomicUsize = AtomicUsize::new(0);
/// Default path where local end of socket is created
const PATH_DEFAULT_CLIENT: &str = "/tmp";
/// Default path to search for socket
const PATH_DEFAULT_SERVER: &str = "/var/run/wpa_supplicant/wlan0";

/// A connection to `wpa_supplicant` / `hostapd`
pub struct Client(ClientInternal);

impl Client {
    /// Register as an event monitor for control interface messages
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let mut wpa = wpactrl::Client::builder().open().await?;
    /// let wpa_attached = wpa.attach().await?;
    /// # Ok::<_,wpactrl::Error>(())
    /// # })?;
    /// # Ok::<_,wpactrl::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// * [`Error::Attach`] - Unexpected (non-OK) response
    /// * [`Error::Io`] - Low-level I/O error
    /// * [`Error::Utf8ToStr`] - Corrupted message or message with non-UTF8 characters
    /// * [`Error::Wait`] - Failed to wait on underlying Unix socket
    #[inline]
    pub async fn attach(mut self) -> Result<ClientAttached> {
        // FIXME: None closure would be better
        if self.0.request("ATTACH", |_: &str| ()).await? == "OK\n" {
            Ok(ClientAttached(self.0, VecDeque::new()))
        } else {
            Err(Error::Attach)
        }
    }

    /// Creates a builder for a `wpa_supplicant` / `hostapd` connection
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let wpa = wpactrl::Client::builder().open().await?;
    /// # Ok::<_,wpactrl::Error>(())
    /// # })?;
    /// # Ok::<_,wpactrl::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }
}

impl WPAClient for Client {
    #[inline]
    async fn request(&mut self, cmd: &str) -> Result<String> {
        self.0.request(cmd, |_: &str| ()).await
    }
}

/// A connection to `wpa_supplicant` / `hostapd` that receives status messages
pub struct ClientAttached(ClientInternal, VecDeque<String>);

impl ClientAttached {

    /// Stop listening for and discard any remaining control interface messages
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let mut wpa = wpactrl::Client::builder().open().await?.attach().await?;
    /// wpa.detach().await?;
    /// # Ok::<_,wpactrl::Error>(())
    /// # })?;
    /// # Ok::<_,wpactrl::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// * [`Error::Detach`] - Unexpected (non-OK) response
    /// * [`Error::Io`] - Low-level I/O error
    /// * [`Error::Utf8ToStr`] - Corrupted message or message with non-UTF8 characters
    /// * [`Error::Wait`] - Failed to wait on underlying Unix socket
    #[inline]
    pub async fn detach(mut self) -> Result<Client> {
        
        if self.0.request("DETACH", |_: &str| ()).await? == "OK\n" {
            Ok(Client(self.0))
        } else {
            Err(Error::Detach)
        }
    }

    /// Receive the next control interface message.
    ///
    /// Note that multiple control interface messages can be pending;
    /// call this function repeatedly until it returns None to get all of them.
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let mut wpa = wpactrl::Client::builder().open().await?.attach().await?;
    /// assert_eq!(wpa.recv().await?, None);
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
    #[inline]
    pub async fn recv(&mut self) -> Result<Option<String>> {
        if let Some(stored_reply) = self.1.pop_back() {
            Ok(Some(stored_reply))
        } else {
            self.0.recv().await
        }
    }
}

impl WPAClient for ClientAttached {
    #[inline]
    async fn request(&mut self, cmd: &str) -> Result<String> {
        let mut messages = VecDeque::new();
        let reply = self.0.request(cmd, |msg: &str| messages.push_front(msg.into())).await;
        self.1.extend(messages);
        reply
    }
}

/// Builder object used to construct a [`Client`] session
#[derive(Default)]
pub struct ClientBuilder {
    /// Local client path
    cli_path: Option<PathBuf>,
    /// Remote `wpa_supplicant` path
    ctrl_path: Option<PathBuf>,
}

impl ClientBuilder {
    /// A path-like object for this application's UNIX domain socket
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use wpactrl::Client;
    /// let wpa = Client::builder()
    ///             .cli_path("/tmp")
    ///             .open()
    ///             .await?;
    /// # Ok::<_,wpactrl::Error>(())
    /// # })?;
    /// # Ok::<_,wpactrl::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn cli_path<I, P>(mut self, cli_path: I) -> Self
    where
        I: Into<Option<P>>,
        P: AsRef<Path> + Sized,
        PathBuf: From<P>,
    {
        self.cli_path = cli_path.into().map(PathBuf::from);
        self
    }

    /// A path-like object for the `wpa_supplicant` / `hostapd` UNIX domain sockets
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use wpactrl::Client;
    /// let wpa = Client::builder()
    ///             .ctrl_path("/var/run/wpa_supplicant/wlan0")
    ///             .open()
    ///             .await?;
    /// # Ok::<_,wpactrl::Error>(())
    /// # })?;
    /// # Ok::<_,wpactrl::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn ctrl_path<I, P>(mut self, ctrl_path: I) -> Self
    where
        I: Into<Option<P>>,
        P: AsRef<Path> + Sized,
        PathBuf: From<P>,
    {
        self.ctrl_path = ctrl_path.into().map(PathBuf::from);
        self
    }

    /// Open a control interface to `wpa_supplicant` / `hostapd`.
    ///
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use wpactrl::Client;
    /// let wpa = Client::builder().open().await?;
    /// # Ok::<_,wpactrl::Error>(())
    /// # })?;
    /// # Ok::<_,wpactrl::Error>(())
    /// ```
    /// # Errors
    ///
    /// * [[`Error::Io`]] - Low-level I/O error
    #[inline]
    pub async fn open(self) -> Result<Client> {
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut tries = 0_usize;
        loop {
            tries = tries.saturating_add(1);
            let bind_filename = format!("wpa_ctrl_{}-{}", process::id(), counter);
            let bind_filepath = self
                .cli_path
                .as_deref()
                .unwrap_or_else(|| Path::new(PATH_DEFAULT_CLIENT))
                .join(bind_filename);
            match UnixDatagram::bind(&bind_filepath) {
                Ok(socket) => {
                    socket.connect(self.ctrl_path.unwrap_or_else(|| PATH_DEFAULT_SERVER.into()))?;
                    return Ok(Client(ClientInternal {
                        buffer: [0; BUF_SIZE],
                        handle: socket,
                        filepath: bind_filepath,
                    }));
                }
                Err(ref err) if tries < 2 && err.kind() == ErrorKind::AddrInUse => {
                    fs::remove_file(bind_filepath).await?;
                }
                Err(err) => return Err(err.into()),
            }
        }
    }
}

/// Contains common code for `Client` and `ClientAttached`
struct ClientInternal {
    /// Buffer for incoming messages
    buffer: [u8; BUF_SIZE],
    /// Path of unix socket
    filepath: PathBuf,
    /// Handle for unix socket
    handle: UnixDatagram,
}

impl ClientInternal {
    /// Check if any messages are available
    pub fn pending(&self) -> Result<bool> {
        select(self.handle.as_raw_fd(), 0)
    }

    /// Receive a message
    pub (crate) async fn recv(&mut self) -> Result<Option<String>> {
        if self.pending()? {
            let buf_len = self.handle.recv(&mut self.buffer).await?;
                self.buffer
                    .get(0..buf_len)
                    .ok_or(Error::UnexpectedMessageSize(buf_len))
                    .and_then(|msg|from_utf8(msg).map_err(Into::into))
                    .map(|reply| Some(reply.to_owned()))
        } else {
            Ok(None)
        }
    }

    /// Send a command to `wpa_supplicant` / `hostapd`.
    pub (crate) async fn request<F: FnMut(&str)>(&mut self, cmd: &str, mut cb: F) -> Result<String> {
        debug!("SEND: {cmd:?}");
        self.handle.send(cmd.as_bytes()).await?;
        loop {
            select(self.handle.as_raw_fd(), 10)?;
            match self.handle.recv(&mut self.buffer).await {
                Ok(len) => {
                    let reply = self.buffer
                        .get(0..len)
                        .ok_or(Error::UnexpectedMessageSize(len))
                        .and_then(|msg|from_utf8(msg).map_err(Into::into))?;
                    if reply.starts_with('<') {
                        cb(reply);
                    } else {
                        return Ok(reply.to_owned());
                    }
                }
                Err(ref err) if err.kind() == ErrorKind::Interrupted => {},
                Err(err) => return Err(err.into()),
            }
        }
    }
}

impl Drop for ClientInternal {
    fn drop(&mut self) {
        #[expect(clippy::absolute_paths, reason = "No way to use tokio fs")]
        if let Err(err) = std::fs::remove_file(&self.filepath) {
            warn!("Unable to unlink {err:?}");
        }
    }
}

/// Checks if `fd` is ready.
#[expect(clippy::multiple_unsafe_ops_per_block, reason = "more readable")]
fn select(fd: RawFd, seconds: i64) -> Result<bool> {
    use core::mem::MaybeUninit;
    use core::ptr;
    use std::io;

    // SAFETY:
    // This is using the C API
    let res = unsafe {
        let mut raw_fd_set = {
            let mut raw_fd_set = MaybeUninit::<libc::fd_set>::uninit();
            libc::FD_ZERO(raw_fd_set.as_mut_ptr());
            raw_fd_set.assume_init()
        };
        libc::FD_SET(fd, &mut raw_fd_set);
        libc::select(
            fd
                .checked_add(1)
                .ok_or_else(|| io::Error::new(ErrorKind::Other, "RawFd overflow"))?,
            &mut raw_fd_set,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut libc::timeval {
                tv_sec: seconds,
                tv_usec: 0,
            },
        )
    };

    if res >= 0 {
        Ok(res > 0)
    } else {
        // SAFETY:
        // errno will always be there for us
        Err(Error::Wait(unsafe { *libc::__errno_location() }))
    }
}

#[cfg(test)]
#[expect(clippy::panic_in_result_fn, reason = "test")]
mod test {
    use core::time::Duration;
    use std::sync::LazyLock;
    use super::*;
    use tokio::sync::Mutex;
    use tokio::time;

    static SERIAL: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);

    #[expect(clippy::large_stack_frames, reason = "test")]
    #[tokio::test]
    async fn attach() -> Result<()> {
        let _lock = SERIAL.lock().await;
        #[expect(clippy::large_futures, reason = "test")]
        wpa_ctrl()
            .await?
            .attach()
            .await?
            .detach()
            .await?
            .attach()
            .await?
            .detach()
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn builder() -> Result<()> {
        let _lock = SERIAL.lock().await;
        wpa_ctrl().await?;
        Ok(())
    }

    #[tokio::test]
    async fn detach() -> Result<()> {
        let _lock = SERIAL.lock().await;
        #[expect(clippy::large_futures, reason = "test")]
        let wpa = wpa_ctrl().await?.attach().await?;
        #[expect(clippy::large_futures, reason = "test")]
        wpa.detach().await?;
        Ok(())
    }

    #[tokio::test]
    async fn recv() -> Result<()> {
        let _lock = SERIAL.lock().await;
        #[expect(clippy::large_futures, reason = "test")]
        let mut wpa = wpa_ctrl().await?.attach().await?;
        assert_eq!(wpa.recv().await?, None);
        assert_eq!(wpa.request("SCAN").await?, "OK\n");
        loop {
            match wpa.recv().await? {
                Some(reply) => {
                    assert_eq!(reply.get(3..), Some("CTRL-EVENT-SCAN-STARTED "));
                    break;
                }
                None => time::sleep(Duration::from_millis(10)).await,
            }
        }
        #[expect(clippy::large_futures, reason = "test")]
        wpa.detach().await?;
        Ok(())
    }

    #[tokio::test]
    async fn request() -> Result<()> {
        let _lock = SERIAL.lock().await;
        let mut wpa = wpa_ctrl().await?;
        assert_eq!(wpa.request("PING").await?, "PONG\n");
        #[expect(clippy::large_futures, reason = "test")]
        let mut wpa_attached = wpa.attach().await?;
        // FIXME: This may not trigger the callback
        assert_eq!(wpa_attached.request("PING").await?, "PONG\n");
        Ok(())
    }

    async fn wpa_ctrl() -> Result<Client> {
        Client::builder().open().await
    }
}
