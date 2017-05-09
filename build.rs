extern crate gcc;

fn main() {
    gcc::Config::new()
    .file("src/wpa_supplicant/src/common/wpa_ctrl.c")
    .include("src/wpa_supplicant/src/common")
    .include("src/wpa_supplicant/src/utils")
    .compile("libwpactrl.a");
}
