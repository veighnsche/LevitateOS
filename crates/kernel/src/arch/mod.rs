//! TEAM_162: Architecture Abstraction Layer
//!
//! This module defines the architecture-independent interface for the kernel.
//! Current supported architectures:
//! - aarch64
//! - x86_64 (stubs)

// TEAM_162: Export the current architecture modules
// TEAM_163: Removed dead EarlyConsole infrastructure (Rule 6: No Dead Code)
#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

#[cfg(target_arch = "aarch64")]
unsafe extern "C" {
    pub fn exception_return();
}

// TEAM_162: Provide a stub for x86_64 to ensure the boundary is clean
#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;
