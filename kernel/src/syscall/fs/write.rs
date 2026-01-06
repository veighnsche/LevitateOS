//! TEAM_208: Filesystem syscalls - Write operations

use crate::fs::vfs::dispatch::*;
use crate::fs::vfs::error::VfsError;
use crate::syscall::errno;
use crate::task::fd_table::FdType;
use los_hal::print;

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

    match entry.fd_type {
        FdType::Stdout | FdType::Stderr => {
            // Write to console
            if crate::memory::user::validate_user_buffer(ttbr0, buf, len, false).is_err() {
                return errno::EFAULT;
            }
            let slice = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };
            if let Ok(s) = core::str::from_utf8(slice) {
                print!("{}", s);
            } else {
                for byte in slice {
                    print!("{:02x}", byte);
                }
            }
            len as i64
        }
        FdType::VfsFile(ref file) => {
            if crate::memory::user::validate_user_buffer(ttbr0, buf, len, false).is_err() {
                return errno::EFAULT;
            }
            let mut kbuf = alloc::vec![0u8; len];
            for i in 0..len {
                if let Some(ptr) = crate::memory::user::user_va_to_kernel_ptr(ttbr0, buf + i) {
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
        _ => errno::EBADF,
    }
}
