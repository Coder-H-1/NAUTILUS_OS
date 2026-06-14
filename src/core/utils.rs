pub fn delay(ms: u32) {
    crate::driver::timer::Timer::delay_ms(ms as u64);
}

use core::sync::atomic::{AtomicU32, Ordering};

static SEED: AtomicU32 = AtomicU32::new(12345);

pub fn random(max: u32) -> u32 {
    let mut seed = SEED.load(Ordering::Relaxed);
    seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    SEED.store(seed, Ordering::Relaxed);
    if max == 0 {
        return 0;
    }
    (seed / 65536) % max
}
