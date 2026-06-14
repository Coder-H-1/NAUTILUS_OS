#[repr(C)]
pub struct TrapFrame {
    pub regs: [u64; 31], // x0 to x30
    pub sp: u64,
    pub elr: u64,
    pub spsr: u64,
}

#[repr(C)]
pub struct Context {
    pub x19_to_x29: [u64; 11],
    pub lr: u64,
    pub sp: u64,
}

pub struct ProcessControlBlock {
    pub id: u32,
    pub state: u32, // 0: ready, 1: running
    pub context: Context,
    pub trap_frame: *mut TrapFrame,
}

pub fn init() {
    // Initialize process list
}

pub fn switch_to_user(pc: usize, sp: usize) {
    unsafe {
        core::arch::asm!(
            "msr elr_el1, {0}",
            "msr sp_el0, {1}",
            "mov x0, #0", // SPSR_EL1 value for EL0t
            "msr spsr_el1, x0",
            "eret",
            in(reg) pc,
            in(reg) sp,
            out("x0") _
        );
    }
}

