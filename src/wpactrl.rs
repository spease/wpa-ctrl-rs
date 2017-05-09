use std::ffi::CString;
use std::ptr;
use std::mem;

use libc::{c_void,c_char,c_int,size_t};

pub struct WpaCtrl {
  handle: *mut c_void
}

#[link(name="wpactrl", kind="static")]
extern "C" {
  fn wpa_ctrl_open(ctrl_path: *const c_char) -> *mut c_void;
  fn wpa_ctrl_open2(ctrl_path: *const c_char, cli_pth: *const c_char) -> *mut c_void;
  fn wpa_ctrl_request(ctrl: *mut c_void, cmd: *const c_char, cmd_len: size_t, reply: *mut c_char, reply_len: *mut size_t, msg_cb: Option<unsafe extern fn(msg: *mut c_char, len: size_t)>) -> c_int;
  fn wpa_ctrl_close(ctrl: *mut c_void);
  fn wpa_ctrl_attach(ctrl: *mut c_void) -> c_int;
  fn wpa_ctrl_detach(ctrl: *mut c_void) -> c_int;
  fn wpa_ctrl_pending(ctrl: *mut c_void) -> c_int;
  fn wpa_ctrl_recv(ctrl: *mut c_void, reply: *mut c_char, len: *mut size_t) -> c_int;
}

macro_rules! cstr {
  ($e:expr) => (CString::new($e.as_bytes().to_vec()).unwrap().as_ptr());
}

fn wrap_cb<F: Fn(&String)>(f: Option<F>) -> Option<unsafe extern fn(*mut c_char, size_t)> {
  assert!(mem::size_of::<F>() == 0);
  match f {
    Some(_) => {
      unsafe extern fn wrapped<F: Fn(&String)>(msg: *mut c_char, len: size_t) {
        let str = &String::from_raw_parts(msg as *mut u8, len, len);
        mem::zeroed::<F>()(str);
      }
      Some(wrapped::<F>)
    },
    None => None
  }

}

impl WpaCtrl {
  pub fn new(ctrl_path: &String) -> Result<WpaCtrl, String> {
    unsafe {
      let handle = wpa_ctrl_open(cstr!(ctrl_path));
      if handle == ptr::null_mut() {
        return Err("failed to create interface".to_string());
      }
      Ok(WpaCtrl{handle})
    }
  }

  pub fn new2(ctrl_path: &String, cli_path: &String) -> Result<WpaCtrl, String> {
    unsafe {
      let handle = wpa_ctrl_open2(cstr!(ctrl_path), cstr!(cli_path));
      if handle == ptr::null_mut() {
        return Err("failed to create interface".to_string());
      }
      Ok(WpaCtrl{handle})
    }
  }

  pub fn request(&self, cmd: &String, cb: Option<fn(&String)>) -> Result<String, String> {
    unsafe {
      let mut res = Vec::<u8>::with_capacity(500);
      let mut res_len = 500;
      let cmd_len = cmd.len();
      let c_cmd = CString::new(cmd.as_bytes().to_vec()).unwrap();

      match wpa_ctrl_request(self.handle, c_cmd.as_ptr(), cmd_len,
                             res.as_mut_ptr() as *mut c_char, &mut res_len,
                             wrap_cb(cb)) {
        0 => {
          res.set_len(res_len as usize);
          Ok(String::from_utf8(res).unwrap())
        },
        -1 => Err("an error occurred".to_string()),
        -2 => Err("timed out".to_string()),
        _ => Err("unknown error".to_string())
      }
    }
  }

  pub fn attach(&self) -> Result<(), String> {
    unsafe {
      match wpa_ctrl_attach(self.handle) {
        0 => Ok(()),
        -1 => Err("an error occurred".to_string()),
        -2 => Err("timed out".to_string()),
        _ => Err("unknown error".to_string())
      }
    }
  }

  pub fn detach(&self) -> Result<(), String> {
    unsafe {
      match wpa_ctrl_detach(self.handle) {
        0 => Ok(()),
        -1 => Err("an error occurred".to_string()),
        -2 => Err("timed out".to_string()),
        _ => Err("unknown error".to_string())
      }
    }
  }

  pub fn pending(&self) -> Result<bool, String> {
    unsafe {
      match wpa_ctrl_pending(self.handle) {
        0 => Ok(false),
        1 => Ok(true),
        -1 => Err("error".to_string()),
        _ => Err("unknown error".to_string())
      }
    }
  }

  pub fn recv(&self) -> Result<String, String> {
    unsafe {
      let mut res = Vec::<u8>::with_capacity(500);
      let mut res_len = 500;
      match wpa_ctrl_recv(self.handle, res.as_mut_ptr() as *mut c_char, &mut res_len) {
        0 => {
          res.set_len(res_len as usize);
          Ok(String::from_utf8(res).unwrap())
        },
        -1 => Err("an error occurred".to_string()),
        _ => Err("unknown error".to_string())
      }
    }
  }
}

impl Drop for WpaCtrl {
  fn drop(&mut self) {
    unsafe { wpa_ctrl_close(self.handle); }
  }
}
