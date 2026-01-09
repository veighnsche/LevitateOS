use crate::fs::vfs::dispatch::*;
use crate::fs::vfs::error::VfsError;
use crate::memory::user as mm_user;
use crate::syscall::{errno, fcntl, read_user_cstring, write_to_user_buf};

/// TEAM_198: UTIME_NOW constant - set time to current time
const UTIME_NOW: u64 = 0x3FFFFFFF;
/// TEAM_198: UTIME_OMIT constant - don't change time
const UTIME_OMIT: u64 = 0x3FFFFFFE;

/// TEAM_345: sys_utimensat - Linux ABI compatible.
/// Signature: utimensat(dirfd, pathname, times, flags)
///
/// TEAM_198: Original implementation.
pub fn sys_utimensat(dirfd: i32, pathname: usize, times: usize, _flags: u32) -> i64 {
    let task = crate::task::current_task();
    
    // TEAM_345: Read null-terminated pathname (Linux ABI)
    let mut path_buf = [0u8; 4096];
    let path_str = match read_user_cstring(task.ttbr0, pathname, &mut path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    // TEAM_345: Handle dirfd
    if dirfd != fcntl::AT_FDCWD && !path_str.starts_with('/') {
        log::warn!("[SYSCALL] utimensat: dirfd {} not yet supported", dirfd);
        return errno::EBADF;
    }

    // Get current time
    let now = crate::syscall::time::uptime_seconds();

    // Determine new atime and mtime
    let (atime, mtime) = if times == 0 {
        (Some(now), Some(now))
    } else {
        // struct timespec { u64 tv_sec; u64 tv_nsec; }
        let mut times_data = [0u64; 4]; // [atime_sec, atime_nsec, mtime_sec, mtime_nsec]
        for i in 0..4 {
            let mut val = 0u64;
            for j in 0..8 {
                if let Some(ptr) =
                    mm_user::user_va_to_kernel_ptr(task.ttbr0, times + i * 8 + j)
                {
                    val |= (unsafe { *ptr } as u64) << (j * 8);
                } else {
                    return errno::EFAULT;
                }
            }
            times_data[i] = val;
        }

        let atime = if times_data[1] == UTIME_OMIT {
            None
        } else if times_data[1] == UTIME_NOW {
            Some(now)
        } else {
            Some(times_data[0])
        };
        let mtime = if times_data[3] == UTIME_OMIT {
            None
        } else if times_data[3] == UTIME_NOW {
            Some(now)
        } else {
            Some(times_data[2])
        };
        (atime, mtime)
    };

    vfs_utimes(path_str, atime, mtime)
        .map(|_| 0)
        .unwrap_or_else(|e| e.to_errno())
}

/// TEAM_345: sys_linkat - Linux ABI compatible.
/// Signature: linkat(olddirfd, oldpath, newdirfd, newpath, flags)
pub fn sys_linkat(
    olddirfd: i32,
    oldpath: usize,
    newdirfd: i32,
    newpath: usize,
    _flags: u32,
) -> i64 {
    let task = crate::task::current_task();
    
    // TEAM_345: Read null-terminated oldpath
    let mut old_path_buf = [0u8; 4096];
    let old_path_str = match read_user_cstring(task.ttbr0, oldpath, &mut old_path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    // TEAM_345: Read null-terminated newpath
    let mut new_path_buf = [0u8; 4096];
    let new_path_str = match read_user_cstring(task.ttbr0, newpath, &mut new_path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    // TEAM_345: Handle dirfd
    if (olddirfd != fcntl::AT_FDCWD && !old_path_str.starts_with('/'))
        || (newdirfd != fcntl::AT_FDCWD && !new_path_str.starts_with('/')) {
        log::warn!("[SYSCALL] linkat: dirfd not yet supported");
        return errno::EBADF;
    }

    match vfs_link(old_path_str, new_path_str) {
        Ok(()) => 0,
        Err(e) => e.to_errno() as i64,
    }
}

/// TEAM_345: sys_symlinkat - Linux ABI compatible.
/// Signature: symlinkat(target, newdirfd, linkpath)
pub fn sys_symlinkat(
    target: usize,
    newdirfd: i32,
    linkpath: usize,
) -> i64 {
    let task = crate::task::current_task();
    
    // TEAM_345: Read null-terminated target
    let mut target_buf = [0u8; 4096];
    let target_str = match read_user_cstring(task.ttbr0, target, &mut target_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    // TEAM_345: Read null-terminated linkpath
    let mut linkpath_buf = [0u8; 4096];
    let linkpath_str = match read_user_cstring(task.ttbr0, linkpath, &mut linkpath_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    // TEAM_345: Handle dirfd
    if newdirfd != fcntl::AT_FDCWD && !linkpath_str.starts_with('/') {
        log::warn!("[SYSCALL] symlinkat: dirfd {} not yet supported", newdirfd);
        return errno::EBADF;
    }

    match vfs_symlink(target_str, linkpath_str) {
        Ok(()) => 0,
        Err(VfsError::AlreadyExists) => errno::EEXIST,
        Err(VfsError::NotFound) => errno::ENOENT,
        Err(VfsError::NotADirectory) => errno::ENOTDIR,
        Err(_) => errno::EINVAL,
    }
}

/// TEAM_345: sys_readlinkat - Linux ABI compatible.
/// Signature: readlinkat(dirfd, pathname, buf, bufsiz)
pub fn sys_readlinkat(dirfd: i32, pathname: usize, buf: usize, bufsiz: usize) -> i64 {
    let task = crate::task::current_task();
    
    // TEAM_345: Read null-terminated pathname
    let mut path_buf = [0u8; 4096];
    let path_str = match read_user_cstring(task.ttbr0, pathname, &mut path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    // TEAM_345: Handle dirfd
    if dirfd != fcntl::AT_FDCWD && !path_str.starts_with('/') {
        log::warn!("[SYSCALL] readlinkat: dirfd {} not yet supported", dirfd);
        return errno::EBADF;
    }
    
    let buf_len = bufsiz;

    match vfs_readlink(path_str) {
        Ok(target) => {
            let n = target.len().min(buf_len);
            let target_bytes = target.as_bytes();
            for i in 0..n {
                if !write_to_user_buf(task.ttbr0, buf, i, target_bytes[i]) {
                    return errno::EFAULT;
                }
            }
            n as i64
        }
        Err(VfsError::NotFound) => errno::ENOENT,
        Err(_) => errno::EIO,
    }
}
