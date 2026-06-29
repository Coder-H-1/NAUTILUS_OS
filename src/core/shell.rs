use crate::driver::hdmi::Hdmi;

pub fn start() {
    Hdmi::write_str("Welcome to shell\n");
    Hdmi::write_str("OS> ");
    
    loop {}
}
