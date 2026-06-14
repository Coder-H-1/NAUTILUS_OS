use crate::driver::hdmi::Hdmi;
use crate::driver::usb::urb::{Urb, UrbDirection, EndpointType};
use crate::driver::usb::dwc2_module::hcd::submit_urb;

pub struct Keyboard {
    dev_addr: u8,
    interrupt_in_ep: u8,
    last_keycode: u8,
}

static mut KEYBOARD: Option<Keyboard> = None;

impl Keyboard {
    pub fn new(dev_addr: u8, interrupt_in_ep: u8) -> Self {
        Keyboard { dev_addr, interrupt_in_ep, last_keycode: 0 }
    }

    pub fn poll(&mut self) -> Option<char> {
        let mut report = [0u8; 8];

        let urb = Urb::new(
            self.dev_addr, self.interrupt_in_ep, EndpointType::Interrupt, UrbDirection::In,
            8, report.as_mut_ptr(), 8
        );
        
        if submit_urb(&urb) {
            let keycode = report[2];
            if keycode != 0 && keycode != self.last_keycode {
                self.last_keycode = keycode;
                return Some(Self::keycode_to_char(keycode, report[0]));
            } else if keycode == 0 {
                self.last_keycode = 0;
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
    unsafe { KEYBOARD = Some(Keyboard::new(3, 1)); }
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
