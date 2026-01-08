//! Synchronization (Futex)
//! TEAM_275: Refactored to use arch::syscallN

use crate::arch;
use crate::sysno::__NR_futex;

/// TEAM_208: Futex operations
pub mod futex_ops {
    pub const FUTEX_WAIT: usize = 0;
    pub const FUTEX_WAKE: usize = 1;
}

/// TEAM_208: Fast userspace mutex operation.
///
/// # Arguments
/// * `addr` - Pointer to a 4-byte aligned u32 value
/// * `op` - Operation (FUTEX_WAIT or FUTEX_WAKE)
/// * `val` - Expected value (for WAIT) or max waiters to wake (for WAKE)
///
/// # Returns
/// * FUTEX_WAIT: 0 on success, -11 (EAGAIN) if value mismatch
/// * FUTEX_WAKE: Number of tasks woken
#[inline]
pub fn futex(addr: *const u32, op: usize, val: u32) -> isize {
    arch::syscall5(
        __NR_futex as u64,
        addr as u64,
        op as u64,
        val as u64,
        0, // timeout (unused)
        0, // addr2 (unused)
    ) as isize
}

/// TEAM_208: sys_futex syscall wrapper.
#[inline]
pub fn sys_futex(
    uaddr: usize,
    op: i32,
    val: u32,
    timeout: usize,
    uaddr2: usize,
    val3: u32,
) -> isize {
    arch::syscall6(
        __NR_futex as u64,
        uaddr as u64,
        op as u64,
        val as u64,
        timeout as u64,
        uaddr2 as u64,
        val3 as u64,
    ) as isize
}
