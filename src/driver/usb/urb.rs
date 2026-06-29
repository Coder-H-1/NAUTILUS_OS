#[derive(Copy, Clone, PartialEq)]
pub enum UrbDirection {
    In,
    Out,
    Setup,
}

#[derive(Copy, Clone, PartialEq)]
pub enum EndpointType {
    Control = 0,
    Isochronous = 1,
    Bulk = 2,
    Interrupt = 3,
}

pub struct Urb {
    pub dev_addr: u8,
    pub ep_num: u8,
    pub ep_type: EndpointType,
    pub direction: UrbDirection,
    pub max_packet_size: u16,
    pub buffer: *mut u8,
    pub buffer_length: u32,
    pub speed: u8, // 0=High, 1=Full, 2=Low
    pub pid: u8,   // 0=DATA0, 2=DATA1, 3=SETUP
}

impl Urb {
    pub fn new(
        dev_addr: u8,
        ep_num: u8,
        ep_type: EndpointType,
        direction: UrbDirection,
        max_packet_size: u16,
        buffer: *mut u8,
        buffer_length: u32,
    ) -> Self {
        let speed = unsafe {
            if dev_addr == 0 {
                crate::driver::usb::ADDR_TO_SPEED[0]
            } else if dev_addr == 1 {
                0
            } else {
                crate::driver::usb::ADDR_TO_SPEED[dev_addr as usize]
            }
        };
        Urb {
            dev_addr,
            ep_num,
            ep_type,
            direction,
            max_packet_size,
            buffer,
            buffer_length,
            speed,
            pid: match direction { UrbDirection::Setup => 3, _ => 0 },
        }
    }
}
