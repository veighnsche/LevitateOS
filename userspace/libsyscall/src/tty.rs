//! TTY and Terminal operations
//! TEAM_275: Refactored to use arch::syscallN

use crate::arch;
use crate::sysno::{__NR_ioctl, SYS_ISATTY};
use linux_raw_sys::ioctl::{TCGETS, TCSETS, TCSETSF, TCSETSW};

// ... constants unchanged ...

/// TEAM_244: Get terminal attributes (POSIX tcgetattr).
/// Returns 0 on success, negative error on failure.
#[inline]
pub fn tcgetattr(fd: i32, termios_p: *mut u8) -> isize {
    arch::syscall3(
        __NR_ioctl as u64,
        fd as u64,
        TCGETS as u64,
        termios_p as u64,
    ) as isize
}

/// TEAM_244: Set terminal attributes (POSIX tcsetattr).
/// Returns 0 on success, negative error on failure.
#[inline]
pub fn tcsetattr(fd: i32, optional_actions: i32, termios_p: *const u8) -> isize {
    let request: u32 = match optional_actions {
        0 => TCSETS,  // TCSANOW
        1 => TCSETSW, // TCSADRAIN
        2 => TCSETSF, // TCSAFLUSH
        _ => TCSETS,
    };
    arch::syscall3(
        __NR_ioctl as u64,
        fd as u64,
        request as u64,
        termios_p as u64,
    ) as isize
}

/// TEAM_244: Check if fd refers to a terminal.
/// Returns 1 if tty, 0 if not, negative error on failure.
#[inline]
pub fn isatty(fd: i32) -> isize {
    arch::syscall1(SYS_ISATTY as u64, fd as u64) as isize
}
