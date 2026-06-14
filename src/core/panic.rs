use ::core::panic::PanicInfo;
use crate::driver::hdmi::Hdmi;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    Hdmi::write_str("KERNEL PANIC!\n");
    loop {}
}
