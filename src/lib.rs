#[macro_use]
extern crate failure;
extern crate libc;
#[macro_use]
extern crate lazy_static;
//extern crate dbus;

mod wpactrl;
//mod wpactrldbus;
pub use wpactrl::WpaCtrl;
//pub use wpactrldbus::WpaCtrlDbus;
