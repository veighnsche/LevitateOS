use crate::memory::user as mm_user;

use crate::fs::vfs::dispatch::*;
use crate::fs::vfs::error::VfsError;
use crate::syscall::{errno, write_to_user_buf};
use crate::task::fd_table::FdType;

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

pub fn sys_getdents(fd: usize, buf: usize, buf_len: usize) -> i64 {
    if buf_len == 0 {
        return 0;
    }

    let task = crate::task::current_task();
    if mm_user::validate_user_buffer(task.ttbr0, buf, buf_len, true).is_err() {
        return errno::EFAULT;
    }

    let fd_table = task.fd_table.lock();
    let entry = match fd_table.get(fd) {
        Some(e) => e.clone(),
        None => return errno::EBADF,
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
                                return errno::EFAULT;
                            }
                        }

                        let name_offset = bytes_written + core::mem::size_of::<Dirent64>();
                        for (i, &byte) in name_bytes.iter().enumerate() {
                            if !write_to_user_buf(task.ttbr0, buf, name_offset + i, byte) {
                                return errno::EFAULT;
                            }
                        }

                        if !write_to_user_buf(task.ttbr0, buf, name_offset + name_len, 0) {
                            return errno::EFAULT;
                        }

                        let _ =
                            file.seek((offset + 1) as i64, crate::fs::vfs::ops::SeekWhence::Set);
                        bytes_written += reclen;
                    }
                    Ok(None) => break,
                    Err(_) => return errno::EBADF,
                }
            }
            bytes_written as i64
        }
        _ => errno::ENOTDIR,
    }
}

pub fn sys_getcwd(buf: usize, size: usize) -> i64 {
    let task = crate::task::current_task();
    if mm_user::validate_user_buffer(task.ttbr0, buf, size, true).is_err() {
        return errno::EFAULT;
    }

    let cwd_lock = task.cwd.lock();
    let path = cwd_lock.as_str();
    let path_len = path.len();
    if size < path_len + 1 {
        return errno::ERANGE;
    }

    // SAFETY: validate_user_buffer confirmed buffer is accessible
    let dest = mm_user::user_va_to_kernel_ptr(task.ttbr0, buf).unwrap();
    unsafe {
        core::ptr::copy_nonoverlapping(path.as_bytes().as_ptr(), dest, path_len);
        *dest.add(path_len) = 0; // null terminator
    }

    (path_len + 1) as i64
}

/// TEAM_345: sys_mkdirat - Linux ABI compatible.
/// Signature: mkdirat(dirfd, pathname, mode)
pub fn sys_mkdirat(dirfd: i32, pathname: usize, mode: u32) -> i64 {
    let task = crate::task::current_task();
    
    // TEAM_345: Read null-terminated pathname (Linux ABI)
    let mut path_buf = [0u8; 4096];
    let path_str = match crate::syscall::read_user_cstring(task.ttbr0, pathname, &mut path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    // TEAM_345: Handle dirfd
    if dirfd != crate::syscall::fcntl::AT_FDCWD && !path_str.starts_with('/') {
        log::warn!("[SYSCALL] mkdirat: dirfd {} not yet supported", dirfd);
        return errno::EBADF;
    }

    match vfs_mkdir(path_str, mode) {
        Ok(()) => 0,
        Err(VfsError::AlreadyExists) => errno::EEXIST,
        Err(VfsError::NotFound) => errno::ENOENT,
        Err(VfsError::NotADirectory) => errno::ENOTDIR,
        Err(_) => errno::EINVAL,
    }
}

/// TEAM_345: sys_unlinkat - Linux ABI compatible.
/// Signature: unlinkat(dirfd, pathname, flags)
pub fn sys_unlinkat(dirfd: i32, pathname: usize, flags: u32) -> i64 {
    let task = crate::task::current_task();
    
    // TEAM_345: Read null-terminated pathname (Linux ABI)
    let mut path_buf = [0u8; 4096];
    let path_str = match crate::syscall::read_user_cstring(task.ttbr0, pathname, &mut path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    // TEAM_345: Handle dirfd
    if dirfd != crate::syscall::fcntl::AT_FDCWD && !path_str.starts_with('/') {
        log::warn!("[SYSCALL] unlinkat: dirfd {} not yet supported", dirfd);
        return errno::EBADF;
    }

    let res = if (flags & crate::syscall::fcntl::AT_REMOVEDIR) != 0 {
        vfs_rmdir(path_str)
    } else {
        vfs_unlink(path_str)
    };

    match res {
        Ok(()) => 0,
        Err(VfsError::NotFound) => errno::ENOENT,
        Err(VfsError::NotADirectory) => errno::ENOTDIR,
        Err(VfsError::DirectoryNotEmpty) => errno::ENOTEMPTY,
        Err(_) => errno::EINVAL,
    }
}

/// TEAM_345: sys_renameat - Linux ABI compatible.
/// Signature: renameat(olddirfd, oldpath, newdirfd, newpath)
pub fn sys_renameat(
    olddirfd: i32,
    oldpath: usize,
    newdirfd: i32,
    newpath: usize,
) -> i64 {
    let task = crate::task::current_task();

    // TEAM_345: Read null-terminated oldpath
    let mut old_path_buf = [0u8; 4096];
    let old_path_str = match crate::syscall::read_user_cstring(task.ttbr0, oldpath, &mut old_path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };

    // TEAM_345: Read null-terminated newpath
    let mut new_path_buf = [0u8; 4096];
    let new_path_str = match crate::syscall::read_user_cstring(task.ttbr0, newpath, &mut new_path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    // TEAM_345: Handle dirfd
    if (olddirfd != crate::syscall::fcntl::AT_FDCWD && !old_path_str.starts_with('/'))
        || (newdirfd != crate::syscall::fcntl::AT_FDCWD && !new_path_str.starts_with('/')) {
        log::warn!("[SYSCALL] renameat: dirfd not yet supported");
        return errno::EBADF;
    }

    match vfs_rename(old_path_str, new_path_str) {
        Ok(()) => 0,
        Err(VfsError::NotFound) => errno::ENOENT,
        Err(VfsError::NotADirectory) => errno::ENOTDIR,
        Err(VfsError::CrossDevice) => errno::EXDEV,
        Err(_) => errno::EINVAL,
    }
}
