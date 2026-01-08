//! Core I/O
//! TEAM_275: Refactored to use arch::syscallN

use crate::arch;
use crate::sched::sched_yield;
use crate::sysno::{__NR_close, __NR_ioctl, __NR_read, __NR_readv, __NR_write, __NR_writev};

use linux_raw_sys::general::iovec;

/// TEAM_217: struct iovec for writev/readv
pub type IoVec = iovec;

/// TEAM_217: Vectored write.
#[inline]
pub fn writev(fd: usize, iov: &[IoVec]) -> isize {
    arch::syscall3(
        __NR_writev as u64,
        fd as u64,
        iov.as_ptr() as u64,
        iov.len() as u64,
    ) as isize
}

/// TEAM_217: Vectored read.
#[inline]
pub fn readv(fd: usize, iov: &mut [IoVec]) -> isize {
    arch::syscall3(
        __NR_readv as u64,
        fd as u64,
        iov.as_mut_ptr() as u64,
        iov.len() as u64,
    ) as isize
}

/// Read from a file descriptor.
#[inline]
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    loop {
        let ret = arch::syscall3(
            __NR_read as u64,
            fd as u64,
            buf.as_mut_ptr() as u64,
            buf.len() as u64,
        );
        if ret == -11 {
            // EAGAIN
            sched_yield();
            continue;
        }
        return ret as isize;
    }
}

/// Write to a file descriptor.
#[inline]
pub fn write(fd: usize, buf: &[u8]) -> isize {
    loop {
        let ret = arch::syscall3(
            __NR_write as u64,
            fd as u64,
            buf.as_ptr() as u64,
            buf.len() as u64,
        );
        if ret == -11 {
            // EAGAIN
            sched_yield();
            continue;
        }
        return ret as isize;
    }
}

/// Close a file descriptor.
#[inline]
pub fn close(fd: usize) -> isize {
    arch::syscall1(__NR_close as u64, fd as u64) as isize
}

/// TEAM_247: Generic ioctl wrapper.
#[inline]
pub fn ioctl(fd: usize, request: u64, arg: usize) -> isize {
    arch::syscall3(__NR_ioctl as u64, fd as u64, request, arg as u64) as isize
}
