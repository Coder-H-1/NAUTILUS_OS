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

    let urb_setup = Urb::new(dev_addr, 0, EndpointType::Control, UrbDirection::Setup, 64, &mut req as *mut _ as *mut u8, 8);
    for _ in 0..3 { if submit_urb(&urb_setup) { break; } }
    
    let mut desc = [0u32; 5]; // 20 bytes, u32 for 4-byte alignment
    let mut urb_in = Urb::new(dev_addr, 0, EndpointType::Control, UrbDirection::In, 64, desc.as_mut_ptr() as *mut u8, 18);
    urb_in.pid = 2; // DATA1
    for _ in 0..3 { if submit_urb(&urb_in) { break; } }
    
    // Status Stage
    let mut urb_out = Urb::new(dev_addr, 0, EndpointType::Control, UrbDirection::Out, 64, core::ptr::null_mut(), 0);
    urb_out.pid = 2; // DATA1
    for _ in 0..3 { if submit_urb(&urb_out) { break; } }
    
    Hdmi::write_str("USB: Device Descriptor requested.\n");
}

pub fn get_config_descriptor(dev_addr: u8, buf: *mut u8, len: u16) -> bool {
    let mut req = UsbDeviceRequest {
        bm_request_type: 0x80, // Device to Host
        b_request: REQ_GET_DESCRIPTOR,
        w_value: 0x0200, // Descriptor type 2 (Configuration)
        w_index: 0,
        w_length: len,
    };

    let urb_setup = Urb::new(dev_addr, 0, EndpointType::Control, UrbDirection::Setup, 64, &mut req as *mut _ as *mut u8, 8);
    let mut success = false;
    for _ in 0..3 { if submit_urb(&urb_setup) { success = true; break; } }
    if !success { return false; }
    
    let mut urb_in = Urb::new(dev_addr, 0, EndpointType::Control, UrbDirection::In, 64, buf, len as u32);
    urb_in.pid = 2; // DATA1
    success = false;
    for _ in 0..3 { if submit_urb(&urb_in) { success = true; break; } }
    if !success { return false; }
    
    // Status Stage
    let mut urb_out = Urb::new(dev_addr, 0, EndpointType::Control, UrbDirection::Out, 64, core::ptr::null_mut(), 0);
    urb_out.pid = 2; // DATA1
    for _ in 0..3 { if submit_urb(&urb_out) { break; } }
    
    true
}

pub fn set_address(dev_addr: u8, new_addr: u8) {
    let mut req = UsbDeviceRequest {
        bm_request_type: 0x00, // Host to Device
        b_request: REQ_SET_ADDRESS,
        w_value: new_addr as u16,
        w_index: 0,
        w_length: 0,
    };

    let urb_setup = Urb::new(dev_addr, 0, EndpointType::Control, UrbDirection::Setup, 64, &mut req as *mut _ as *mut u8, 8);
    for _ in 0..3 { if submit_urb(&urb_setup) { break; } }
    
    // Status Stage
    let mut urb_in = Urb::new(dev_addr, 0, EndpointType::Control, UrbDirection::In, 64, core::ptr::null_mut(), 0);
    urb_in.pid = 2; // DATA1 for Status
    for _ in 0..3 { if submit_urb(&urb_in) { break; } }
}

pub fn set_configuration(dev_addr: u8, config_val: u8) {
    let mut req = UsbDeviceRequest {
        bm_request_type: 0x00, // Host to Device
        b_request: REQ_SET_CONFIGURATION,
        w_value: config_val as u16,
        w_index: 0,
        w_length: 0,
    };

    let urb_setup = Urb::new(dev_addr, 0, EndpointType::Control, UrbDirection::Setup, 64, &mut req as *mut _ as *mut u8, 8);
    for _ in 0..3 { if submit_urb(&urb_setup) { break; } }
    
    // Status Stage
    let mut urb_in = Urb::new(dev_addr, 0, EndpointType::Control, UrbDirection::In, 64, core::ptr::null_mut(), 0);
    urb_in.pid = 2; // DATA1 for Status
    for _ in 0..3 { if submit_urb(&urb_in) { break; } }
}

pub fn enumerate() {
    Hdmi::write_str("USB: Starting Enumeration (Phase 3)...\n");
    get_device_descriptor(0);
    set_address(0, 1);
    
    crate::core::utils::delay(10); // Wait for address to settle
    set_configuration(1, 1); // Configure Hub
    
    Hdmi::write_str("USB: Hub Enumerated!\n");
}
