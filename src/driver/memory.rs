pub mod allocator;
pub mod mmu;

pub struct Memory;

impl Memory {
    pub fn init() {
        allocator::init();
        mmu::init();
    }
}
