use crate::driver::hdmi::Hdmi;
use crate::driver::usb::urb::{Urb, UrbDirection, EndpointType};
use crate::driver::usb::dwc2_module::hcd::submit_urb;
use crate::driver::usb::setup::UsbDeviceRequest;

pub const HUB_SET_FEATURE: u8 = 3;
pub const PORT_POWER_FEATURE: u16 = 8;

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

        let urb = Urb::new(
            hub_addr, 0, EndpointType::Control, UrbDirection::Setup,
            64, &mut req as *mut _ as *mut u8, 8
        );
        
        submit_urb(&urb);
    }
    
    Hdmi::write_str("USB Hub: Port power ON requests sent.\n");
}
