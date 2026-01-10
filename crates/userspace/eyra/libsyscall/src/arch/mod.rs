//! TEAM_275: Architecture-specific syscall primitives
//!
//! This module provides low-level syscall invocation for each supported architecture.
//! All other modules use `arch::syscallN()` instead of inline assembly directly.

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;
