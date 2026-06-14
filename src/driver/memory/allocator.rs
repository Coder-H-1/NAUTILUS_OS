// Physical Page Allocator

const PAGE_SIZE: usize = 4096;
const MEMORY_SIZE: usize = 128 * 1024 * 1024; // Assume 128MB
const NUM_PAGES: usize = MEMORY_SIZE / PAGE_SIZE;

static mut PAGE_BITMAP: [u8; NUM_PAGES / 8] = [0; NUM_PAGES / 8];

pub fn init() {
    // Reserve kernel pages (first 16MB)
    let kernel_pages = (16 * 1024 * 1024) / PAGE_SIZE;
    unsafe {
        for i in 0..kernel_pages {
            set_page_used(i);
        }
    }
}

unsafe fn set_page_used(page_idx: usize) {
    let byte_idx = page_idx / 8;
    let bit_idx = page_idx % 8;
    PAGE_BITMAP[byte_idx] |= 1 << bit_idx;
}

unsafe fn set_page_free(page_idx: usize) {
    let byte_idx = page_idx / 8;
    let bit_idx = page_idx % 8;
    PAGE_BITMAP[byte_idx] &= !(1 << bit_idx);
}

pub fn alloc_page() -> Option<usize> {
    unsafe {
        for i in 0..NUM_PAGES {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            if (PAGE_BITMAP[byte_idx] & (1 << bit_idx)) == 0 {
                set_page_used(i);
                return Some(i * PAGE_SIZE);
            }
        }
    }
    None
}

pub fn free_page(phys_addr: usize) {
    let page_idx = phys_addr / PAGE_SIZE;
    unsafe {
        set_page_free(page_idx);
    }
}
