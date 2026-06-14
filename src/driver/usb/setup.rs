use crate::driver::hdmi::Hdmi;
use crate::driver::usb::urb::{Urb, UrbDirection, EndpointType};
use crate::driver::usb::dwc2_module::hcd::submit_urb;

#[repr(C, align(4))]
pub struct UsbDeviceRequest {
    pub bm_request_type: u8,
    pub b_request: u8,
    pub w_value: u16,
    pub w_index: u16,
    pub w_length: u16,
}

pub const REQ_GET_DESCRIPTOR: u8 = 6;
pub const REQ_SET_ADDRESS: u8 = 5;
pub const REQ_SET_CONFIGURATION: u8 = 9;

pub fn get_device_descriptor(dev_addr: u8) {
    Hdmi::write_str("USB: Requesting Device Descriptor...\n");

    let mut req = UsbDeviceRequest {
        bm_request_type: 0x80, // Device to Host
        b_request: REQ_GET_DESCRIPTOR,
        w_value: 0x0100, // Descriptor type 1 (Device)
        w_index: 0,
        w_length: 18,
    };

    let urb = Urb::new(
        dev_addr, 0, EndpointType::Control, UrbDirection::Setup,
        64, &mut req as *mut _ as *mut u8, 8
    );
    
    submit_urb(&urb);
    Hdmi::write_str("USB: Device Descriptor requested.\n");
}

pub fn set_address(new_addr: u8) {
    Hdmi::write_str("USB: Setting Address to ");
    // Fake print address
    if new_addr == 1 { Hdmi::write_str("1...\n"); } else { Hdmi::write_str("?\n"); }

    let mut req = UsbDeviceRequest {
        bm_request_type: 0x00, // Host to Device
        b_request: REQ_SET_ADDRESS,
        w_value: new_addr as u16,
        w_index: 0,
        w_length: 0,
    };

    let urb = Urb::new(
        0, 0, EndpointType::Control, UrbDirection::Setup,
        64, &mut req as *mut _ as *mut u8, 8
    );
    
    submit_urb(&urb);
}

pub fn enumerate() {
    Hdmi::write_str("USB: Starting Enumeration (Phase 3)...\n");
    get_device_descriptor(0);
    set_address(1);
    Hdmi::write_str("USB: Enumeration basic steps sent!\n");
}
