pub mod vfs;
pub mod fat32;

use crate::driver::hdmi::Hdmi;

pub fn init() {
    Hdmi::write_str("Starting Filesystem init...\n");
    vfs::init();
    fat32::init();
    Hdmi::write_str("Filesystem init done!\n");
}
