//! Time operations
//! TEAM_275: Refactored to use arch::syscallN

use crate::arch;
use crate::sysno::{__NR_clock_gettime, __NR_nanosleep};

use linux_raw_sys::general::timespec;

/// TEAM_217: Linux-compatible Timespec.
pub type Timespec = timespec;

/// TEAM_170: Sleep for specified duration.
///
/// **Non-standard ABI**: Unlike Linux, LevitateOS's nanosleep takes two u64 arguments
/// (seconds and nanoseconds) directly instead of a pointer to `struct timespec`.
/// The kernel automatically normalizes nanoseconds > 1e9 into additional seconds.
///
/// # Arguments
/// - `seconds`: Number of seconds to sleep
/// - `nanoseconds`: Number of nanoseconds to sleep (will be normalized if >= 1e9)
///
/// # Returns
/// - `0` on success
/// - Negative error code on failure
#[inline]
pub fn nanosleep(seconds: u64, nanoseconds: u64) -> isize {
    arch::syscall2(__NR_nanosleep as u64, seconds, nanoseconds) as isize
}

/// TEAM_170: Get current monotonic time.
#[inline]
pub fn clock_gettime(ts: &mut Timespec) -> isize {
    arch::syscall1(__NR_clock_gettime as u64, ts as *mut Timespec as u64) as isize
}
