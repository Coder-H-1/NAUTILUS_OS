pub mod registers;
pub mod hcd;

use crate::driver::hdmi::Hdmi;
use crate::core::utils::delay;
use registers::{read_reg, write_reg};

pub fn init() {
    Hdmi::write_str("DWC2: Starting Core Reset...\n");
    
    // AHB Config: Set global interrupt mask and Enable DMA
    let mut gahbcfg = read_reg(0x008);
    gahbcfg |= 1 << 5; // DMA Enable
    gahbcfg |= 1;        // Global Interrupt Enable
    write_reg(0x008, gahbcfg);
    
    // Core Reset
    let mut grstctl = read_reg(0x010);
    grstctl |= 1; // Core soft reset
    write_reg(0x010, grstctl);
    
    // Wait for reset to finish
    for _ in 0..1000 {
        if (read_reg(0x010) & 1) == 0 {
            break;
        }
        delay(1); // Wait 1ms
    }

    if (read_reg(0x010) & 1) != 0 {
        Hdmi::write_str("DWC2: Core reset timeout!\n");
        return;
    }

    // Force Host Mode
    let mut gusbcfg = read_reg(0x00C);
    gusbcfg |= 1 << 29; // Force host mode
    write_reg(0x00C, gusbcfg);

    // Force FS/LS Only to avoid SPLIT transactions
    let mut hcfg = read_reg(0x400);
    hcfg |= 1 << 2; // FSLSSupp
    hcfg &= !3;     // Clear FSLSPclkSel
    hcfg |= 1;      // Set FSLSPclkSel to 1 (48 MHz)
    write_reg(0x400, hcfg);

    // Configure FIFOs (1024 words each)
    write_reg(0x024, 1024); // GRXFSIZ
    write_reg(0x028, (1024 << 16) | 1024); // GNPTXFSIZ
    write_reg(0x100, (1024 << 16) | 2048); // HPTXFSIZ
    
    // Flush TX FIFOs
    write_reg(0x010, 0x20); // TxFIFO Flush (bit 5)
    for _ in 0..1000 { if (read_reg(0x010) & 0x20) == 0 { break; } }
    // Flush RX FIFO
    write_reg(0x010, 0x10); // RxFIFO Flush (bit 4)
    for _ in 0..1000 { if (read_reg(0x010) & 0x10) == 0 { break; } }

    // Power on and reset root port
    Hdmi::write_str("DWC2: Powering on Root Port...\n");
    let mut hprt = read_reg(0x440);
    hprt &= !( (1 << 1) | (1 << 3) | (1 << 5) );
    hprt |= 1 << 12; // PrtPwr
    write_reg(0x440, hprt);
    
    delay(50); // Wait 50ms for port power
    
    Hdmi::write_str("DWC2: Resetting Root Port...\n");
    hprt = read_reg(0x440);
    hprt &= !( (1 << 1) | (1 << 3) | (1 << 5) );
    hprt |= 1 << 8; // PrtRst
    write_reg(0x440, hprt);
    
    delay(50); // Wait 50ms for reset
    
    hprt = read_reg(0x440);
    hprt &= !( (1 << 1) | (1 << 3) | (1 << 5) | (1 << 8) );
    write_reg(0x440, hprt);
    
    delay(50); // Wait 50ms for port to settle
    
    let final_hprt = read_reg(0x440);
    Hdmi::write_str("DWC2: Root Port ready. HPRT=");
    // crude hex print
    for i in (0..8).rev() {
        let nibble = (final_hprt >> (i * 4)) & 0xF;
        let c = if nibble < 10 { (b'0' + nibble as u8) as char } else { (b'A' + (nibble - 10) as u8) as char };
        Hdmi::draw_char(c);
    }
    Hdmi::write_str("\n");
    if (final_hprt & (1 << 2)) == 0 {
        Hdmi::write_str("DWC2 WARNING: Port NOT enabled (PrtEn=0)!\n");
    }
}
