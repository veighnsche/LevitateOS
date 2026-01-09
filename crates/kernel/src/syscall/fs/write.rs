use crate::memory::user as mm_user;

use crate::fs::vfs::dispatch::*;
use crate::fs::vfs::error::VfsError;
use crate::syscall::errno;
use crate::task::fd_table::FdType;

/// TEAM_217: struct iovec for writev/readv
#[repr(C)]
#[derive(Clone, Copy)]
struct UserIoVec {
    base: usize,
    len: usize,
}

/// TEAM_217: sys_writev - Vectored write.
/// Required for standard Rust println! efficiency.
pub fn sys_writev(fd: usize, iov_ptr: usize, count: usize) -> i64 {
    if count == 0 {
        return 0;
    }
    if count > 1024 {
        return errno::EINVAL;
    }

    let task = crate::task::current_task();
    let ttbr0 = task.ttbr0;

    // Validate iovec array
    let iov_size = count * core::mem::size_of::<UserIoVec>();
    if mm_user::validate_user_buffer(ttbr0, iov_ptr, iov_size, false).is_err() {
        return errno::EFAULT;
    }

    let mut total_written: i64 = 0;

    for i in 0..count {
        let entry_addr = iov_ptr + i * core::mem::size_of::<UserIoVec>();
        let iov = unsafe {
            let kptr = mm_user::user_va_to_kernel_ptr(ttbr0, entry_addr).unwrap();
            *(kptr as *const UserIoVec)
        };

        if iov.len == 0 {
            continue;
        }

        let res = sys_write(fd, iov.base, iov.len);
        if res < 0 {
            if total_written == 0 {
                return res;
            } else {
                return total_written;
            }
        }
        total_written += res;
    }

    total_written
}

/// TEAM_073: sys_write - Write to a file descriptor.
/// TEAM_194: Updated to support writing to tmpfs files.
pub fn sys_write(fd: usize, buf: usize, len: usize) -> i64 {
    let len = len.min(4096);
    let task = crate::task::current_task();

    // TEAM_194: Look up fd type and dispatch accordingly
    let fd_table = task.fd_table.lock();
    let entry = match fd_table.get(fd) {
        Some(e) => e.clone(),
        None => return errno::EBADF,
    };
    drop(fd_table);

    let ttbr0 = task.ttbr0;
    // los_hal::println!("[SYS_WRITE] buf={:x}, len={}", buf, len);
    match entry.fd_type {
        FdType::Stdout | FdType::Stderr => {
            let n = write_to_tty(&crate::fs::tty::CONSOLE_TTY, buf, len, ttbr0, true, None);
            if n < 0 {
                los_hal::println!("[SYS_WRITE] FAILED: {}", n);
            }
            n
        }
        FdType::PtyMaster(ref pair) => {
            if mm_user::validate_user_buffer(ttbr0, buf, len, false).is_err() {
                return errno::EFAULT;
            }
            let mut kbuf = alloc::vec![0u8; len];
            for i in 0..len {
                if let Some(ptr) = mm_user::user_va_to_kernel_ptr(ttbr0, buf + i) {
                    kbuf[i] = unsafe { *ptr };
                } else {
                    return errno::EFAULT;
                }
            }

            for &byte in &kbuf {
                pair.tty.lock().process_input(byte);
            }
            len as i64
        }
        FdType::PtySlave(ref pair) => write_to_tty(
            &pair.tty,
            buf,
            len,
            ttbr0,
            false,
            Some(pair.master_read_buffer.clone()),
        ),
        FdType::VfsFile(ref file) => {
            if mm_user::validate_user_buffer(ttbr0, buf, len, false).is_err() {
                return errno::EFAULT;
            }
            let mut kbuf = alloc::vec![0u8; len];
            for i in 0..len {
                if let Some(ptr) = mm_user::user_va_to_kernel_ptr(ttbr0, buf + i) {
                    kbuf[i] = unsafe { *ptr };
                } else {
                    return errno::EFAULT;
                }
            }
            match vfs_write(file, &kbuf) {
                Ok(n) => n as i64,
                Err(VfsError::NoSpace) => -28,      // ENOSPC
                Err(VfsError::FileTooLarge) => -27, // EFBIG
                Err(_) => errno::EIO,
            }
        }
        // TEAM_233: Write to pipe
        FdType::PipeWrite(ref pipe) => {
            if mm_user::validate_user_buffer(ttbr0, buf, len, false).is_err() {
                return errno::EFAULT;
            }
            let mut kbuf = alloc::vec![0u8; len];
            for i in 0..len {
                if let Some(ptr) = mm_user::user_va_to_kernel_ptr(ttbr0, buf + i) {
                    kbuf[i] = unsafe { *ptr };
                } else {
                    return errno::EFAULT;
                }
            }
            let result = pipe.write(&kbuf);
            if result < 0 {
                return result as i64;
            }
            result as i64
        }
        _ => errno::EBADF,
    }
}

fn write_to_tty(
    tty_mutex: &los_utils::Mutex<crate::fs::tty::TtyState>,
    buf: usize,
    len: usize,
    ttbr0: usize,
    is_console: bool,
    master_buffer: Option<alloc::sync::Arc<los_utils::Mutex<alloc::collections::VecDeque<u8>>>>,
) -> i64 {
    use crate::fs::tty::{ONLCR, OPOST};

    if mm_user::validate_user_buffer(ttbr0, buf, len, false).is_err() {
        return errno::EFAULT;
    }

    let mut kbuf = alloc::vec![0u8; len];
    for i in 0..len {
        if let Some(ptr) = mm_user::user_va_to_kernel_ptr(ttbr0, buf + i) {
            kbuf[i] = unsafe { *ptr };
        } else {
            return errno::EFAULT;
        }
    }

    loop {
        let stopped = tty_mutex.lock().stopped;
        if !stopped {
            break;
        }

        unsafe {
            los_hal::interrupts::enable();
        }
        let _ = los_hal::interrupts::disable();
        crate::task::yield_now();
    }

    let tty = tty_mutex.lock();
    let oflag = tty.termios.c_oflag;
    drop(tty);

    for &byte in &kbuf {
        if (oflag & OPOST) != 0 && byte == b'\n' && (oflag & ONLCR) != 0 {
            if is_console {
                los_hal::print!("\r\n");
            } else if let Some(ref buffer) = master_buffer {
                let mut b = buffer.lock();
                b.push_back(b'\r');
                b.push_back(b'\n');
            }
        } else {
            if is_console {
                los_hal::print!("{}", byte as char);
            } else if let Some(ref buffer) = master_buffer {
                buffer.lock().push_back(byte);
            }
        }
    }

    len as i64
}
