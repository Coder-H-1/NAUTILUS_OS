use core::ptr::read_volatile;

const TIMER_BASE: u32 = 0x3F00_3000;
const TIMER_CLO: *const u32 = (TIMER_BASE + 0x04) as *const u32;
const TIMER_CHI: *const u32 = (TIMER_BASE + 0x08) as *const u32;

pub struct Timer;

impl Timer {
    pub fn init() {
        // Timer runs by default on RPi
    }

    pub fn get_time_us() -> u64 {
        let mut hi = unsafe { read_volatile(TIMER_CHI) };
        let mut lo = unsafe { read_volatile(TIMER_CLO) };
        
        let hi2 = unsafe { read_volatile(TIMER_CHI) };
        if hi != hi2 {
            hi = hi2;
            lo = unsafe { read_volatile(TIMER_CLO) };
        }
        
        ((hi as u64) << 32) | (lo as u64)
    }

    pub fn delay_us(us: u64) {
        let start = Self::get_time_us();
        while Self::get_time_us() - start < us {
            core::hint::spin_loop();
        }
    }
    
    pub fn delay_ms(ms: u64) {
        Self::delay_us(ms * 1000);
    }
}
