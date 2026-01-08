//! Scheduling
//! TEAM_275: Refactored to use arch::syscallN

use crate::arch;
use crate::sysno::__NR_sched_yield;

/// Yield execution to another thread.
#[inline]
pub fn sched_yield() {
    arch::syscall0(__NR_sched_yield as u64);
}

#[inline]
pub fn yield_cpu() {
    sched_yield();
}
