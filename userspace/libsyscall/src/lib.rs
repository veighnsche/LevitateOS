//! Userspace Syscall Library for `LevitateOS`
//!
//! `TEAM_118`: Extracted from shell to provide a single source of truth for syscall ABI.
//!
//! ## Usage
//! ```rust
//! use libsyscall::{read, write, exit, println};
//! ```

#![no_std]

use core::panic::PanicInfo;

// ============================================================================
// Syscall Numbers (must match kernel/src/syscall.rs)
// ============================================================================

pub const SYS_READ: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT: u64 = 2;
pub const SYS_GETPID: u64 = 3;
pub const SYS_SBRK: u64 = 4;

// ============================================================================
// Syscall Wrappers
// ============================================================================

/// Read from a file descriptor.
///
/// # Arguments
/// * `fd` - File descriptor (0 = stdin)
/// * `buf` - Buffer to read into
///
/// # Returns
/// Number of bytes read, or negative error code.
#[inline]
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_READ,
            in("x0") fd,
            in("x1") buf.as_mut_ptr(),
            in("x2") buf.len(),
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret as isize
}

/// Write to a file descriptor.
///
/// # Arguments
/// * `fd` - File descriptor (1 = stdout, 2 = stderr)
/// * `buf` - Buffer to write from
///
/// # Returns
/// Number of bytes written, or negative error code.
#[inline]
pub fn write(fd: usize, buf: &[u8]) -> isize {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_WRITE,
            in("x0") fd,
            in("x1") buf.as_ptr(),
            in("x2") buf.len(),
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret as isize
}

/// Exit the process.
///
/// # Arguments
/// * `code` - Exit code (0 = success)
#[inline]
pub fn exit(code: i32) -> ! {
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_EXIT,
            in("x0") code,
            options(noreturn, nostack)
        );
    }
}

/// Get current process ID.
#[inline]
pub fn getpid() -> i64 {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_GETPID,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret
}

/// Adjust program break (heap allocation).
#[inline]
pub fn sbrk(increment: isize) -> i64 {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_SBRK,
            in("x0") increment,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret
}

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
    write(2, msg);
    exit(1);
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
        write(1, s.as_bytes());
        Ok(())
    }
}
