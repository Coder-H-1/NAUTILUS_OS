use super::timer::Timer;

pub struct Clock;

impl Clock {
    pub fn init() {
        Timer::init();
    }
    
    pub fn get_uptime_ms() -> u64 {
        Timer::get_time_us() / 1000
    }
}
