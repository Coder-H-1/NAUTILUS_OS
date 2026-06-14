use core::arch::asm;
use crate::kernel::process::TrapFrame;

extern "C" {
    static vector_table: u8;
}

pub fn init() {
    // Set VBAR_EL1 to our vector table
    unsafe {
        asm!("msr vbar_el1, {}", in(reg) &vector_table as *const u8 as u64);
    }
}

#[no_mangle]
pub extern "C" fn sync_handler(tf: *mut TrapFrame, esr: u64, _elr: u64, _far: u64) {
    // Check if exception is from SVC instruction (syscall)
    let ec = esr >> 26;
    if ec == 0b010101 { // SVC instruction in AArch64 state
        unsafe {
            crate::kernel::syscall::handle_syscall(&mut *tf);
        }
    }
}

#[no_mangle]
pub extern "C" fn irq_handler() {
    // Handle IRQ
}
