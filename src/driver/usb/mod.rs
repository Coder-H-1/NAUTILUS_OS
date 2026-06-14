#[path = "dwc2/mod.rs"]
pub mod dwc2_module;
pub mod setup;
pub mod mass_storage;
pub mod keyboard;
pub mod urb;
pub mod hub;

use crate::driver::hdmi::Hdmi;

pub fn init() {
    Hdmi::write_str("Initializing USB Driver (Phase 1)...\n");

    if crate::driver::mailbox::power_on_usb() {
        Hdmi::write_str("USB Power On via Mailbox: SUCCESS\n");
    } else {
        Hdmi::write_str("USB Power On via Mailbox: FAILED\n");
    }

    dwc2_module::init();
    
    setup::enumerate();
    hub::power_on_ports(1, 4); // Assume LAN9514 has 4 ports on addr 1
    
    // Give devices time to power up
    crate::core::utils::delay(100);
    
    // Enumerate devices on hub ports, starting from address 2
    hub::enumerate_ports(1, 4, 2);

    Hdmi::write_str("Phase 3 Init Done.\n");

    mass_storage::init();
    keyboard::init();
    
    Hdmi::write_str("USB Fully Initialized!\n");
}
