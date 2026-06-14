use core::ptr::{read_volatile, write_volatile};

const MAILBOX_BASE: u32 = 0x3F00_B880;
const MAILBOX_READ: u32 = MAILBOX_BASE;
const MAILBOX_STATUS: u32 = MAILBOX_BASE + 0x18;
const MAILBOX_WRITE: u32 = MAILBOX_BASE + 0x20;

const MAILBOX_EMPTY: u32 = 0x4000_0000;
const MAILBOX_FULL: u32 = 0x8000_0000;

pub fn write_mailbox(channel: u8, data: u32) {
    loop {
        let status = unsafe { read_volatile(MAILBOX_STATUS as *const u32) };
        if (status & MAILBOX_FULL) == 0 {
            break;
        }
    }
    unsafe { write_volatile(MAILBOX_WRITE as *mut u32, (data & !0xF) | (channel as u32 & 0xF)) };
}

pub fn read_mailbox(channel: u8) -> u32 {
    loop {
        loop {
            let status = unsafe { read_volatile(MAILBOX_STATUS as *const u32) };
            if (status & MAILBOX_EMPTY) == 0 {
                break;
            }
        }
        let data = unsafe { read_volatile(MAILBOX_READ as *const u32) };
        if (data & 0xF) == (channel as u32 & 0xF) {
            return data & !0xF;
        }
    }
}

pub fn power_on_usb() -> bool {
    // 0x28001: Set power state
    // Device ID 3 is USB HCD
    #[repr(C, align(16))]
    struct Message {
        size: u32,
        code: u32,
        tag: u32,
        buffer_size: u32,
        req_res_code: u32,
        device_id: u32,
        state: u32,
        end_tag: u32,
    }

    let mut msg = Message {
        size: 32,
        code: 0,
        tag: 0x00028001,
        buffer_size: 8,
        req_res_code: 8,
        device_id: 3, // USB HCD
        state: 3,     // Wait | Power on
        end_tag: 0,
    };

    let msg_ptr = &mut msg as *mut Message as u32;
    write_mailbox(8, msg_ptr);
    read_mailbox(8);

    msg.code == 0x8000_0000 && (msg.state & 1) != 0
}
