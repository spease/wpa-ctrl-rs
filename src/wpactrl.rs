#![deny(missing_docs)]
use super::Result;
use log::warn;
use std::collections::VecDeque;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crate::error::Error;

const BUF_SIZE: usize = 10_240;
const PATH_DEFAULT_CLIENT: &str = "/tmp";
const PATH_DEFAULT_SERVER: &str = "/var/run/wpa_supplicant/wlan0";

// Counter to avoid using the same file when creating multiple clients.
static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Builder object used to construct a [`Client`] session
#[derive(Default)]
pub struct ClientBuilder {
    cli_path: Option<PathBuf>,
    ctrl_path: Option<PathBuf>,
}

impl ClientBuilder {
    /// A path-like object for this application's UNIX domain socket
    ///
    /// # Examples
    ///
    /// ```
    /// use wpactrl::Client;
    /// let wpa = Client::builder()
    ///             .cli_path("/tmp")
    ///             .open()
    ///             .unwrap();
    /// ```
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
    /// use wpactrl::Client;
    /// let wpa = Client::builder()
    ///             .ctrl_path("/var/run/wpa_supplicant/wlan0")
    ///             .open()
    ///             .unwrap();
    /// ```
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
    /// use wpactrl::Client;
    /// let wpa = Client::builder().open().unwrap();
    /// ```
    /// # Errors
    ///
    /// * [[`Error::Io`]] - Low-level I/O error
    pub fn open(self) -> Result<Client> {
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut tries = 0;
        loop {
            tries += 1;
            let bind_filename = format!("wpa_ctrl_{}-{}", std::process::id(), counter);
            let bind_filepath = self
                .cli_path
                .as_deref()
                .unwrap_or_else(|| Path::new(PATH_DEFAULT_CLIENT))
                .join(bind_filename);
            match UnixDatagram::bind(&bind_filepath) {
                Ok(socket) => {
                    socket.connect(self.ctrl_path.unwrap_or_else(|| PATH_DEFAULT_SERVER.into()))?;
                    socket.set_nonblocking(true)?;
                    return Ok(Client(ClientInternal {
                        buffer: [0; BUF_SIZE],
                        handle: socket,
                        filepath: bind_filepath,
                    }));
                }
                Err(ref e) if tries < 2 && e.kind() == std::io::ErrorKind::AddrInUse => {
                    std::fs::remove_file(bind_filepath)?;
                    continue;
                }
                Err(e) => return Err(e.into()),
            };
        }
    }
}

struct ClientInternal {
    buffer: [u8; BUF_SIZE],
    handle: UnixDatagram,
    filepath: PathBuf,
}

fn select(fd: RawFd, duration: Duration) -> Result<bool> {
    let r = unsafe {
        let mut raw_fd_set = {
            let mut raw_fd_set = std::mem::MaybeUninit::<libc::fd_set>::uninit();
            libc::FD_ZERO(raw_fd_set.as_mut_ptr());
            raw_fd_set.assume_init()
        };
        libc::FD_SET(fd, &mut raw_fd_set);
        libc::select(
            fd + 1,
            &mut raw_fd_set,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut libc::timeval {
                tv_sec: duration.as_secs().try_into().unwrap(),
                tv_usec: duration.subsec_micros().try_into().unwrap(),
            },
        )
    };

    if r >= 0 {
        Ok(r > 0)
    } else {
        Err(Error::Wait)
    }
}

impl ClientInternal {
    /// Check if any messages are available
    pub fn pending(&mut self) -> Result<bool> {
        select(self.handle.as_raw_fd(), Duration::from_secs(0))
    }

    /// Receive a message
    pub fn recv(&mut self) -> Result<Option<String>> {
        if self.pending()? {
            let buf_len = self.handle.recv(&mut self.buffer)?;
            std::str::from_utf8(&self.buffer[0..buf_len])
                .map(|s| Some(s.to_owned()))
                .map_err(std::convert::Into::into)
        } else {
            Ok(None)
        }
    }

    /// Send a command to `wpa_supplicant` / `hostapd`.
    fn request<F: FnMut(&str)>(&mut self, cmd: &str, mut cb: F) -> Result<String> {
        self.handle.send(cmd.as_bytes())?;
        loop {
            select(self.handle.as_raw_fd(), Duration::from_secs(10))?;
            match self.handle.recv(&mut self.buffer) {
                Ok(len) => {
                    let s = std::str::from_utf8(&self.buffer[0..len])?;
                    if s.starts_with('<') {
                        cb(s);
                    } else {
                        return Ok(s.to_owned());
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }
    }
}

impl Drop for ClientInternal {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.filepath) {
            warn!("Unable to unlink {:?}", e);
        }
    }
}

/// A connection to `wpa_supplicant` / `hostapd`
pub struct Client(ClientInternal);

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
    pub fn attach(mut self) -> Result<ClientAttached> {
        // FIXME: None closure would be better
        if self.0.request("ATTACH", |_: &str| ())? == "OK\n" {
            Ok(ClientAttached(self.0, VecDeque::new()))
        } else {
            Err(Error::Attach)
        }
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
        self.0.request(cmd, |_: &str| ())
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
    pub fn detach(mut self) -> Result<Client> {
        if self.0.request("DETACH", |_: &str| ())? == "OK\n" {
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
        if let Some(s) = self.1.pop_back() {
            Ok(Some(s))
        } else {
            self.0.recv()
        }
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
        let mut messages = VecDeque::new();
        let r = self.0.request(cmd, |s: &str| messages.push_front(s.into()));
        self.1.extend(messages);
        r
    }
}

#[cfg(test)]
mod test {
    use serial_test::serial;
    use super::*;

    fn wpa_ctrl() -> Client {
        Client::builder().open().unwrap()
    }

    #[test]
    #[serial]
    fn attach() {
        wpa_ctrl()
            .attach()
            .unwrap()
            .detach()
            .unwrap()
            .attach()
            .unwrap()
            .detach()
            .unwrap();
    }

    #[test]
    #[serial]
    fn detach() {
        let wpa = wpa_ctrl().attach().unwrap();
        wpa.detach().unwrap();
    }

    #[test]
    #[serial]
    fn builder() {
        wpa_ctrl();
    }

    #[test]
    #[serial]
    fn request() {
        let mut wpa = wpa_ctrl();
        assert_eq!(wpa.request("PING").unwrap(), "PONG\n");
        let mut wpa_attached = wpa.attach().unwrap();
        // FIXME: This may not trigger the callback
        assert_eq!(wpa_attached.request("PING").unwrap(), "PONG\n");
    }

    #[test]
    #[serial]
    fn recv() {
        let mut wpa = wpa_ctrl().attach().unwrap();
        assert_eq!(wpa.recv().unwrap(), None);
        assert_eq!(wpa.request("SCAN").unwrap(), "OK\n");
        loop {
            match wpa.recv().unwrap() {
                Some(s) => {
                    assert_eq!(&s[3..], "CTRL-EVENT-SCAN-STARTED ");
                    break;
                }
                None => std::thread::sleep(std::time::Duration::from_millis(10)),
            }
        }
        wpa.detach().unwrap();
    }
}
