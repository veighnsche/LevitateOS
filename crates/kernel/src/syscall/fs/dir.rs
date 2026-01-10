use crate::memory::user as mm_user;

use crate::fs::vfs::dispatch::*;
use crate::fs::vfs::error::VfsError;
// TEAM_420: Direct linux_raw_sys import, no shims
// TEAM_421: Import SyscallResult
use crate::syscall::{write_to_user_buf, SyscallResult};
use crate::task::fd_table::FdType;
use linux_raw_sys::errno::{EBADF, EEXIST, EFAULT, EINVAL, ENOENT, ENOTEMPTY, ENOTDIR, ERANGE, EXDEV};
use linux_raw_sys::general::{AT_FDCWD, AT_REMOVEDIR};

// TEAM_176: Dirent64 structure for getdents syscall.
// Matches Linux ABI layout.
#[repr(C, packed)]
struct Dirent64 {
    d_ino: u64,    // Inode number
    d_off: i64,    // Offset to next entry
    d_reclen: u16, // Length of this record
    d_type: u8,    // File type
                   // d_name follows (null-terminated)
}

/// TEAM_421: Updated to return SyscallResult.
pub fn sys_getdents(fd: usize, buf: usize, buf_len: usize) -> SyscallResult {
    if buf_len == 0 {
        return Ok(0);
    }

    let task = crate::task::current_task();
    if mm_user::validate_user_buffer(task.ttbr0, buf, buf_len, true).is_err() {
        return Err(EFAULT);
    }

    let fd_table = task.fd_table.lock();
    let entry = match fd_table.get(fd) {
        Some(e) => e.clone(),
        None => return Err(EBADF),
    };
    drop(fd_table);

    match entry.fd_type {
        FdType::VfsFile(ref file) => {
            let mut bytes_written = 0usize;
            loop {
                let offset = file.tell() as usize;
                match vfs_readdir(file, offset) {
                    Ok(Some(entry)) => {
                        let name_bytes = entry.name.as_bytes();
                        let name_len = name_bytes.len();
                        let reclen = ((19 + name_len + 1 + 7) / 8) * 8;

                        if bytes_written + reclen > buf_len {
                            break;
                        }

                        let dtype = match entry.file_type {
                            crate::fs::mode::S_IFDIR => 4,
                            crate::fs::mode::S_IFREG => 8,
                            crate::fs::mode::S_IFLNK => 10,
                            _ => 0,
                        };

                        let dirent = Dirent64 {
                            d_ino: entry.ino,
                            d_off: (offset + 1) as i64,
                            d_reclen: reclen as u16,
                            d_type: dtype,
                        };

                        let dirent_bytes = unsafe {
                            core::slice::from_raw_parts(
                                &dirent as *const Dirent64 as *const u8,
                                core::mem::size_of::<Dirent64>(),
                            )
                        };

                        for (i, &byte) in dirent_bytes.iter().enumerate() {
                            if !write_to_user_buf(task.ttbr0, buf, bytes_written + i, byte) {
                                return Err(EFAULT);
                            }
                        }

                        let name_offset = bytes_written + core::mem::size_of::<Dirent64>();
                        for (i, &byte) in name_bytes.iter().enumerate() {
                            if !write_to_user_buf(task.ttbr0, buf, name_offset + i, byte) {
                                return Err(EFAULT);
                            }
                        }

                        if !write_to_user_buf(task.ttbr0, buf, name_offset + name_len, 0) {
                            return Err(EFAULT);
                        }

                        let _ =
                            file.seek((offset + 1) as i64, crate::fs::vfs::ops::SeekWhence::Set);
                        bytes_written += reclen;
                    }
                    Ok(None) => break,
                    Err(_) => return Err(EBADF),
                }
            }
            Ok(bytes_written as i64)
        }
        _ => Err(ENOTDIR),
    }
}

/// TEAM_421: Updated to return SyscallResult.
pub fn sys_getcwd(buf: usize, size: usize) -> SyscallResult {
    let task = crate::task::current_task();
    if mm_user::validate_user_buffer(task.ttbr0, buf, size, true).is_err() {
        return Err(EFAULT);
    }

    let cwd_lock = task.cwd.lock();
    let path = cwd_lock.as_str();
    let path_len = path.len();
    if size < path_len + 1 {
        return Err(ERANGE);
    }

    // TEAM_416: Replace unwrap() with proper error handling for panic safety
    let dest = match mm_user::user_va_to_kernel_ptr(task.ttbr0, buf) {
        Some(p) => p,
        None => return Err(EFAULT),
    };
    unsafe {
        core::ptr::copy_nonoverlapping(path.as_bytes().as_ptr(), dest, path_len);
        *dest.add(path_len) = 0; // null terminator
    }

    Ok((path_len + 1) as i64)
}

/// TEAM_345: sys_mkdirat - Linux ABI compatible.
/// TEAM_421: Updated to return SyscallResult.
/// Signature: mkdirat(dirfd, pathname, mode)
pub fn sys_mkdirat(dirfd: i32, pathname: usize, mode: u32) -> SyscallResult {
    let task = crate::task::current_task();

    // TEAM_418: Use PATH_MAX from SSOT
    let mut path_buf = [0u8; linux_raw_sys::general::PATH_MAX as usize];
    let path_str = crate::syscall::read_user_cstring(task.ttbr0, pathname, &mut path_buf)?;

    // TEAM_345: Handle dirfd
    if dirfd != AT_FDCWD && !path_str.starts_with('/') {
        log::warn!("[SYSCALL] mkdirat: dirfd {} not yet supported", dirfd);
        return Err(EBADF);
    }

    match vfs_mkdir(path_str, mode) {
        Ok(()) => Ok(0),
        Err(VfsError::AlreadyExists) => Err(EEXIST),
        Err(VfsError::NotFound) => Err(ENOENT),
        Err(VfsError::NotADirectory) => Err(ENOTDIR),
        Err(_) => Err(EINVAL),
    }
}

/// TEAM_345: sys_unlinkat - Linux ABI compatible.
/// TEAM_421: Updated to return SyscallResult.
/// Signature: unlinkat(dirfd, pathname, flags)
pub fn sys_unlinkat(dirfd: i32, pathname: usize, flags: u32) -> SyscallResult {
    let task = crate::task::current_task();

    // TEAM_418: Use PATH_MAX from SSOT
    let mut path_buf = [0u8; linux_raw_sys::general::PATH_MAX as usize];
    let path_str = crate::syscall::read_user_cstring(task.ttbr0, pathname, &mut path_buf)?;

    // TEAM_345: Handle dirfd
    if dirfd != AT_FDCWD && !path_str.starts_with('/') {
        log::warn!("[SYSCALL] unlinkat: dirfd {} not yet supported", dirfd);
        return Err(EBADF);
    }

    let res = if (flags & AT_REMOVEDIR) != 0 {
        vfs_rmdir(path_str)
    } else {
        vfs_unlink(path_str)
    };

    match res {
        Ok(()) => Ok(0),
        Err(VfsError::NotFound) => Err(ENOENT),
        Err(VfsError::NotADirectory) => Err(ENOTDIR),
        Err(VfsError::DirectoryNotEmpty) => Err(ENOTEMPTY),
        Err(_) => Err(EINVAL),
    }
}

/// TEAM_345: sys_renameat - Linux ABI compatible.
/// TEAM_421: Updated to return SyscallResult.
/// Signature: renameat(olddirfd, oldpath, newdirfd, newpath)
pub fn sys_renameat(
    olddirfd: i32,
    oldpath: usize,
    newdirfd: i32,
    newpath: usize,
) -> SyscallResult {
    let task = crate::task::current_task();

    // TEAM_418: Use PATH_MAX from SSOT
    let mut old_path_buf = [0u8; linux_raw_sys::general::PATH_MAX as usize];
    let old_path_str = crate::syscall::read_user_cstring(task.ttbr0, oldpath, &mut old_path_buf)?;

    // TEAM_418: Use PATH_MAX from SSOT
    let mut new_path_buf = [0u8; linux_raw_sys::general::PATH_MAX as usize];
    let new_path_str = crate::syscall::read_user_cstring(task.ttbr0, newpath, &mut new_path_buf)?;

    // TEAM_345: Handle dirfd
    if (olddirfd != AT_FDCWD && !old_path_str.starts_with('/'))
        || (newdirfd != AT_FDCWD && !new_path_str.starts_with('/')) {
        log::warn!("[SYSCALL] renameat: dirfd not yet supported");
        return Err(EBADF);
    }

    match vfs_rename(old_path_str, new_path_str) {
        Ok(()) => Ok(0),
        Err(VfsError::NotFound) => Err(ENOENT),
        Err(VfsError::NotADirectory) => Err(ENOTDIR),
        Err(VfsError::CrossDevice) => Err(EXDEV),
        Err(_) => Err(EINVAL),
    }
}
