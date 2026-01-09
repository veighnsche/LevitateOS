use crate::memory::user as mm_user;

use crate::fs::vfs::dispatch::*;
use crate::fs::vfs::error::VfsError;
use crate::syscall::{errno, errno_file, write_to_user_buf};

/// TEAM_198: UTIME_NOW constant - set time to current time
const UTIME_NOW: u64 = 0x3FFFFFFF;
/// TEAM_198: UTIME_OMIT constant - don't change time
const UTIME_OMIT: u64 = 0x3FFFFFFE;

/// TEAM_198: sys_utimensat - Set file access and modification times.
///
/// # Arguments
/// * `_dirfd` - Directory fd (AT_FDCWD for cwd) - currently ignored
/// * `path` - Path to file
/// * `path_len` - Length of path
/// * `times` - Pointer to [atime, mtime] timespec array (0 = use current time)
/// * `_flags` - AT_SYMLINK_NOFOLLOW - currently ignored
pub fn sys_utimensat(_dirfd: i32, path: usize, path_len: usize, times: usize, _flags: u32) -> i64 {
    let task = crate::task::current_task();
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

/// TEAM_209: sys_linkat - Create a hard link.
pub fn sys_linkat(
    _olddirfd: i32,
    oldpath: usize,
    oldpath_len: usize,
    _newdirfd: i32,
    newpath: usize,
    newpath_len: usize,
    _flags: u32,
) -> i64 {
    let task = crate::task::current_task();
    
    // Resolve oldpath
    let mut old_path_buf = [0u8; 256];
    let old_path_str = match crate::syscall::copy_user_string(task.ttbr0, oldpath, oldpath_len, &mut old_path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };

    // Resolve newpath
    let mut new_path_buf = [0u8; 256];
    let new_path_str = match crate::syscall::copy_user_string(task.ttbr0, newpath, newpath_len, &mut new_path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };

    match vfs_link(old_path_str, new_path_str) {
        Ok(()) => 0,
        Err(e) => e.to_errno() as i64,
    }
}

/// TEAM_198: sys_symlinkat - Create a symbolic link.
///
/// # Arguments
/// * `target` - Target path the symlink points to
/// * `target_len` - Length of target
/// * `_linkdirfd` - Directory fd for link path (ignored, use AT_FDCWD)
/// * `linkpath` - Path for the new symlink
/// * `linkpath_len` - Length of link path
pub fn sys_symlinkat(
    target: usize,
    target_len: usize,
    _linkdirfd: i32,
    linkpath: usize,
    linkpath_len: usize,
) -> i64 {
    let task = crate::task::current_task();
    let mut target_buf = [0u8; 256];
    for i in 0..target_len {
        if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, target + i) {
            target_buf[i] = unsafe { *ptr };
        } else {
            return errno::EFAULT;
        }
    }
    let target_str = match core::str::from_utf8(&target_buf[..target_len]) {
        Ok(s) => s,
        Err(_) => return errno::EINVAL,
    };

    let mut linkpath_buf = [0u8; 256];
    for i in 0..linkpath_len {
        if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, linkpath + i) {
            linkpath_buf[i] = unsafe { *ptr };
        } else {
            return errno::EFAULT;
        }
    }
    let linkpath_str = match core::str::from_utf8(&linkpath_buf[..linkpath_len]) {
        Ok(s) => s,
        Err(_) => return errno::EINVAL,
    };

    match vfs_symlink(target_str, linkpath_str) {
        Ok(()) => 0,
        Err(VfsError::AlreadyExists) => -17, // EEXIST
        Err(VfsError::NotFound) => errno_file::ENOENT,
        Err(VfsError::NotADirectory) => errno_file::ENOTDIR,
        Err(_) => errno::EINVAL,
    }
}

/// TEAM_204: sys_readlinkat - Read value of a symbolic link.
pub fn sys_readlinkat(_dirfd: i32, path: usize, path_len: usize, buf: usize, buf_len: usize) -> i64 {
    let task = crate::task::current_task();
    let mut path_buf = [0u8; 256];
    let path_str = match crate::syscall::copy_user_string(task.ttbr0, path, path_len, &mut path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };

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
        Err(VfsError::NotFound) => errno_file::ENOENT,
        Err(_) => errno::EIO,
    }
}
