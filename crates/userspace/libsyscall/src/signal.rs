//! Signal handling
//! TEAM_275: Refactored to use arch::syscallN

use crate::arch;
use crate::sysno::{
    __NR_kill, __NR_pause, __NR_rt_sigaction, __NR_rt_sigprocmask, __NR_rt_sigreturn,
};

// TEAM_216: Signal constants
pub use linux_raw_sys::general::{SIGCHLD, SIGINT, SIGKILL};

/// TEAM_216: Send a signal to a process.
#[inline]
pub fn kill(pid: i32, sig: i32) -> isize {
    arch::syscall2(__NR_kill as u64, pid as u64, sig as u64) as isize
}

/// TEAM_216: Wait for a signal.
#[inline]
pub fn pause() -> isize {
    arch::syscall0(__NR_pause as u64) as isize
}

/// TEAM_216: Examine and change a signal action.
#[inline]
pub fn sigaction(sig: i32, handler: usize, restorer: usize) -> isize {
    arch::syscall3(
        __NR_rt_sigaction as u64,
        sig as u64,
        handler as u64,
        restorer as u64,
    ) as isize
}

/// TEAM_216: Examine and change blocked signals.
#[inline]
pub fn sigprocmask(how: i32, set: usize, oldset: usize) -> isize {
    arch::syscall3(
        __NR_rt_sigprocmask as u64,
        how as u64,
        set as u64,
        oldset as u64,
    ) as isize
}

/// TEAM_216: Return from signal handler and cleanup stack frame.
#[inline]
pub fn sigreturn() -> ! {
    arch::syscall_noreturn(__NR_rt_sigreturn as u64)
}
