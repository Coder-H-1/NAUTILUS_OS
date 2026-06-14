use crate::kernel::process::TrapFrame;

pub fn handle_syscall(tf: &mut TrapFrame) {
    let syscall_num = tf.regs[8];
    match syscall_num {
        1 => { // SYS_EXIT
            sys_exit(tf.regs[0] as i32);
        },
        2 => { // SYS_WRITE
            let res = sys_write(tf.regs[0] as u32, tf.regs[1] as *const u8, tf.regs[2] as usize);
            tf.regs[0] = res as u64; // Return value
        },
        _ => {}
    }
}

pub fn sys_write(_fd: u32, _buf: *const u8, _len: usize) -> isize {
    // Write logic
    0
}

pub fn sys_exit(_code: i32) {
    // Terminate
}

