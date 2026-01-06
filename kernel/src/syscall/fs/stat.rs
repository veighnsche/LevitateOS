//! TEAM_208: Filesystem syscalls - File status operations

use crate::fs::vfs::dispatch::*;
use crate::syscall::{Stat, errno};
use crate::task::fd_table::FdType;

/// TEAM_168: sys_fstat - Get file status.
pub fn sys_fstat(fd: usize, stat_buf: usize) -> i64 {
    let task = crate::task::current_task();
    let stat_size = core::mem::size_of::<Stat>();
    if crate::memory::user::validate_user_buffer(task.ttbr0, stat_buf, stat_size, true).is_err() {
        return errno::EFAULT;
    }

    let fd_table = task.fd_table.lock();
    let entry = match fd_table.get(fd) {
        Some(e) => e,
        None => return errno::EBADF,
    };

    let stat = match entry.fd_type {
        // TEAM_201: Updated to use extended Stat struct
        FdType::Stdin | FdType::Stdout | FdType::Stderr => Stat {
            st_dev: 0,
            st_ino: 0,
            st_mode: crate::fs::mode::S_IFCHR | 0o666,
            st_nlink: 1,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_atime: 0,
            st_atime_nsec: 0,
            st_mtime: 0,
            st_mtime_nsec: 0,
            st_ctime: 0,
            st_ctime_nsec: 0,
        },
        FdType::VfsFile(ref file) => match vfs_fstat(file) {
            Ok(s) => s,
            Err(_) => return errno::EBADF,
        },
    };

    let stat_bytes =
        unsafe { core::slice::from_raw_parts(&stat as *const Stat as *const u8, stat_size) };

    for (i, &byte) in stat_bytes.iter().enumerate() {
        if let Some(ptr) = crate::memory::user::user_va_to_kernel_ptr(task.ttbr0, stat_buf + i) {
            unsafe { *ptr = byte };
        } else {
            return errno::EFAULT;
        }
    }

    0
}
