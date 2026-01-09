use crate::memory::user as mm_user;

use crate::fs::vfs::dispatch::*;
use crate::fs::vfs::error::VfsError;
use crate::fs::vfs::file::OpenFlags;
use crate::syscall::{errno, errno_file};
use crate::task::fd_table::FdType;

/// TEAM_168: sys_openat - Open a file from initramfs.
/// TEAM_176: Updated to support opening directories for getdents.
/// TEAM_194: Updated to support tmpfs at /tmp with O_CREAT and O_TRUNC.
pub fn sys_openat(path: usize, path_len: usize, flags: u32) -> i64 {
    if path_len == 0 || path_len > 256 {
        return errno::EINVAL;
    }

    let task = crate::task::current_task();
    if mm_user::validate_user_buffer(task.ttbr0, path, path_len, false).is_err() {
        return errno::EFAULT;
    }

    let mut path_buf = [0u8; 256];
    for i in 0..path_len {
        if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, path + i) {
            path_buf[i] = unsafe { *ptr };
        } else {
            return errno::EFAULT;
        }
    }

    let path_str = match core::str::from_utf8(&path_buf[..path_len]) {
        Ok(s) => s,
        Err(_) => return errno::EINVAL,
    };

    // TEAM_247: Handle PTY devices
    if path_str == "/dev/ptmx" {
        if let Some(pair) = crate::fs::tty::pty::allocate_pty() {
            let mut fd_table = task.fd_table.lock();
            match fd_table.alloc(FdType::PtyMaster(pair)) {
                Some(fd) => return fd as i64,
                None => return errno_file::EMFILE,
            }
        }
        return errno::ENOMEM;
    }

    if path_str.starts_with("/dev/pts/") {
        if let Ok(id) = path_str[9..].parse::<usize>() {
            if let Some(pair) = crate::fs::tty::pty::get_pty(id) {
                let mut fd_table = task.fd_table.lock();
                match fd_table.alloc(FdType::PtySlave(pair)) {
                    Some(fd) => return fd as i64,
                    None => return errno_file::EMFILE,
                }
            }
        }
        return errno_file::ENOENT;
    }

    // TEAM_205: All paths now go through generic vfs_open
    let vfs_flags = OpenFlags::new(flags);
    match vfs_open(path_str, vfs_flags, 0o666) {
        Ok(file) => {
            let mut fd_table = task.fd_table.lock();
            match fd_table.alloc(FdType::VfsFile(file)) {
                Some(fd) => fd as i64,
                None => errno_file::EMFILE,
            }
        }
        Err(VfsError::NotFound) => errno_file::ENOENT,
        Err(VfsError::AlreadyExists) => errno_file::EEXIST,
        Err(VfsError::NotADirectory) => errno_file::ENOTDIR,
        Err(VfsError::IsADirectory) => {
            errno_file::EIO // Should not normally happen if vfs_open succeeded
        }
        Err(_) => errno_file::EIO,
    }
}

/// TEAM_168: sys_close - Close a file descriptor.
pub fn sys_close(fd: usize) -> i64 {
    let task = crate::task::current_task();
    let mut fd_table = task.fd_table.lock();

    if fd < 3 {
        return errno::EINVAL;
    }

    if fd_table.close(fd) { 0 } else { errno::EBADF }
}
