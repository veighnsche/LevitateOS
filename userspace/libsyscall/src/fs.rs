//! Filesystem operations
//! TEAM_275: Refactored to use arch::syscallN
//! TEAM_310: Deep integration of linux-raw-sys

use crate::arch;
use crate::sysno::{
    __NR_dup, __NR_dup3, __NR_fstat, __NR_getcwd, __NR_getdents, __NR_linkat, __NR_mkdirat,
    __NR_openat, __NR_pipe2, __NR_readlinkat, __NR_renameat, __NR_symlinkat, __NR_unlinkat,
    __NR_utimensat,
};
use crate::time::Timespec;

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

/// TEAM_168: Open a file.
#[inline]
pub fn openat(path: &str, flags: u32) -> isize {
    arch::syscall3(
        __NR_openat as u64,
        path.as_ptr() as u64,
        path.len() as u64,
        flags as u64,
    ) as isize
}

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

/// TEAM_192: Create directory.
#[inline]
pub fn mkdirat(dfd: i32, path: &str, mode: u32) -> isize {
    arch::syscall4(
        __NR_mkdirat as u64,
        dfd as u64,
        path.as_ptr() as u64,
        path.len() as u64,
        mode as u64,
    ) as isize
}

/// TEAM_192: Remove file or directory.
#[inline]
pub fn unlinkat(dfd: i32, path: &str, flags: u32) -> isize {
    arch::syscall4(
        __NR_unlinkat as u64,
        dfd as u64,
        path.as_ptr() as u64,
        path.len() as u64,
        flags as u64,
    ) as isize
}

/// TEAM_192: Rename/move file or directory.
#[inline]
pub fn renameat(old_dfd: i32, old_path: &str, new_dfd: i32, new_path: &str) -> isize {
    arch::syscall6(
        __NR_renameat as u64,
        old_dfd as u64,
        old_path.as_ptr() as u64,
        old_path.len() as u64,
        new_dfd as u64,
        new_path.as_ptr() as u64,
        new_path.len() as u64,
    ) as isize
}

/// TEAM_198: Create a symbolic link.
#[inline]
pub fn symlinkat(target: &str, linkdirfd: i32, linkpath: &str) -> isize {
    arch::syscall5(
        __NR_symlinkat as u64,
        target.as_ptr() as u64,
        target.len() as u64,
        linkdirfd as u64,
        linkpath.as_ptr() as u64,
        linkpath.len() as u64,
    ) as isize
}

/// TEAM_253: Read value of a symbolic link.
#[inline]
pub fn readlinkat(dirfd: i32, path: &str, buf: &mut [u8]) -> isize {
    arch::syscall5(
        __NR_readlinkat as u64,
        dirfd as u64,
        path.as_ptr() as u64,
        path.len() as u64,
        buf.as_mut_ptr() as u64,
        buf.len() as u64,
    ) as isize
}

/// TEAM_209: Create a hard link.
#[inline]
pub fn linkat(olddfd: i32, oldpath: &str, newdfd: i32, newpath: &str, flags: u32) -> isize {
    arch::syscall7(
        __NR_linkat as u64,
        olddfd as u64,
        oldpath.as_ptr() as u64,
        oldpath.len() as u64,
        newdfd as u64,
        newpath.as_ptr() as u64,
        newpath.len() as u64,
        flags as u64,
    ) as isize
}

/// TEAM_198: UTIME_NOW - set to current time
pub const UTIME_NOW: u64 = 0x3FFFFFFF;
/// TEAM_198: UTIME_OMIT - don't change
pub const UTIME_OMIT: u64 = 0x3FFFFFFE;

/// TEAM_198: Set file access and modification times.
#[inline]
pub fn utimensat(dirfd: i32, path: &str, times: Option<&[Timespec; 2]>, flags: u32) -> isize {
    let times_ptr = match times {
        Some(t) => t.as_ptr() as u64,
        None => 0,
    };
    arch::syscall5(
        __NR_utimensat as u64,
        dirfd as u64,
        path.as_ptr() as u64,
        path.len() as u64,
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
