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

    let is_split = urb.speed != 0;
    let hub_addr = 1u32;
    let hub_port = unsafe { crate::driver::usb::ADDR_TO_PORT[(urb.dev_addr & 0xF) as usize] } as u32;

    let hcsplt_ssplit = (1 << 31) | (0 << 16) | (hub_addr << 7) | hub_port;
    let hcsplt_csplit = (1 << 31) | (1 << 16) | (hub_addr << 7) | hub_port;

    let do_transaction = |is_csplit: bool| -> bool {
        write_reg(hc_offset + 0x10, hctsiz); // HCTSIZ
        write_reg(hc_offset + 0x14, urb.buffer as u32); // HCDMA
        write_reg(hc_offset + 0x08, 0xFFFFFFFF); // Clear interrupts
        
        if is_split {
            write_reg(hc_offset + 0x04, if is_csplit { hcsplt_csplit } else { hcsplt_ssplit });
        } else {
            write_reg(hc_offset + 0x04, 0);
        }

        let mut char_reg = hcchar | (1 << 31); // CHENA
        write_reg(hc_offset + 0x00, char_reg);

        let mut spin_count = 0;
        loop {
            let hcint = super::registers::read_reg(hc_offset + 0x08);
            if (hcint & 1) != 0 { // XFERCOMP
                // Wait for CHHLTD
                while (super::registers::read_reg(hc_offset + 0x08) & 2) == 0 {}
                return true;
            }
            
            let has_err = (hcint & ((1 << 7) | (1 << 3) | (1 << 9) | (1 << 8) | (1 << 10))) != 0;
            let has_nak = (hcint & (1 << 4)) != 0; // NYET maps to NAK here
            
            if has_err || has_nak {
                let mut c = super::registers::read_reg(hc_offset + 0x00);
                c |= (1 << 30) | (1 << 31); // CHDIS | CHENA
                super::registers::write_reg(hc_offset + 0x00, c);
                while (super::registers::read_reg(hc_offset + 0x08) & 2) == 0 {}
                return false;
            }
            
            if (hcint & 2) != 0 { return false; } // CHHLTD without XFERCOMP
            
            spin_count += 1;
            if spin_count > 10_000_000 {
                let mut c = super::registers::read_reg(hc_offset + 0x00);
                c |= (1 << 30) | (1 << 31); // CHDIS | CHENA
                super::registers::write_reg(hc_offset + 0x00, c);
                while (super::registers::read_reg(hc_offset + 0x08) & 2) == 0 {}
                return false;
            }
        }
    };

    if !is_split {
        return do_transaction(false);
    }

    // SPLIT Transaction
    // 1. Start Split
    if !do_transaction(false) {
        return false;
    }

    // 2. Complete Split
    crate::core::utils::delay(1); // Wait for hub to process

    for _ in 0..10 {
        if do_transaction(true) {
            return true;
        }
        crate::core::utils::delay(1); // Retry CSPLIT on NYET (mapped to NAK)
    }

    false
}
