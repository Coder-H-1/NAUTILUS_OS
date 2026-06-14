use crate::driver;
use crate::core::shell;
use crate::kernel::exception;
use crate::kernel::process;

pub fn kernel_main() -> ! {
    // initialize drivers
    driver::clock::Clock::init();
    driver::timer::Timer::init();
    driver::memory::Memory::init();
    driver::hdmi::Hdmi::init();
    driver::usb::init();
    crate::fs::init();

    // initialize os components
    exception::init();
    process::init();

    // route to shell
    shell::start();

    loop {}
}
