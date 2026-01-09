use crate::memory::user as mm_user;

use crate::fs::vfs::dispatch::*;
use crate::fs::vfs::error::VfsError;
use crate::syscall::{errno, errno_file, write_to_user_buf};
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
        _ => errno_file::ENOTDIR,
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
        return -34; // ERANGE
    }

    for (i, &byte) in path.as_bytes().iter().enumerate() {
        if !write_to_user_buf(task.ttbr0, buf, i, byte) {
            return errno::EFAULT;
        }
    }
    if !write_to_user_buf(task.ttbr0, buf, path_len, 0) {
        return errno::EFAULT;
    }

    (path_len + 1) as i64
}

/// TEAM_192: sys_mkdirat - Create directory.
/// TEAM_194: Updated to support tmpfs at /tmp.
pub fn sys_mkdirat(_dfd: i32, path: usize, path_len: usize, mode: u32) -> i64 {
    let task = crate::task::current_task();
    let mut path_buf = [0u8; 256];
    let path_str = match crate::syscall::copy_user_string(task.ttbr0, path, path_len, &mut path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };

    match vfs_mkdir(path_str, mode) {
        Ok(()) => 0,
        Err(VfsError::AlreadyExists) => -17, // EEXIST
        Err(VfsError::NotFound) => errno_file::ENOENT,
        Err(VfsError::NotADirectory) => errno_file::ENOTDIR,
        Err(_) => errno::EINVAL,
    }
}

/// TEAM_192: sys_unlinkat - Remove file or directory.
/// TEAM_194: Updated to support tmpfs at /tmp.
pub fn sys_unlinkat(_dfd: i32, path: usize, path_len: usize, flags: u32) -> i64 {
    /// TEAM_194: AT_REMOVEDIR flag for unlinkat
    const AT_REMOVEDIR: u32 = 0x200;

    let task = crate::task::current_task();
    let mut path_buf = [0u8; 256];
    let path_str = match crate::syscall::copy_user_string(task.ttbr0, path, path_len, &mut path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };

    let res = if (flags & AT_REMOVEDIR) != 0 {
        vfs_rmdir(path_str)
    } else {
        vfs_unlink(path_str)
    };

    match res {
        Ok(()) => 0,
        Err(VfsError::NotFound) => errno_file::ENOENT,
        Err(VfsError::NotADirectory) => errno_file::ENOTDIR,
        Err(VfsError::DirectoryNotEmpty) => -39, // ENOTEMPTY
        Err(_) => errno::EINVAL,
    }
}

/// TEAM_192: sys_renameat - Rename or move file or directory.
/// TEAM_194: Updated to support tmpfs at /tmp.
pub fn sys_renameat(
    _old_dfd: i32,
    old_path: usize,
    old_path_len: usize,
    _new_dfd: i32,
    new_path: usize,
    new_path_len: usize,
) -> i64 {
    let task = crate::task::current_task();

    // Resolve old path
    let mut old_path_buf = [0u8; 256];
    let old_path_str = match crate::syscall::copy_user_string(task.ttbr0, old_path, old_path_len, &mut old_path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };

    // Resolve new path
    let mut new_path_buf = [0u8; 256];
    let new_path_str = match crate::syscall::copy_user_string(task.ttbr0, new_path, new_path_len, &mut new_path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };

    match vfs_rename(old_path_str, new_path_str) {
        Ok(()) => 0,
        Err(VfsError::NotFound) => errno_file::ENOENT,
        Err(VfsError::NotADirectory) => errno_file::ENOTDIR,
        Err(VfsError::CrossDevice) => -18, // EXDEV
        Err(_) => errno::EINVAL,
    }
}
