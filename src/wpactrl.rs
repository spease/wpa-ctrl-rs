#![deny(missing_docs)]
use super::Result;
use log::warn;
use std::collections::VecDeque;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::WpaError;

const BUF_SIZE: usize = 10_240;
const PATH_DEFAULT_CLIENT: &str = "/tmp";
const PATH_DEFAULT_SERVER: &str = "/var/run/wpa_supplicant/wlan0";

/// Builder object used to construct a `WpaCtrl` session
#[derive(Default)]
pub struct WpaCtrlBuilder {
    cli_path: Option<PathBuf>,
    ctrl_path: Option<PathBuf>,
}

impl WpaCtrlBuilder {
    /// A path-like object for this application's UNIX domain socket
    ///
    /// # Examples
    ///
    /// ```
    /// use wpactrl::WpaCtrl;
    /// let wpa = WpaCtrl::builder()
    ///             .cli_path("/tmp")
    ///             .open()
    ///             .unwrap();
    /// ```
    pub fn cli_path<I, P>(mut self, cli_path: I) -> Self
    where
        I: Into<Option<P>>,
        P: AsRef<Path> + Sized,
        PathBuf: From<P>,
    {
        self.cli_path = cli_path.into().map(PathBuf::from);
        self
    }

    /// A path-like object for the wpasupplicant / hostap UNIX domain sockets
    ///
    /// # Examples
    ///
    /// ```
    /// use wpactrl::WpaCtrl;
    /// let wpa = WpaCtrl::builder()
    ///             .ctrl_path("/var/run/wpa_supplicant/wlan0")
    ///             .open()
    ///             .unwrap();
    /// ```
    pub fn ctrl_path<I, P>(mut self, ctrl_path: I) -> Self
    where
        I: Into<Option<P>>,
        P: AsRef<Path> + Sized,
        PathBuf: From<P>,
    {
        self.ctrl_path = ctrl_path.into().map(PathBuf::from);
        self
    }

    /// Open a control interface to wpasupplicant.
    ///
    /// # Examples
    ///
    /// ```
    /// use wpactrl::WpaCtrl;
    /// let wpa = WpaCtrl::builder().open().unwrap();
    /// ```
    pub fn open(self) -> Result<WpaCtrl> {
        let mut counter = 0;
        loop {
            counter += 1;
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
                    return Ok(WpaCtrl(WpaCtrlInternal {
                        buffer: [0; BUF_SIZE],
                        handle: socket,
                        filepath: bind_filepath,
                    }));
                }
                Err(ref e) if counter < 2 && e.kind() == std::io::ErrorKind::AddrInUse => {
                    std::fs::remove_file(bind_filepath)?;
                    continue;
                }
                Err(e) => return Err(e.into()),
            };
        }
    }
}

struct WpaCtrlInternal {
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
                tv_sec: duration.as_secs() as i64,
                tv_usec: duration.subsec_micros() as i64,
            },
        )
    };

    if r >= 0 {
        Ok(r > 0)
    } else {
        Err(WpaError::Wait)
    }
}

impl WpaCtrlInternal {
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
                .map_err(|e| e.into())
        } else {
            Ok(None)
        }
    }

    /// Send a command to wpasupplicant / hostapd.
    fn request<F: FnMut(&str)>(&mut self, cmd: &str, mut cb: F) -> Result<String> {
        self.handle.send(cmd.as_bytes())?;
        loop {
            select(self.handle.as_raw_fd(), Duration::from_secs(10))?;
            match self.handle.recv(&mut self.buffer) {
                Ok(len) => {
                    let s = std::str::from_utf8(&self.buffer[0..len])?;
                    if s.starts_with('<') {
                        cb(s)
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

impl Drop for WpaCtrlInternal {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.filepath) {
            warn!("Unable to unlink {:?}", e);
        }
    }
}

/// A connection to wpasupplicant / hostap
pub struct WpaCtrl(WpaCtrlInternal);

impl WpaCtrl {
    /// Creates a builder for a wpasupplicant / hostap connection
    ///
    /// # Examples
    ///
    /// ```
    /// let wpa = wpactrl::WpaCtrl::builder().open().unwrap();
    /// ```
    pub fn builder() -> WpaCtrlBuilder {
        WpaCtrlBuilder::default()
    }

    /// Register as an event monitor for control interface messages
    ///
    /// # Examples
    ///
    /// ```
    /// let mut wpa = wpactrl::WpaCtrl::builder().open().unwrap();
    /// let wpa_attached = wpa.attach().unwrap();
    /// ```
    pub fn attach(mut self) -> Result<WpaCtrlAttached> {
        // FIXME: None closure would be better
        if self.0.request("ATTACH", |_: &str| ())? != "OK\n" {
            Err(WpaError::Attach)
        } else {
            Ok(WpaCtrlAttached(self.0, VecDeque::new()))
        }
    }

    /// Send a command to wpa_supplicant/hostapd.
    ///
    /// Commands are generally identical to those used in wpa_cli,
    /// except all uppercase (eg LIST_NETWORKS, SCAN, etc)
    ///
    /// # Examples
    ///
    /// ```
    /// let mut wpa = wpactrl::WpaCtrl::builder().open().unwrap();
    /// assert_eq!(wpa.request("PING").unwrap(), "PONG\n");
    /// ```
    pub fn request(&mut self, cmd: &str) -> Result<String> {
        self.0.request(cmd, |_: &str| ())
    }
}

/// A connection to wpasupplicant / hostap that receives status messages
pub struct WpaCtrlAttached(WpaCtrlInternal, VecDeque<String>);

impl WpaCtrlAttached {
    /// Stop listening for and discard any remaining control interface messages
    ///
    /// # Examples
    ///
    /// ```
    /// let mut wpa = wpactrl::WpaCtrl::builder().open().unwrap().attach().unwrap();
    /// wpa.detach().unwrap();
    /// ```
    pub fn detach(mut self) -> Result<WpaCtrl> {
        if self.0.request("DETACH", |_: &str| ())? != "OK\n" {
            Err(WpaError::Detach)
        } else {
            Ok(WpaCtrl(self.0))
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
    /// let mut wpa = wpactrl::WpaCtrl::builder().open().unwrap().attach().unwrap();
    /// assert_eq!(wpa.recv().unwrap(), None);
    /// ```
    pub fn recv(&mut self) -> Result<Option<String>> {
        if let Some(s) = self.1.pop_back() {
            Ok(Some(s))
        } else {
            self.0.recv()
        }
    }

    /// Send a command to wpa_supplicant/hostapd.
    ///
    /// Commands are generally identical to those used in wpa_cli,
    /// except all uppercase (eg LIST_NETWORKS, SCAN, etc)
    ///
    /// Control interface messages will be buffered as the command
    /// runs, and will be returned on the next call to recv.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut wpa = wpactrl::WpaCtrl::builder().open().unwrap();
    /// assert_eq!(wpa.request("PING").unwrap(), "PONG\n");
    /// ```
    pub fn request(&mut self, cmd: &str) -> Result<String> {
        let mut messages = VecDeque::new();
        let r = self.0.request(cmd, |s: &str| messages.push_front(s.into()));
        self.1.extend(messages);
        r
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn wpa_ctrl() -> WpaCtrl {
        WpaCtrl::builder().open().unwrap()
    }

    #[test]
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
    fn detach() {
        let wpa = wpa_ctrl().attach().unwrap();
        wpa.detach().unwrap();
    }

    #[test]
    fn builder() {
        wpa_ctrl();
    }

    #[test]
    fn request() {
        let mut wpa = wpa_ctrl();
        assert_eq!(wpa.request("PING").unwrap(), "PONG\n");
        let mut wpa_attached = wpa.attach().unwrap();
        // FIXME: This may not trigger the callback
        assert_eq!(wpa_attached.request("PING").unwrap(), "PONG\n");
    }

    #[test]
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
