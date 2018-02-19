use failure::Error;
use std::ffi::CString;
use std::{mem, ptr};
use std::path::Path;
use std::os::unix::ffi::OsStrExt;
use std;

use libc::{c_char, c_int, c_void, size_t};

pub struct WpaCtrl {
    handle: *mut c_void,
}

#[derive(Debug, Fail, PartialEq)]
enum WpaError {
    #[fail(display = "an error occurred")]
    Failure,
    #[fail(display = "failed to create interface")]
    Interface,
    #[fail(display = "timed out")]
    Timeout,
    #[fail(display = "unknown error {}", _0)]
    Unknown(c_int),
}

type Result<T> = std::result::Result<T, Error>;

#[link(name = "wpactrl", kind = "static")]
extern "C" {
    fn wpa_ctrl_open(ctrl_path: *const c_char) -> *mut c_void;
    fn wpa_ctrl_open2(ctrl_path: *const c_char, cli_pth: *const c_char) -> *mut c_void;
    fn wpa_ctrl_request(
        ctrl: *mut c_void,
        cmd: *const c_char,
        cmd_len: size_t,
        reply: *mut c_char,
        reply_len: *mut size_t,
        msg_cb: Option<unsafe extern "C" fn(msg: *mut c_char, len: size_t)>,
    ) -> c_int;
    fn wpa_ctrl_close(ctrl: *mut c_void);
    fn wpa_ctrl_attach(ctrl: *mut c_void) -> c_int;
    fn wpa_ctrl_detach(ctrl: *mut c_void) -> c_int;
    fn wpa_ctrl_pending(ctrl: *mut c_void) -> c_int;
    fn wpa_ctrl_recv(ctrl: *mut c_void, reply: *mut c_char, len: *mut size_t) -> c_int;
}

fn wrap_cb<F: Fn(Result<&str>)>(f: Option<F>) -> Option<unsafe extern "C" fn(*mut c_char, size_t)> {
    match f {
        Some(_) => {
            unsafe extern "C" fn wrapped<F: Fn(Result<&str>)>(msg: *mut c_char, len: size_t) {
                let s = std::str::from_utf8(std::slice::from_raw_parts(msg as *const u8, len))
                    .map_err(Error::from);
                mem::zeroed::<F>()(s);
            }
            Some(wrapped::<F>)
        }
        None => None,
    }
}

impl WpaCtrl {
    pub fn new<P: AsRef<Path>>(ctrl_path: P) -> Result<WpaCtrl> {
        unsafe {
            let handle =
                wpa_ctrl_open(CString::new(ctrl_path.as_ref().as_os_str().as_bytes())?.as_ptr());
            if handle == ptr::null_mut() {
                Err(WpaError::Interface)?;
            }
            Ok(WpaCtrl { handle })
        }
    }

    pub fn new2<P1: AsRef<Path>, P2: AsRef<Path>>(
        ctrl_path: P1,
        cli_path: P2,
    ) -> Result<WpaCtrl> {
        unsafe {
            let handle = wpa_ctrl_open2(
                CString::new(ctrl_path.as_ref().as_os_str().as_bytes())?.as_ptr(),
                CString::new(cli_path.as_ref().as_os_str().as_bytes())?.as_ptr(),
            );
            if handle == ptr::null_mut() {
                Err(WpaError::Interface)?;
            }
            Ok(WpaCtrl { handle })
        }
    }

    pub fn request(&self, cmd: &str, cb: Option<fn(Result<&str>)>) -> Result<String> {
        let mut res_len: size_t = 10240;
        let mut res = Vec::with_capacity(10240);
        let c_cmd = CString::new(cmd)?;
        let c_cmd_len = c_cmd.as_bytes().len();

        match unsafe {
            wpa_ctrl_request(
                self.handle,
                c_cmd.as_ptr(),
                c_cmd_len,
                res.as_mut_ptr() as *mut c_char,
                &mut res_len,
                wrap_cb(cb),
            )
        } {
            0 => {
                unsafe {
                    res.set_len(res_len);
                }
                Ok(String::from_utf8(res)?)
            }
            -1 => Err(WpaError::Failure.into()),
            -2 => Err(WpaError::Timeout.into()),
            x => Err(WpaError::Unknown(x).into()),
        }
    }

    pub fn attach(&self) -> Result<()> {
        match unsafe { wpa_ctrl_attach(self.handle) } {
            0 => Ok(()),
            -1 => Err(WpaError::Failure.into()),
            -2 => Err(WpaError::Timeout.into()),
            x => Err(WpaError::Unknown(x).into()),
        }
    }

    pub fn detach(&self) -> Result<()> {
        match unsafe { wpa_ctrl_detach(self.handle) } {
            0 => Ok(()),
            -1 => Err(WpaError::Failure.into()),
            -2 => Err(WpaError::Timeout.into()),
            x => Err(WpaError::Unknown(x).into()),
        }
    }

    pub fn pending(&self) -> Result<bool> {
        match unsafe { wpa_ctrl_pending(self.handle) } {
            0 => Ok(false),
            1 => Ok(true),
            -1 => Err(WpaError::Failure.into()),
            x => Err(WpaError::Unknown(x).into()),
        }
    }

    pub fn recv(&self) -> Result<String> {
        let mut res_len: size_t = 10240;
        let mut res = Vec::with_capacity(res_len);
        match unsafe { wpa_ctrl_recv(self.handle, res.as_mut_ptr() as *mut c_char, &mut res_len) } {
            0 => {
                unsafe {
                    res.set_len(res_len);
                }
                Ok(String::from_utf8(res)?)
            }
            -1 => Err(WpaError::Failure.into()),
            x => Err(WpaError::Unknown(x).into()),
        }
    }
}

impl Drop for WpaCtrl {
    fn drop(&mut self) {
        unsafe {
            wpa_ctrl_close(self.handle);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_err<T: std::fmt::Debug>(r: Result<T>, e2: WpaError) {
        assert_eq!(r.unwrap_err().downcast::<WpaError>().unwrap(), e2);
    }

    fn wpa_ctrl() -> WpaCtrl {
        WpaCtrl::new("/run/wpa_supplicant/wlan0").unwrap()
    }

    #[test]
    fn attach() {
        let wpa = wpa_ctrl();
        wpa.attach().unwrap();
        wpa.detach().unwrap();
        assert_err(wpa.detach(), WpaError::Failure);
        wpa.attach().unwrap();
        wpa.attach().unwrap();
        wpa.detach().unwrap();
        wpa.detach().unwrap();
        assert_err(wpa.detach(), WpaError::Failure);
    }

    #[test]
    fn detach() {
        let wpa = wpa_ctrl();
        assert_err(wpa.detach(), WpaError::Failure);
        wpa.attach().unwrap();
        wpa.detach().unwrap();
    }

    #[test]
    fn new() {
        wpa_ctrl();
    }

    #[test]
    fn request() {
        let wpa = wpa_ctrl();
        assert_eq!(wpa.request("PING", None).unwrap(), "PONG\n");
        // FIXME: This may not trigger the callback
        assert_eq!(wpa_ctrl().request("PING", Some(|s|println!("CB: {:?}", s.unwrap()))).unwrap(), "PONG\n");
    }

    #[test]
    fn pending() {
        let wpa = wpa_ctrl();
        wpa.attach().unwrap();
        assert_eq!(wpa.pending().unwrap(), false);
        wpa.detach().unwrap();
    }

    #[test]
    fn recv() {
        let wpa = wpa_ctrl();
        wpa.attach().unwrap();
        assert_err(wpa.recv(), WpaError::Failure);
        assert_eq!(wpa.request("SCAN", None).unwrap(), "OK\n");
        while !wpa.pending().unwrap() {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(&wpa.recv().unwrap()[3..], "CTRL-EVENT-SCAN-STARTED ");
        wpa.detach().unwrap();
    }
}
