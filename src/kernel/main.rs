use crate::driver;
use crate::core::shell;

pub fn kernel_main() -> ! {
    // initialize drivers
    driver::clock::Clock::init();
    driver::timer::Timer::init();
    driver::memory::Memory::init();
    driver::hdmi::Hdmi::init();
    driver::usb::init();
    crate::fs::init();

    // route to shell
    shell::start();

    loop {}
}
