//! Filesystem operations
//! TEAM_275: Refactored to use arch::syscallN
//! TEAM_310: Deep integration of linux-raw-sys

use crate::arch;
use crate::errno::ENAMETOOLONG;
use crate::sysno::{
    __NR_dup, __NR_dup3, __NR_fstat, __NR_getcwd, __NR_getdents, __NR_linkat, __NR_mkdirat,
    __NR_openat, __NR_pipe2, __NR_readlinkat, __NR_renameat, __NR_symlinkat, __NR_unlinkat,
    __NR_utimensat,
};
use crate::time::Timespec;

/// Maximum path length for null-terminated paths (excluding null terminator).
/// This matches Linux PATH_MAX (4096 bytes including null terminator).
const PATH_MAX: usize = 4095;

/// Helper function to copy a path into a buffer with null termination.
/// Returns an error if the path is too long.
///
/// # Safety
/// The buffer must be at least PATH_MAX + 1 bytes (4096 bytes).
#[inline]
fn copy_path_with_null(path: &str, buf: &mut [u8; 4096]) -> Result<(), isize> {
    if path.len() > PATH_MAX {
        return Err(-(ENAMETOOLONG as isize));
    }
    let len = path.len();
    buf[..len].copy_from_slice(path.as_bytes());
    buf[len] = 0; // Null terminator
    Ok(())
}

/// TEAM_250: Open flags for suite_test_core
// Re-exporting directly from linux-raw-sys
pub use linux_raw_sys::general::{
    O_APPEND, O_CLOEXEC, O_CREAT, O_EXCL, O_RDONLY, O_RDWR, O_TRUNC, O_WRONLY,
};

use linux_raw_sys::general::{linux_dirent64, stat};

/// TEAM_217: Linux-compatible Stat structure.
pub type Stat = stat;

/// TEAM_176: Dirent64 structure for directory entries.
pub type Dirent64 = linux_dirent64;

/// TEAM_176: File type constants for d_type field.
pub mod d_type {
    pub use linux_raw_sys::general::{
        DT_BLK, DT_CHR, DT_DIR, DT_FIFO, DT_LNK, DT_REG, DT_SOCK, DT_UNKNOWN,
    };
}

/// TEAM_345: Linux ABI - openat(dirfd, pathname, flags, mode)
/// TEAM_168: Original implementation.
///
/// Returns -ENAMETOOLONG if path exceeds PATH_MAX (4095 bytes).
#[inline]
pub fn openat(dirfd: i32, path: &str, flags: u32, mode: u32) -> isize {
    let mut buf = [0u8; 4096];
    if let Err(e) = copy_path_with_null(path, &mut buf) {
        return e;
    }

    arch::syscall4(
        __NR_openat as u64,
        dirfd as u64,
        buf.as_ptr() as u64,
        flags as u64,
        mode as u64,
    ) as isize
}

/// TEAM_345: Convenience wrapper using AT_FDCWD
#[inline]
pub fn open(path: &str, flags: u32) -> isize {
    openat(AT_FDCWD, path, flags, 0o666)
}

/// TEAM_345: AT_FDCWD constant for openat and other *at syscalls
pub const AT_FDCWD: i32 = -100;

/// TEAM_168: Get file status.
#[inline]
pub fn fstat(fd: usize, stat: &mut Stat) -> isize {
    arch::syscall2(__NR_fstat as u64, fd as u64, stat as *mut Stat as u64) as isize
}

/// TEAM_176: Read directory entries.
#[inline]
pub fn getdents(fd: usize, buf: &mut [u8]) -> isize {
    arch::syscall3(
        __NR_getdents as u64,
        fd as u64,
        buf.as_mut_ptr() as u64,
        buf.len() as u64,
    ) as isize
}

/// TEAM_192: Get current working directory.
#[inline]
pub fn getcwd(buf: &mut [u8]) -> isize {
    arch::syscall2(
        __NR_getcwd as u64,
        buf.as_mut_ptr() as u64,
        buf.len() as u64,
    ) as isize
}

/// TEAM_345: Linux ABI - mkdirat(dirfd, pathname, mode)
///
/// Returns -ENAMETOOLONG if path exceeds PATH_MAX (4095 bytes).
#[inline]
pub fn mkdirat(dirfd: i32, path: &str, mode: u32) -> isize {
    let mut path_buf = [0u8; 4096];
    if let Err(e) = copy_path_with_null(path, &mut path_buf) {
        return e;
    }

    arch::syscall3(
        __NR_mkdirat as u64,
        dirfd as u64,
        path_buf.as_ptr() as u64,
        mode as u64,
    ) as isize
}

/// TEAM_345: Linux ABI - unlinkat(dirfd, pathname, flags)
///
/// Returns -ENAMETOOLONG if path exceeds PATH_MAX (4095 bytes).
#[inline]
pub fn unlinkat(dirfd: i32, path: &str, flags: u32) -> isize {
    let mut path_buf = [0u8; 4096];
    if let Err(e) = copy_path_with_null(path, &mut path_buf) {
        return e;
    }

    arch::syscall3(
        __NR_unlinkat as u64,
        dirfd as u64,
        path_buf.as_ptr() as u64,
        flags as u64,
    ) as isize
}

/// TEAM_345: Linux ABI - renameat(olddirfd, oldpath, newdirfd, newpath)
///
/// Returns -ENAMETOOLONG if either path exceeds PATH_MAX (4095 bytes).
#[inline]
pub fn renameat(olddirfd: i32, oldpath: &str, newdirfd: i32, newpath: &str) -> isize {
    let mut old_buf = [0u8; 4096];
    if let Err(e) = copy_path_with_null(oldpath, &mut old_buf) {
        return e;
    }

    let mut new_buf = [0u8; 4096];
    if let Err(e) = copy_path_with_null(newpath, &mut new_buf) {
        return e;
    }

    arch::syscall4(
        __NR_renameat as u64,
        olddirfd as u64,
        old_buf.as_ptr() as u64,
        newdirfd as u64,
        new_buf.as_ptr() as u64,
    ) as isize
}

/// TEAM_345: Linux ABI - symlinkat(target, newdirfd, linkpath)
///
/// Returns -ENAMETOOLONG if either path exceeds PATH_MAX (4095 bytes).
#[inline]
pub fn symlinkat(target: &str, newdirfd: i32, linkpath: &str) -> isize {
    let mut target_buf = [0u8; 4096];
    if let Err(e) = copy_path_with_null(target, &mut target_buf) {
        return e;
    }

    let mut link_buf = [0u8; 4096];
    if let Err(e) = copy_path_with_null(linkpath, &mut link_buf) {
        return e;
    }

    arch::syscall3(
        __NR_symlinkat as u64,
        target_buf.as_ptr() as u64,
        newdirfd as u64,
        link_buf.as_ptr() as u64,
    ) as isize
}

/// TEAM_345: Linux ABI - readlinkat(dirfd, pathname, buf, bufsiz)
///
/// Returns -ENAMETOOLONG if path exceeds PATH_MAX (4095 bytes).
#[inline]
pub fn readlinkat(dirfd: i32, path: &str, buf: &mut [u8]) -> isize {
    let mut path_buf = [0u8; 4096];
    if let Err(e) = copy_path_with_null(path, &mut path_buf) {
        return e;
    }

    arch::syscall4(
        __NR_readlinkat as u64,
        dirfd as u64,
        path_buf.as_ptr() as u64,
        buf.as_mut_ptr() as u64,
        buf.len() as u64,
    ) as isize
}

/// TEAM_345: Linux ABI - linkat(olddirfd, oldpath, newdirfd, newpath, flags)
///
/// Returns -ENAMETOOLONG if either path exceeds PATH_MAX (4095 bytes).
#[inline]
pub fn linkat(olddirfd: i32, oldpath: &str, newdirfd: i32, newpath: &str, flags: u32) -> isize {
    let mut old_buf = [0u8; 4096];
    if let Err(e) = copy_path_with_null(oldpath, &mut old_buf) {
        return e;
    }

    let mut new_buf = [0u8; 4096];
    if let Err(e) = copy_path_with_null(newpath, &mut new_buf) {
        return e;
    }

    arch::syscall5(
        __NR_linkat as u64,
        olddirfd as u64,
        old_buf.as_ptr() as u64,
        newdirfd as u64,
        new_buf.as_ptr() as u64,
        flags as u64,
    ) as isize
}

/// TEAM_198: UTIME_NOW - set to current time
pub const UTIME_NOW: u64 = 0x3FFFFFFF;
/// TEAM_198: UTIME_OMIT - don't change
pub const UTIME_OMIT: u64 = 0x3FFFFFFE;

/// TEAM_345: Linux ABI - utimensat(dirfd, pathname, times, flags)
///
/// Returns -ENAMETOOLONG if path exceeds PATH_MAX (4095 bytes).
#[inline]
pub fn utimensat(dirfd: i32, path: &str, times: Option<&[Timespec; 2]>, flags: u32) -> isize {
    let mut path_buf = [0u8; 4096];
    if let Err(e) = copy_path_with_null(path, &mut path_buf) {
        return e;
    }

    let times_ptr = match times {
        Some(t) => t.as_ptr() as u64,
        None => 0,
    };
    arch::syscall4(
        __NR_utimensat as u64,
        dirfd as u64,
        path_buf.as_ptr() as u64,
        times_ptr,
        flags as u64,
    ) as isize
}

/// TEAM_233: Create a pipe.
#[inline]
pub fn pipe2(pipefd: &mut [i32; 2], flags: u32) -> isize {
    arch::syscall2(__NR_pipe2 as u64, pipefd.as_mut_ptr() as u64, flags as u64) as isize
}

/// TEAM_233: Duplicate a file descriptor to lowest available.
#[inline]
pub fn dup(oldfd: usize) -> isize {
    arch::syscall1(__NR_dup as u64, oldfd as u64) as isize
}

/// TEAM_233: Duplicate a file descriptor to a specific number.
#[inline]
pub fn dup2(oldfd: usize, newfd: usize) -> isize {
    dup3(oldfd, newfd, 0)
}

/// TEAM_233: Duplicate a file descriptor with flags.
#[inline]
pub fn dup3(oldfd: usize, newfd: usize, flags: u32) -> isize {
    arch::syscall3(__NR_dup3 as u64, oldfd as u64, newfd as u64, flags as u64) as isize
}
