use core::ptr::{read_volatile, write_volatile};

const PERIPHERAL_BASE: u32 = 0x3F00_0000;
const MAILBOX_BASE: u32 = PERIPHERAL_BASE + 0xB880;
const MAILBOX_READ: u32 = MAILBOX_BASE + 0x00;
const MAILBOX_STATUS: u32 = MAILBOX_BASE + 0x18;
const MAILBOX_WRITE: u32 = MAILBOX_BASE + 0x20;

const MAILBOX_FULL: u32 = 0x8000_0000;
const MAILBOX_EMPTY: u32 = 0x4000_0000;

#[repr(C, align(16))]
struct MailboxBuffer {
    data: [u32; 36],
}

static mut FB_PTR: *mut u32 = core::ptr::null_mut();
static mut CURSOR_X: u32 = 0;
static mut CURSOR_Y: u32 = 0;

pub struct Hdmi;

impl Hdmi {
    pub fn init() {
        let mut mb = MailboxBuffer { data: [0; 36] };
        mb.data[0] = 35 * 4;
        mb.data[1] = 0;
        
        mb.data[2] = 0x48003; 
        mb.data[3] = 8;
        mb.data[4] = 8;
        mb.data[5] = 1024;
        mb.data[6] = 768;
        
        mb.data[7] = 0x48004;
        mb.data[8] = 8;
        mb.data[9] = 8;
        mb.data[10] = 1024;
        mb.data[11] = 768;
        
        mb.data[12] = 0x48005;
        mb.data[13] = 4;
        mb.data[14] = 4;
        mb.data[15] = 32;
        
        mb.data[16] = 0x40001;
        mb.data[17] = 8;
        mb.data[18] = 8;
        mb.data[19] = 16;
        mb.data[20] = 0;
        
        mb.data[21] = 0;

        let ptr = mb.data.as_ptr() as u32;
        let channel = 8;
        
        unsafe {
            while (read_volatile(MAILBOX_STATUS as *const u32) & MAILBOX_FULL) != 0 {}
            write_volatile(MAILBOX_WRITE as *mut u32, (ptr & !0xF) | channel);
            
            loop {
                while (read_volatile(MAILBOX_STATUS as *const u32) & MAILBOX_EMPTY) != 0 {}
                let response = read_volatile(MAILBOX_READ as *const u32);
                if (response & 0xF) == channel {
                    break;
                }
            }
        }
        
        if mb.data[1] == 0x8000_0000 {
            let fb_ptr = (mb.data[19] & 0x3FFF_FFFF) as *mut u32;
            if fb_ptr as u32 != 0 {
                unsafe {
                    FB_PTR = fb_ptr;
                    CURSOR_X = 0;
                    CURSOR_Y = 0;
                    // Clear to black
                    for i in 0..(1024 * 768) {
                        write_volatile(FB_PTR.add(i), 0);
                    }
                }
            }
        }
    }

    fn scroll() {
        unsafe {
            if FB_PTR.is_null() { return; }
            let scale = 2;
            let line_height = 10 * scale as usize;
            let width = 1024;
            let height = 768;
            
            let move_size = (height - line_height) * width;
            core::ptr::copy(
                FB_PTR.add(line_height * width),
                FB_PTR,
                move_size,
            );
            
            core::ptr::write_bytes(
                FB_PTR.add(move_size),
                0,
                line_height * width,
            );
        }
    }

    pub fn draw_char(c: char) {
        unsafe {
            if FB_PTR.is_null() { return; }
        }
        
        let scale = 2;
        let char_width = 8 * scale;
        let line_height = 10 * scale;

        if c == '\n' {
            unsafe {
                CURSOR_X = 0;
                CURSOR_Y += line_height;
                if CURSOR_Y + line_height > 768 { 
                    Self::scroll();
                    CURSOR_Y -= line_height;
                }
            }
            return;
        }

        use font8x8::{BASIC_FONTS, UnicodeFonts};
        if let Some(glyph) = BASIC_FONTS.get(c) {
            unsafe {
                let get_bit = |x: i32, y: i32| -> bool {
                    if x < 0 || x > 7 || y < 0 || y > 7 { false }
                    else { (glyph[y as usize] & (1 << x)) != 0 }
                };

                for y in 0..8 {
                    for x in 0..8 {
                        let b = get_bit(x, y - 1);
                        let d = get_bit(x - 1, y);
                        let e = get_bit(x, y);
                        let f = get_bit(x + 1, y);
                        let h = get_bit(x, y + 1);

                        let mut e0 = e;
                        let mut e1 = e;
                        let mut e2 = e;
                        let mut e3 = e;

                        // Scale2x algorithm for smoother font
                        if b != h && d != f {
                            e0 = if d == b { d } else { e };
                            e1 = if b == f { f } else { e };
                            e2 = if d == h { d } else { e };
                            e3 = if h == f { f } else { e };
                        }

                        let base_x = CURSOR_X + (x as u32 * 2);
                        let base_y = CURSOR_Y + (y as u32 * 2) + 2; // +2 padding for line spacing

                        if base_y + 1 < 768 && base_x + 1 < 1024 {
                            let base_offset = base_y * 1024 + base_x;
                            if e0 { write_volatile(FB_PTR.add(base_offset as usize), 0x00FF_FFFF); }
                            if e1 { write_volatile(FB_PTR.add((base_offset + 1) as usize), 0x00FF_FFFF); }
                            if e2 { write_volatile(FB_PTR.add((base_offset + 1024) as usize), 0x00FF_FFFF); }
                            if e3 { write_volatile(FB_PTR.add((base_offset + 1025) as usize), 0x00FF_FFFF); }
                        }
                    }
                }
                CURSOR_X += char_width;
                if CURSOR_X + char_width > 1024 {
                    CURSOR_X = 0;
                    CURSOR_Y += line_height;
                    if CURSOR_Y + line_height > 768 { 
                        Self::scroll();
                        CURSOR_Y -= line_height;
                    }
                }
            }
        }
    }

    pub fn write_str(s: &str) {
        for c in s.chars() {
            Self::draw_char(c);
        }
    }
}
