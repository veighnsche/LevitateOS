use crate::memory::user as mm_user;

use crate::fs::vfs::dispatch::*;
use crate::fs::vfs::error::VfsError;
use crate::syscall::{errno, write_to_user_buf};
use crate::task::fd_table::FdType;

/// TEAM_217: sys_readv - Vectored read.
pub fn sys_readv(fd: usize, iov_ptr: usize, count: usize) -> i64 {
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

    let mut total_read: i64 = 0;

    for i in 0..count {
        let entry_addr = iov_ptr + i * core::mem::size_of::<UserIoVec>();
        let iov = unsafe {
            let kptr = mm_user::user_va_to_kernel_ptr(ttbr0, entry_addr).unwrap();
            *(kptr as *const UserIoVec)
        };

        if iov.len == 0 {
            continue;
        }

        let res = sys_read(fd, iov.base, iov.len);
        if res < 0 {
            if total_read == 0 {
                return res;
            } else {
                return total_read;
            }
        }
        total_read += res;
        if res < iov.len as i64 {
            // Short read, stop here
            break;
        }
    }

    total_read
}

/// TEAM_217: struct iovec for writev/readv
#[repr(C)]
#[derive(Clone, Copy)]
struct UserIoVec {
    base: usize,
    len: usize,
}

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
        FdType::Stdin => {
            #[cfg(feature = "verbose-syscalls")]
            los_hal::println!("[SYS_READ] fd={} type=Stdin len={}", fd, len);
            read_stdin(buf, len, ttbr0)
        }
        FdType::VfsFile(ref file) => {
            #[cfg(feature = "verbose-syscalls")]
            los_hal::println!("[SYS_READ] fd={} type=VfsFile len={}", fd, len);
            if mm_user::validate_user_buffer(ttbr0, buf, len, true).is_err() {
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
        FdType::PipeRead(ref pipe) => {
            #[cfg(feature = "verbose-syscalls")]
            los_hal::println!("[SYS_READ] fd={} type=PipeRead len={}", fd, len);
            if mm_user::validate_user_buffer(ttbr0, buf, len, true).is_err() {
                return errno::EFAULT;
            }
            let mut kbuf = alloc::vec![0u8; len];
            let result = pipe.read(&mut kbuf);
            if result < 0 {
                return result as i64;
            }
            let n = result as usize;
            for i in 0..n {
                if !write_to_user_buf(ttbr0, buf, i, kbuf[i]) {
                    return errno::EFAULT;
                }
            }
            n as i64
        }
        FdType::PtyMaster(ref pair) => {
            #[cfg(feature = "verbose-syscalls")]
            los_hal::println!("[SYS_READ] fd={} type=PtyMaster len={}", fd, len);
            let mut bytes_read = 0;
            loop {
                let mut buffer = pair.master_read_buffer.lock();
                if let Some(byte) = buffer.pop_front() {
                    if !write_to_user_buf(ttbr0, buf, bytes_read, byte) {
                        return errno::EFAULT;
                    }
                    bytes_read += 1;
                    if bytes_read >= len {
                        return bytes_read as i64;
                    }
                    continue; // Check for more
                }

                if bytes_read > 0 {
                    return bytes_read as i64;
                }
                drop(buffer);

                // Wait for output
                unsafe {
                    los_hal::interrupts::enable();
                }
                let _ = los_hal::interrupts::disable();
                crate::task::yield_now();
            }
        }
        FdType::PtySlave(ref pair) => {
            #[cfg(feature = "verbose-syscalls")]
            los_hal::println!("[SYS_READ] fd={} type=PtySlave len={}", fd, len);
            read_from_tty(&pair.tty, buf, len, ttbr0, false)
        }
        _ => {
            #[cfg(feature = "verbose-syscalls")]
            los_hal::println!("[SYS_READ] fd={} type=Other(EBADF?)", fd);
            errno::EBADF
        }
    }
}

/// TEAM_178: Read from stdin (keyboard/console input).
/// TEAM_247: Refactored to use TTY line discipline.
fn read_stdin(buf: usize, len: usize, ttbr0: usize) -> i64 {
    use crate::fs::tty::CONSOLE_TTY;
    read_from_tty(&CONSOLE_TTY, buf, len, ttbr0, true)
}

fn read_from_tty(
    tty_mutex: &los_utils::Mutex<crate::fs::tty::TtyState>,
    buf: usize,
    len: usize,
    ttbr0: usize,
    is_console: bool,
) -> i64 {
    let max_read = len.min(4096);
    if mm_user::validate_user_buffer(ttbr0, buf, max_read, true).is_err() {
        return errno::EFAULT;
    }

    loop {
        // 1. Feed hardware input into TTY (only for console)
        if is_console {
            poll_to_tty();
        }

        // 2. Try to satisfy read from TTY input buffer
        let mut tty = tty_mutex.lock();
        if !tty.input_buffer.is_empty() {
            #[cfg(feature = "verbose-syscalls")]
            los_hal::println!("[TTY] Input buffer has {} bytes", tty.input_buffer.len());

            let mut bytes_read = 0;
            while bytes_read < max_read {
                if let Some(byte) = tty.input_buffer.pop_front() {
                    if !write_to_user_buf(ttbr0, buf, bytes_read, byte) {
                        return errno::EFAULT;
                    }
                    bytes_read += 1;

                    // In canonical mode, read() returns at most one line per call
                    if (tty.termios.c_lflag & crate::fs::tty::ICANON) != 0 && byte == b'\n' {
                        return bytes_read as i64;
                    }
                } else {
                    break;
                }
            }
            return bytes_read as i64;
        }
        drop(tty);

        // 3. Wait/Yield if nothing available
        // #[cfg(feature = "verbose-syscalls")]
        // los_hal::println!("[TTY] Waiting for input..."); // Too noisy for loop

        unsafe {
            los_hal::interrupts::enable();
        }
        let _ = los_hal::interrupts::disable();

        crate::task::yield_now();
    }
}

fn poll_to_tty() {
    use crate::fs::tty::CONSOLE_TTY;

    crate::input::poll();

    while let Some(ch) = crate::input::read_char() {
        CONSOLE_TTY.lock().process_input(ch as u8);
    }

    while let Some(byte) = los_hal::console::read_byte() {
        CONSOLE_TTY.lock().process_input(byte);
    }
}
