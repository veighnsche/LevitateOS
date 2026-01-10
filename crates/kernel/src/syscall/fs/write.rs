use crate::memory::user as mm_user;

use crate::fs::vfs::dispatch::*;
use crate::fs::vfs::error::VfsError;
// TEAM_420: Direct linux_raw_sys imports, no shims
// TEAM_421: Return SyscallResult directly - NO ERRNO CASTS
use crate::syscall::SyscallResult;
use crate::task::fd_table::FdType;
use linux_raw_sys::errno::{EBADF, EFAULT, EFBIG, EINVAL, EIO, ENOSPC};

/// TEAM_217: struct iovec for writev/readv
#[repr(C)]
#[derive(Clone, Copy)]
struct UserIoVec {
    base: usize,
    len: usize,
}

/// TEAM_217: sys_writev - Vectored write.
/// TEAM_421: Returns SyscallResult directly - NO ERRNO CASTS.
/// Required for standard Rust println! efficiency.
pub fn sys_writev(fd: usize, iov_ptr: usize, count: usize) -> SyscallResult {
    if count == 0 {
        return Ok(0);
    }
    if count > 1024 {
        return Err(EINVAL);
    }

    let task = crate::task::current_task();
    let ttbr0 = task.ttbr0;

    // Validate iovec array
    let iov_size = count * core::mem::size_of::<UserIoVec>();
    if mm_user::validate_user_buffer(ttbr0, iov_ptr, iov_size, false).is_err() {
        return Err(EFAULT);
    }

    let mut total_written: i64 = 0;

    for i in 0..count {
        let entry_addr = iov_ptr + i * core::mem::size_of::<UserIoVec>();
        let iov = unsafe {
            let kptr = match mm_user::user_va_to_kernel_ptr(ttbr0, entry_addr) {
                Some(p) => p,
                None => return Err(EFAULT),
            };
            *(kptr as *const UserIoVec)
        };

        if iov.len == 0 {
            continue;
        }

        // TEAM_421: Call sys_write which returns SyscallResult directly
        match sys_write(fd, iov.base, iov.len) {
            Ok(n) => {
                total_written += n;
            }
            Err(e) => {
                if total_written == 0 {
                    return Err(e);
                } else {
                    return Ok(total_written);
                }
            }
        }
    }

    Ok(total_written)
}

/// TEAM_073: sys_write - Write to a file descriptor.
/// TEAM_194: Updated to support writing to tmpfs files.
/// TEAM_421: Returns SyscallResult directly - NO ERRNO CASTS.
pub fn sys_write(fd: usize, buf: usize, len: usize) -> SyscallResult {
    let len = len.min(4096);
    let task = crate::task::current_task();

    // TEAM_194: Look up fd type and dispatch accordingly
    let fd_table = task.fd_table.lock();
    let entry = match fd_table.get(fd) {
        Some(e) => e.clone(),
        None => return Err(EBADF),
    };
    drop(fd_table);

    let ttbr0 = task.ttbr0;
    match entry.fd_type {
        FdType::Stdout | FdType::Stderr => {
            write_to_tty(&crate::fs::tty::CONSOLE_TTY, buf, len, ttbr0, true, None)
        }
        FdType::PtyMaster(ref pair) => {
            if mm_user::validate_user_buffer(ttbr0, buf, len, false).is_err() {
                return Err(EFAULT);
            }
            let mut kbuf = alloc::vec![0u8; len];
            let src = match mm_user::user_va_to_kernel_ptr(ttbr0, buf) {
                Some(p) => p,
                None => return Err(EFAULT),
            };
            unsafe {
                core::ptr::copy_nonoverlapping(src, kbuf.as_mut_ptr(), len);
            }

            for &byte in &kbuf {
                pair.tty.lock().process_input(byte);
            }
            Ok(len as i64)
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
                return Err(EFAULT);
            }
            let mut kbuf = alloc::vec![0u8; len];
            let src = match mm_user::user_va_to_kernel_ptr(ttbr0, buf) {
                Some(p) => p,
                None => return Err(EFAULT),
            };
            unsafe {
                core::ptr::copy_nonoverlapping(src, kbuf.as_mut_ptr(), len);
            }
            match vfs_write(file, &kbuf) {
                Ok(n) => Ok(n as i64),
                Err(VfsError::NoSpace) => Err(ENOSPC),
                Err(VfsError::FileTooLarge) => Err(EFBIG),
                Err(_) => Err(EIO),
            }
        }
        // TEAM_233: Write to pipe
        // TEAM_421: Pipe now returns Result<usize, u32> directly
        FdType::PipeWrite(ref pipe) => {
            if mm_user::validate_user_buffer(ttbr0, buf, len, false).is_err() {
                return Err(EFAULT);
            }
            let mut kbuf = alloc::vec![0u8; len];
            let src = match mm_user::user_va_to_kernel_ptr(ttbr0, buf) {
                Some(p) => p,
                None => return Err(EFAULT),
            };
            unsafe {
                core::ptr::copy_nonoverlapping(src, kbuf.as_mut_ptr(), len);
            }
            let n = pipe.write(&kbuf)?;
            Ok(n as i64)
        }
        _ => Err(EBADF),
    }
}

/// TEAM_421: TTY write implementation returning SyscallResult directly.
fn write_to_tty(
    tty_mutex: &los_utils::Mutex<crate::fs::tty::TtyState>,
    buf: usize,
    len: usize,
    ttbr0: usize,
    is_console: bool,
    master_buffer: Option<alloc::sync::Arc<los_utils::Mutex<alloc::collections::VecDeque<u8>>>>,
) -> SyscallResult {
    use crate::fs::tty::{ONLCR, OPOST};

    if mm_user::validate_user_buffer(ttbr0, buf, len, false).is_err() {
        return Err(EFAULT);
    }

    let mut kbuf = alloc::vec![0u8; len];
    let src = match mm_user::user_va_to_kernel_ptr(ttbr0, buf) {
        Some(p) => p,
        None => return Err(EFAULT),
    };
    unsafe {
        core::ptr::copy_nonoverlapping(src, kbuf.as_mut_ptr(), len);
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

    Ok(len as i64)
}
