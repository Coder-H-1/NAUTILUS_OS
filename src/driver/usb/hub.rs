use crate::driver::hdmi::Hdmi;
use crate::driver::usb::urb::{Urb, UrbDirection, EndpointType};
use crate::driver::usb::dwc2_module::hcd::submit_urb;
use crate::driver::usb::setup::{UsbDeviceRequest, REQ_SET_ADDRESS};
use crate::core::utils::delay;

pub const HUB_SET_FEATURE: u8 = 3;
pub const HUB_GET_PORT_STATUS: u8 = 0;
pub const PORT_POWER_FEATURE: u16 = 8;
pub const PORT_RESET_FEATURE: u16 = 4;

pub fn power_on_ports(hub_addr: u8, num_ports: u8) {
    Hdmi::write_str("USB Hub: Powering on ports...\n");
    for port in 1..=num_ports {
        let mut req = UsbDeviceRequest {
            bm_request_type: 0x23, // Class, Other, Host to Device
            b_request: HUB_SET_FEATURE,
            w_value: PORT_POWER_FEATURE,
            w_index: port as u16,
            w_length: 0,
        };
        let mut urb = Urb::new(hub_addr, 0, EndpointType::Control, UrbDirection::Setup, 64, &mut req as *mut _ as *mut u8, 8);
        for _ in 0..1000 { if submit_urb(&urb) { break; } }
        
        // Status Stage
        let mut urb_in = Urb::new(hub_addr, 0, EndpointType::Control, UrbDirection::In, 64, core::ptr::null_mut(), 0);
        urb_in.pid = 2; // DATA1
        for _ in 0..1000 { if submit_urb(&urb_in) { break; } }
    }
    Hdmi::write_str("USB Hub: Port power ON requests sent.\n");
}

pub fn enumerate_ports(hub_addr: u8, num_ports: u8, mut next_addr: u8) -> u8 {
    Hdmi::write_str("USB Hub: Enumerating ports...\n");
    for port in 1..=num_ports {
        let mut status = [0u32; 1]; // Use u32 to enforce 4-byte alignment for DMA
        let mut req = UsbDeviceRequest {
            bm_request_type: 0xA3, // Class, Other, Device to Host
            b_request: HUB_GET_PORT_STATUS,
            w_value: 0,
            w_index: port as u16,
            w_length: 4,
        };
        let mut urb_setup = Urb::new(hub_addr, 0, EndpointType::Control, UrbDirection::Setup, 64, &mut req as *mut _ as *mut u8, 8);
        for _ in 0..1000 { if submit_urb(&urb_setup) { break; } }
        
        let mut urb_in = Urb::new(hub_addr, 0, EndpointType::Control, UrbDirection::In, 64, status.as_mut_ptr() as *mut u8, 4);
        urb_in.pid = 2; // DATA1
        for _ in 0..1000 { if submit_urb(&urb_in) { break; } }
        
        // Status Stage
        let mut urb_out = Urb::new(hub_addr, 0, EndpointType::Control, UrbDirection::Out, 64, core::ptr::null_mut(), 0);
        urb_out.pid = 2; // DATA1
        for _ in 0..1000 { if submit_urb(&urb_out) { break; } }
        
        let status_bytes = status[0].to_le_bytes();
        let port_status = u16::from_le_bytes([status_bytes[0], status_bytes[1]]);
        
        if (port_status & 1) != 0 { // Current Connect Status
            Hdmi::write_str("USB Hub: Device connected on port ");
            Hdmi::draw_char((b'0' + port as u8) as char);
            Hdmi::write_str("! Resetting...\n");
            
            let mut req_rst = UsbDeviceRequest {
                bm_request_type: 0x23,
                b_request: HUB_SET_FEATURE,
                w_value: PORT_RESET_FEATURE,
                w_index: port as u16,
                w_length: 0,
            };
            let mut urb_rst_setup = Urb::new(hub_addr, 0, EndpointType::Control, UrbDirection::Setup, 64, &mut req_rst as *mut _ as *mut u8, 8);
            for _ in 0..1000 { if submit_urb(&urb_rst_setup) { break; } }
            
            // Status Stage for Set Feature
            let mut urb_rst_in = Urb::new(hub_addr, 0, EndpointType::Control, UrbDirection::In, 64, core::ptr::null_mut(), 0);
            urb_rst_in.pid = 2; // DATA1
            for _ in 0..1000 { if submit_urb(&urb_rst_in) { break; } }
            
            delay(50); // Wait for reset (USB spec min 50ms)
            
            Hdmi::write_str("USB Hub: Assigning address ");
            Hdmi::draw_char((b'0' + next_addr) as char);
            Hdmi::write_str("...\n");
            
            crate::driver::usb::setup::set_address(0, next_addr);
            unsafe { crate::driver::usb::ADDR_TO_PORT[(next_addr & 0xF) as usize] = port as u8; }
            
            delay(10); // Recovery after set address
            
            crate::driver::usb::setup::set_configuration(next_addr, 1);
            
            next_addr += 1;
        }
    }
    next_addr
}
