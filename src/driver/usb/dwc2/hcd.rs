use crate::driver::usb::urb::{Urb, UrbDirection};
use crate::driver::hdmi::Hdmi;
use super::registers::write_reg;

pub fn submit_urb(urb: &Urb) -> bool {
    let hc_offset = 0x500; // Host Channel 0 base

    // Calculate HCCHAR (Host Channel Characteristics)
    let ep_type = urb.ep_type as u32;
    let ep_dir = if urb.direction == UrbDirection::In { 1 } else { 0 };
    let mps = urb.max_packet_size as u32;
    let dev_addr = urb.dev_addr as u32;
    let ep_num = urb.ep_num as u32;
    let lspddev = if urb.speed == 2 { 1 } else { 0 };

    let mut hcchar = (mps & 0x7FF)
        | ((ep_num & 0xF) << 11)
        | ((ep_dir & 0x1) << 15)
        | ((lspddev & 0x1) << 17)
        | ((ep_type & 0x3) << 18)
        | (1 << 20) // MultiCnt = 1
        | ((dev_addr & 0x7F) << 22);

    if urb.ep_type == crate::driver::usb::urb::EndpointType::Interrupt || urb.ep_type == crate::driver::usb::urb::EndpointType::Isochronous {
        let hfnum = super::registers::read_reg(0x408);
        let odd = (hfnum & 1) ^ 1;
        hcchar |= odd << 29;
    }

    // Calculate HCTSIZ (Host Channel Transfer Size)
    let pktcnt = if urb.buffer_length == 0 { 1 } else { (urb.buffer_length + mps - 1) / mps };
    
    let pid = urb.pid;

    let hctsiz = (urb.buffer_length & 0x7FFFF)
        | ((pktcnt & 0x3FF) << 19)
        | ((pid as u32 & 0x3) << 29);

    // Write registers
    write_reg(hc_offset + 0x10, hctsiz); // HCTSIZ
    write_reg(hc_offset + 0x14, urb.buffer as u32); // HCDMA

    // Clear all channel interrupts (HCINT)
    write_reg(hc_offset + 0x08, 0xFFFFFFFF);

    // Enable channel (Set CHENA)
    hcchar |= 1 << 31;
    write_reg(hc_offset + 0x00, hcchar); // HCCHAR

    // Wait for transfer complete or error
    let mut spin_count = 0;
    loop {
        let hcint = super::registers::read_reg(hc_offset + 0x08);
        if (hcint & 1) != 0 { // XFERCOMP
            return true;
        }
        if (hcint & (1 << 4)) != 0 { // NAK
            let mut char_reg = super::registers::read_reg(hc_offset + 0x00);
            char_reg |= (1 << 30) | (1 << 31); // CHDIS | CHENA
            super::registers::write_reg(hc_offset + 0x00, char_reg);
            
            let mut wait_count = 0;
            loop {
                if (super::registers::read_reg(hc_offset + 0x08) & 2) != 0 { break; }
                wait_count += 1;
                if wait_count > 100_000 { break; }
            }
            return false;
        }
        if (hcint & (1 << 7)) != 0 { return false; } // XACTERR
        if (hcint & (1 << 3)) != 0 { return false; } // STALL
        if (hcint & 2) != 0 { return false; } // CHHLTD (without XFERCOMP)
        if (hcint & (1 << 9)) != 0 { return false; } // FRMOVRUN
        if (hcint & (1 << 8)) != 0 { return false; } // BBLERR
        if (hcint & (1 << 10)) != 0 { return false; } // DATATGLERR
        
        spin_count += 1;
        if spin_count > 50_000_000 {
            Hdmi::write_str("HCINT HANG! hcint=");
            for i in (0..8).rev() {
                let nibble = (hcint >> (i * 4)) & 0xF;
                let c = if nibble < 10 { (b'0' + nibble as u8) as char } else { (b'A' + (nibble - 10) as u8) as char };
                Hdmi::draw_char(c);
            }
            Hdmi::write_str("\n");
            
            // Force halt
            let mut char_reg = super::registers::read_reg(hc_offset + 0x00);
            char_reg |= (1 << 30) | (1 << 31); // CHDIS | CHENA
            super::registers::write_reg(hc_offset + 0x00, char_reg);
            
            return false;
        }
    }
}
