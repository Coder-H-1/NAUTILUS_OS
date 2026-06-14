use core::ptr::{read_volatile, write_volatile};

pub const DWC2_BASE: u32 = 0x3F98_0000;

pub struct CoreRegisters {
    pub gotgctrl: u32, // 0x000
    pub gotgint: u32,  // 0x004
    pub gahbcfg: u32,  // 0x008
    pub gusbcfg: u32,  // 0x00C
    pub grstctl: u32,  // 0x010
    pub gintsts: u32,  // 0x014
    pub gintmsk: u32,  // 0x018
    // Skipping other registers for now to keep simple
}

pub struct HostRegisters {
    pub hcfg: u32,     // 0x400
    pub hfir: u32,     // 0x404
    pub hfnum: u32,    // 0x408
    pub hptxsts: u32,  // 0x410
    pub haint: u32,    // 0x414
    pub haintmsk: u32, // 0x418
    pub hprt0: u32,    // 0x440
}

pub fn write_reg(offset: u32, val: u32) {
    unsafe { write_volatile((DWC2_BASE + offset) as *mut u32, val) };
}

pub fn read_reg(offset: u32) -> u32 {
    unsafe { read_volatile((DWC2_BASE + offset) as *const u32) }
}
