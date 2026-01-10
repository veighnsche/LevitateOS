//! Userspace Syscall Library for `LevitateOS`
//!
//! `TEAM_118`: Extracted from shell to provide a single source of truth for syscall ABI.
//! `TEAM_251`: Refactored into modules.
//!
//! ## Usage
//! ```rust
//! use libsyscall::{read, write, exit, println};
//! ```

#![no_std]

use core::panic::PanicInfo;

// TEAM_275: Architecture-specific syscall primitives
mod arch;

// Modules
pub mod errno;
pub mod fs;
pub mod io;
pub mod mm;
pub mod process;
pub mod sched;
pub mod signal;
pub mod sync;
pub mod sysno;
pub mod time;
pub mod tty;

// Re-exports for backward compatibility
pub use errno::*;
pub use fs::*;
pub use io::*;
pub use mm::*;
pub use process::*;
pub use sched::*;
pub use signal::*;
pub use sync::*;
pub use sysno::*;
pub use time::*;
pub use tty::*;

// ============================================================================
// Panic Handler (shared logic)
// ============================================================================

/// Common panic handler logic.
///
/// Call this from `#[panic_handler]` in each binary crate.
///
/// # Example
/// ```rust
/// #[panic_handler]
/// fn panic(info: &PanicInfo) -> ! {
///     libsyscall::common_panic_handler(info)
/// }
/// ```
pub fn common_panic_handler(_info: &PanicInfo) -> ! {
    // Use write() directly to avoid recursion through print! macros
    let msg = b"PANIC!\n";
    io::write(2, msg);
    process::exit(1);
}

// ============================================================================
// Print Macros
// ============================================================================

/// Print to stdout without newline.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut writer = $crate::Stdout;
        let _ = write!(writer, $($arg)*);
    }};
}

/// Print to stdout with newline.
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}

/// Stdout writer for print! macro.
pub struct Stdout;

impl core::fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        io::write(1, s.as_bytes());
        Ok(())
    }
}
