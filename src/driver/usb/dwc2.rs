use crate::driver::hdmi::Hdmi;

pub fn init_controller() {
    // Stub for DWC2 controller hardware initialization
    // Base address is usually 0x3F980000 on RPI3
    Hdmi::write_str("Resetting DWC2 USB Controller...\n");
    // Hardware reset and power on
    Hdmi::write_str("DWC2 Power On.\n");
}
