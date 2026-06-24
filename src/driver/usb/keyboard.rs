use crate::driver::hdmi::Hdmi;
use crate::driver::usb::urb::{Urb, UrbDirection, EndpointType};
use crate::driver::usb::dwc2_module::hcd::submit_urb;

pub struct Keyboard {
    dev_addr: u8,
    interrupt_in_ep: u8,
    last_keycode: u8,
    next_pid: u8,
}

static mut KEYBOARD: Option<Keyboard> = None;

impl Keyboard {
    pub fn new(dev_addr: u8, interrupt_in_ep: u8) -> Self {
        Keyboard { dev_addr, interrupt_in_ep, last_keycode: 0, next_pid: 0 }
    }

    pub fn poll(&mut self) -> Option<char> {
        let mut report_buf = [0u32; 2]; // 8 bytes, u32 for 4-byte alignment

        // Probe addresses if we haven't locked onto one
        let addrs_to_try = if self.dev_addr == 0 { 2..=5 } else { self.dev_addr..=self.dev_addr };

        for addr in addrs_to_try {
            let mut urb = Urb::new(
                addr, self.interrupt_in_ep, EndpointType::Bulk, UrbDirection::In,
                8, report_buf.as_mut_ptr() as *mut u8, 8
            );
            urb.pid = self.next_pid;
            
            if submit_urb(&urb) {
                // Lock onto this address
                self.dev_addr = addr;
                
                // Toggle PID on successful transaction
                self.next_pid = if self.next_pid == 0 { 2 } else { 0 };

                let report_bytes = [
                    (report_buf[0] & 0xFF) as u8,
                    ((report_buf[0] >> 8) & 0xFF) as u8,
                    ((report_buf[0] >> 16) & 0xFF) as u8,
                    ((report_buf[0] >> 24) & 0xFF) as u8,
                    (report_buf[1] & 0xFF) as u8,
                    ((report_buf[1] >> 8) & 0xFF) as u8,
                    ((report_buf[1] >> 16) & 0xFF) as u8,
                    ((report_buf[1] >> 24) & 0xFF) as u8,
                ];

                let keycode = report_bytes[2];
                if keycode != 0 && keycode != self.last_keycode {
                    self.last_keycode = keycode;
                    return Some(Self::keycode_to_char(keycode, report_bytes[0]));
                } else if keycode == 0 {
                    self.last_keycode = 0;
                }
                return None;
            }
        }
        None
    }

    fn keycode_to_char(code: u8, modifiers: u8) -> char {
        let shift = (modifiers & 0x02) != 0 || (modifiers & 0x20) != 0;
        match code {
            0x04..=0x1D => {
                let c = (code - 0x04) + b'a';
                if shift { (c - 32) as char } else { c as char }
            },
            0x1E..=0x26 => {
                let c = (code - 0x1E) + b'1';
                if shift {
                    match code {
                        0x1E => '!', 0x1F => '@', 0x20 => '#', 0x21 => '$', 0x22 => '%',
                        0x23 => '^', 0x24 => '&', 0x25 => '*', 0x26 => '(',
                        _ => ' ',
                    }
                } else {
                    c as char
                }
            },
            0x27 => if shift { ')' } else { '0' },
            0x28 => '\n',
            0x2A => '\x08', // Backspace
            0x2C => ' ',
            0x2D => if shift { '_' } else { '-' },
            0x2E => if shift { '+' } else { '=' },
            0x2F => if shift { '{' } else { '[' },
            0x30 => if shift { '}' } else { ']' },
            0x31 => if shift { '|' } else { '\\' },
            0x33 => if shift { ':' } else { ';' },
            0x34 => if shift { '"' } else { '\'' },
            0x35 => if shift { '~' } else { '`' },
            0x36 => if shift { '<' } else { ',' },
            0x37 => if shift { '>' } else { '.' },
            0x38 => if shift { '?' } else { '/' },
            _ => '\0',
        }
    }
}

pub fn init() {
    Hdmi::write_str("USB: Initializing HID Keyboard (Phase 5)...\n");
    unsafe { KEYBOARD = Some(Keyboard::new(0, 1)); }
    Hdmi::write_str("USB HID: Keyboard ready.\n");
}

pub fn get_key() -> Option<char> {
    unsafe {
        if let Some(ref mut kb) = KEYBOARD {
            return kb.poll();
        }
    }
    None
}
