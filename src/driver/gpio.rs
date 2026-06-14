use core::ptr::{read_volatile, write_volatile};

const GPIO_BASE: u32 = 0x3F20_0000;

const GPFSEL0: *mut u32 = (GPIO_BASE + 0x00) as *mut u32;
const GPSET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
const GPCLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;
const GPLEV0: *mut u32 = (GPIO_BASE + 0x34) as *mut u32;

#[derive(Copy, Clone)]
pub enum PinMode {
    Input = 0,
    Output = 1,
    Alt5 = 2,
    Alt4 = 3,
    Alt0 = 4,
    Alt1 = 5,
    Alt2 = 6,
}

pub fn set_mode(pin: u32, mode: PinMode) {
    if pin > 53 { return; }
    
    let reg = unsafe { GPFSEL0.add((pin / 10) as usize) };
    let shift = (pin % 10) * 3;
    
    unsafe {
        let mut val = read_volatile(reg);
        val &= !(7 << shift);
        val |= (mode as u32) << shift;
        write_volatile(reg, val);
    }
}

pub fn set_pin(pin: u32, state: bool) {
    if pin > 53 { return; }
    
    if state {
        let reg = unsafe { GPSET0.add((pin / 32) as usize) };
        unsafe { write_volatile(reg, 1 << (pin % 32)) };
    } else {
        let reg = unsafe { GPCLR0.add((pin / 32) as usize) };
        unsafe { write_volatile(reg, 1 << (pin % 32)) };
    }
}

pub fn get_pin(pin: u32) -> bool {
    if pin > 53 { return false; }
    
    let reg = unsafe { GPLEV0.add((pin / 32) as usize) };
    let val = unsafe { read_volatile(reg) };
    (val & (1 << (pin % 32))) != 0
}
