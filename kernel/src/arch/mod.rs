//! TEAM_162: Architecture Abstraction Layer
//!
//! This module defines the architecture-independent interface for the kernel.
//! Current supported architectures:
//! - aarch64
//! - x86_64 (stubs)

use crate::alloc::string::ToString;

// TEAM_162: Export the current architecture modules
#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

// TEAM_162: Provide a stub for x86_64 to ensure the boundary is clean
#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

// TEAM_162: Export architecture-specific types
#[cfg(target_arch = "x86_64")]
mod x86_64;

/// TEAM_162: Early diagnostic interface for new architecture bringup.
/// Every architecture must provide a way to write to a console before the full
/// HAL is initialized.
pub trait EarlyConsole {
    fn write_str(&self, s: &str);
}

/// TEAM_162: Global early console instance.
/// Provided by the architecture-specific module.
pub fn early_println(args: core::fmt::Arguments) {
    let console_opt = unsafe { get_early_console() };
    if let Some(console) = console_opt {
        // Simple write for now, can be expanded to use core::fmt::Write
        // For early boot, simplicity > features (Rule 20)
        let _ = console.write_str(core::format_args!("{}\n", args).to_string().as_str());
    }
}

// These should be implemented by the specific architecture modules
unsafe extern "Rust" {
    fn get_early_console() -> Option<&'static dyn EarlyConsole>;
}
