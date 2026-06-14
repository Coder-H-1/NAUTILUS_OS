#![no_std]
#![no_main]

mod driver;
mod core;
mod kernel;
mod fs;

use ::core::arch::global_asm;

// Minimal _start routine that branches to kernel_main
global_asm!(
    ".section .text._start",
    ".globl _start",
    "_start:",
    "    // read cpu id, stop slave cores",
    "    mrs     x1, mpidr_el1",
    "    and     x1, x1, #3",
    "    cbz     x1, 2f",
    "1:  wfe",
    "    b       1b",
    "2:  // cpu id == 0",
    "    // set stack before our code",
    "    ldr     x1, =_start",
    "    mov     sp, x1",
    "    // jump to C/Rust code, should not return",
    "    bl      rust_main",
    "    // for failsafe, halt this core too",
    "    b       1b",
);

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    kernel::main::kernel_main()
}
