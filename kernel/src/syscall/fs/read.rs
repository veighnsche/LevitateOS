//! TEAM_208: Filesystem syscalls - Read operations

use crate::fs::vfs::dispatch::*;
use crate::fs::vfs::error::VfsError;
use crate::syscall::{errno, write_to_user_buf};
use crate::task::fd_table::FdType;

/// TEAM_081: sys_read - Read from a file descriptor.
/// TEAM_178: Refactored to dispatch by fd type, added InitramfsFile support.
pub fn sys_read(fd: usize, buf: usize, len: usize) -> i64 {
    if len == 0 {
        return 0;
    }

    let task = crate::task::current_task();

    // TEAM_178: Look up fd type and dispatch accordingly
    let fd_table = task.fd_table.lock();
    let entry = match fd_table.get(fd) {
        Some(e) => e.clone(),
        None => return errno::EBADF,
    };
    drop(fd_table);

    let ttbr0 = task.ttbr0;

    match entry.fd_type {
        FdType::Stdin => read_stdin(buf, len, ttbr0),
        FdType::VfsFile(ref file) => {
            if crate::memory::user::validate_user_buffer(ttbr0, buf, len, true).is_err() {
                return errno::EFAULT;
            }
            let mut kbuf = alloc::vec![0u8; len];
            match vfs_read(file, &mut kbuf) {
                Ok(n) => {
                    for i in 0..n {
                        if !write_to_user_buf(ttbr0, buf, i, kbuf[i]) {
                            return errno::EFAULT;
                        }
                    }
                    n as i64
                }
                Err(VfsError::BadFd) => errno::EBADF,
                Err(_) => errno::EIO,
            }
        }
        _ => errno::EBADF,
    }
}

/// TEAM_178: Read from stdin (keyboard/console input).
fn read_stdin(buf: usize, len: usize, ttbr0: usize) -> i64 {
    let max_read = len.min(4096);
    if crate::memory::user::validate_user_buffer(ttbr0, buf, max_read, true).is_err() {
        return errno::EFAULT;
    }

    let mut bytes_read = 0usize;

    loop {
        poll_input_devices(ttbr0, buf, &mut bytes_read, max_read);
        if bytes_read > 0 {
            break;
        }

        unsafe {
            los_hal::interrupts::enable();
        }
        let _ = los_hal::interrupts::disable();

        crate::task::yield_now();
    }

    bytes_read as i64
}

fn poll_input_devices(ttbr0: usize, user_buf: usize, bytes_read: &mut usize, max_read: usize) {
    crate::input::poll();

    while *bytes_read < max_read {
        if let Some(ch) = crate::input::read_char() {
            if !write_to_user_buf(ttbr0, user_buf, *bytes_read, ch as u8) {
                return;
            }
            *bytes_read += 1;
            if ch == '\n' {
                return;
            }
        } else {
            break;
        }
    }

    if *bytes_read < max_read {
        while let Some(byte) = los_hal::console::read_byte() {
            let byte = if byte == b'\r' { b'\n' } else { byte };
            if !write_to_user_buf(ttbr0, user_buf, *bytes_read, byte) {
                return;
            }
            *bytes_read += 1;
            if byte == b'\n' {
                return;
            }
            if *bytes_read >= max_read {
                return;
            }
        }
    }
}
