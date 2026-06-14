use crate::driver::hdmi::Hdmi;
use crate::driver::usb::keyboard::get_key;

pub fn start() {
    Hdmi::write_str("Welcome to shell\n");
    Hdmi::write_str("OS> ");
    
    let mut buffer = [0u8; 128];
    let mut cursor = 0;

    loop {
        if let Some(c) = get_key() {
            if c == '\0' { continue; }
            
            if c == '\n' {
                Hdmi::draw_char('\n');
                if cursor > 0 {
                    let cmd = core::str::from_utf8(&buffer[0..cursor]).unwrap_or("");
                    if cmd == "help" {
                        Hdmi::write_str("Commands: help, echo\n");
                    } else if cmd.starts_with("echo ") {
                        Hdmi::write_str(&cmd[5..]);
                        Hdmi::draw_char('\n');
                    } else {
                        Hdmi::write_str("Unknown: ");
                        Hdmi::write_str(cmd);
                        Hdmi::draw_char('\n');
                    }
                }
                cursor = 0;
                Hdmi::write_str("OS> ");
            } else if c == '\x08' {
                // Backspace not visually supported yet
                if cursor > 0 { cursor -= 1; }
            } else {
                if cursor < buffer.len() {
                    buffer[cursor] = c as u8;
                    cursor += 1;
                    Hdmi::draw_char(c);
                }
            }
        }
    }
}
