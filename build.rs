extern crate cc;

use std::path::Path;
use std::process::Command;

fn main() {
    if !Path::new("src/wpa_supplicant/.git").exists() {
        let _ = Command::new("git").args(&["submodule", "update", "--init"])
                                   .status();
    }

    cc::Build::new()
    .define("CONFIG_CTRL_IFACE", "unix")
    .define("CONFIG_CTRL_IFACE_UNIX", "y")
    .file("src/wpa_supplicant/src/utils/common.c")
    .file("src/wpa_supplicant/src/utils/os_unix.c")
    .file("src/wpa_supplicant/src/utils/wpa_debug.c")
    .file("src/wpa_supplicant/src/common/wpa_ctrl.c")
    .include("src/wpa_supplicant/src")
    .include("src/wpa_supplicant/src/utils")
    .compile("libwpactrl.a");
}
