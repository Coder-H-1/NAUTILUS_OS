use core::arch::asm;
use crate::driver::memory::allocator;

const PAGE_SIZE: usize = 4096;

// Memory Attributes
const MAIR_NORMAL: u64 = 0xFF; // Normal, Inner/Outer write-back non-transient
const MAIR_DEVICE: u64 = 0x00; // Device-nGnRnE

// TCR Flags
const TCR_T0SZ: u64 = (64 - 48) << 0;
const TCR_T1SZ: u64 = (64 - 48) << 16;
const TCR_TG0_4K: u64 = 0 << 14;
const TCR_TG1_4K: u64 = 2 << 30;

#[repr(align(4096))]
pub struct PageTable {
    pub entries: [u64; 512],
}

pub fn init() {
    unsafe {
        // 1. Set MAIR_EL1
        let mair = (MAIR_NORMAL << 0) | (MAIR_DEVICE << 8);
        asm!("msr mair_el1, {}", in(reg) mair);

        // 2. Set TCR_EL1 (48-bit VA, 4KB page size)
        let tcr = TCR_T0SZ | TCR_T1SZ | TCR_TG0_4K | TCR_TG1_4K;
        asm!("msr tcr_el1, {}", in(reg) tcr);

        // 3. Allocate a top-level page table for the kernel
        if let Some(pg_dir_phys) = allocator::alloc_page() {
            // Zero out the page table
            core::ptr::write_bytes(pg_dir_phys as *mut u8, 0, PAGE_SIZE);

            // Set TTBR1_EL1 to the newly allocated page directory
            asm!("msr ttbr1_el1, {}", in(reg) pg_dir_phys);
            
            // Set TTBR0_EL1 to another one for user space (optional early on)
            if let Some(user_pg_dir) = allocator::alloc_page() {
                core::ptr::write_bytes(user_pg_dir as *mut u8, 0, PAGE_SIZE);
                asm!("msr ttbr0_el1, {}", in(reg) user_pg_dir);
            }

            // Enable MMU in SCTLR_EL1 (M bit)
            let mut sctlr: u64;
            asm!("mrs {}, sctlr_el1", out(reg) sctlr);
            sctlr |= 1; // Set M bit
            asm!("msr sctlr_el1, {}", in(reg) sctlr);
            
            // Invalidate TLB
            asm!("tlbi vmalle1is", "dsb ish", "isb");
        }
    }
}

pub fn map_page(virt_addr: usize, phys_addr: usize, flags: u64) {
    // In a full implementation, this walks the 4-level page table:
    // L0 -> L1 -> L2 -> L3
    // Since this is bare metal, we extract indices from the virtual address:
    // let l0_idx = (virt_addr >> 39) & 0x1FF;
    // let l1_idx = (virt_addr >> 30) & 0x1FF;
    // let l2_idx = (virt_addr >> 21) & 0x1FF;
    // let l3_idx = (virt_addr >> 12) & 0x1FF;
    
    // 1. Get TTBR1_EL1 or TTBR0_EL1 depending on virt_addr
    // 2. Follow pointers, allocating new tables via `allocator::alloc_page()` if not present (valid bit == 0)
    // 3. Set terminal L3 entry to `phys_addr | flags | 0b11` (Page entry, valid)
}
