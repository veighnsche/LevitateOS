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
pub const SYS_SPAWN: u64 = 5;
pub const SYS_EXEC: u64 = 6;
pub const SYS_YIELD: u64 = 7;
/// TEAM_142: Graceful system shutdown
pub const SYS_SHUTDOWN: u64 = 8;
/// TEAM_168: Open file
pub const SYS_OPENAT: u64 = 9;
/// TEAM_168: Close file descriptor
pub const SYS_CLOSE: u64 = 10;
/// TEAM_168: Get file status
pub const SYS_FSTAT: u64 = 11;
/// TEAM_170: Sleep for nanoseconds
pub const SYS_NANOSLEEP: u64 = 12;
/// TEAM_170: Get monotonic time
pub const SYS_CLOCK_GETTIME: u64 = 13;
/// TEAM_186: Spawn process with arguments
pub const SYS_SPAWN_ARGS: u64 = 15;
/// TEAM_188: Wait for child process
pub const SYS_WAITPID: u64 = 16;

/// TEAM_142: Shutdown flags
pub mod shutdown_flags {
    /// Normal shutdown (minimal output)
    pub const NORMAL: u32 = 0;
    /// Verbose shutdown (for golden file testing)
    pub const VERBOSE: u32 = 1;
}

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

/// Spawn a new process from a path.
///
/// # Arguments
/// * `path` - Path to the executable
///
/// # Returns
/// PID of the new process, or negative error code.
#[inline]
pub fn spawn(path: &str) -> isize {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_SPAWN,
            in("x0") path.as_ptr(),
            in("x1") path.len(),
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret as isize
}

/// Replace current process with a new one from a path.
///
/// # Arguments
/// * `path` - Path to the executable
///
/// # Returns
/// Does not return on success. Negative error code on failure.
#[inline]
pub fn exec(path: &str) -> isize {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_EXEC,
            in("x0") path.as_ptr(),
            in("x1") path.len(),
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret as isize
}

/// TEAM_186: Argv entry for spawn_args syscall.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ArgvEntry {
    /// Pointer to argument string
    pub ptr: *const u8,
    /// Length of argument string
    pub len: usize,
}

/// TEAM_186: Spawn a process with command-line arguments.
///
/// # Arguments
/// * `path` - Path to the executable
/// * `argv` - Command-line arguments (including program name as argv[0])
///
/// # Returns
/// PID of the new process, or negative error code.
#[inline]
pub fn spawn_args(path: &str, argv: &[&str]) -> isize {
    // Build ArgvEntry array on stack (max 16 args)
    let mut entries = [ArgvEntry {
        ptr: core::ptr::null(),
        len: 0,
    }; 16];
    let argc = argv.len().min(16);
    for (i, arg) in argv.iter().take(argc).enumerate() {
        entries[i] = ArgvEntry {
            ptr: arg.as_ptr(),
            len: arg.len(),
        };
    }

    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_SPAWN_ARGS,
            in("x0") path.as_ptr(),
            in("x1") path.len(),
            in("x2") entries.as_ptr(),
            in("x3") argc,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret as isize
}

/// TEAM_188: Wait for a child process to exit.
///
/// # Arguments
/// * `pid` - PID of child to wait for (must be > 0)
/// * `status` - Optional pointer to store exit status
///
/// # Returns
/// PID of exited child on success, negative error code on failure.
#[inline]
pub fn waitpid(pid: i32, status: Option<&mut i32>) -> isize {
    let status_ptr = match status {
        Some(s) => s as *mut i32 as usize,
        None => 0,
    };

    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_WAITPID,
            in("x0") pid,
            in("x1") status_ptr,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret as isize
}

/// Yield CPU to other tasks.
///
/// TEAM_129: Added to allow cooperative scheduling.
#[inline]
pub fn yield_cpu() {
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_YIELD,
            options(nostack)
        );
    }
}

/// TEAM_142: Graceful system shutdown.
///
/// # Arguments
/// * `flags` - Shutdown flags (see `shutdown_flags` module)
///   - `NORMAL` (0): Minimal output
///   - `VERBOSE` (1): Detailed output for golden file testing
///
/// # Returns
/// Does not return on success. Halts the system.
#[inline]
pub fn shutdown(flags: u32) -> ! {
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_SHUTDOWN,
            in("x0") flags,
            options(noreturn, nostack)
        );
    }
}

// ============================================================================
// File Syscalls (TEAM_168: Phase 10 Step 3)
// ============================================================================

/// TEAM_168: Open a file.
///
/// # Arguments
/// * `path` - Path to the file
/// * `flags` - Open flags (0 for read-only)
///
/// # Returns
/// File descriptor on success, or negative error code.
#[inline]
pub fn openat(path: &str, flags: u32) -> isize {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_OPENAT,
            in("x0") path.as_ptr(),
            in("x1") path.len(),
            in("x2") flags,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret as isize
}

/// TEAM_168: Close a file descriptor.
///
/// # Arguments
/// * `fd` - File descriptor to close
///
/// # Returns
/// 0 on success, or negative error code.
#[inline]
pub fn close(fd: usize) -> isize {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_CLOSE,
            in("x0") fd,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret as isize
}

/// TEAM_168: Stat structure for fstat.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Stat {
    /// File size in bytes
    pub st_size: u64,
    /// File type (1 = regular, 2 = char device)
    pub st_mode: u32,
    /// Padding
    pub _pad: u32,
}

/// TEAM_168: Get file status.
///
/// # Arguments
/// * `fd` - File descriptor
/// * `stat` - Output buffer for file status
///
/// # Returns
/// 0 on success, or negative error code.
#[inline]
pub fn fstat(fd: usize, stat: &mut Stat) -> isize {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_FSTAT,
            in("x0") fd,
            in("x1") stat as *mut Stat,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret as isize
}

// ============================================================================
// Time Syscalls (TEAM_170: Phase 10 Step 7)
// ============================================================================

/// TEAM_170: Timespec structure for time syscalls.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Timespec {
    /// Seconds
    pub tv_sec: u64,
    /// Nanoseconds
    pub tv_nsec: u64,
}

/// TEAM_170: Sleep for specified duration.
///
/// # Arguments
/// * `seconds` - Number of seconds to sleep
/// * `nanoseconds` - Additional nanoseconds to sleep
///
/// # Returns
/// 0 on success, or negative error code.
#[inline]
pub fn nanosleep(seconds: u64, nanoseconds: u64) -> isize {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_NANOSLEEP,
            in("x0") seconds,
            in("x1") nanoseconds,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret as isize
}

/// TEAM_170: Get current monotonic time.
///
/// # Arguments
/// * `ts` - Output buffer for timespec
///
/// # Returns
/// 0 on success, or negative error code.
#[inline]
pub fn clock_gettime(ts: &mut Timespec) -> isize {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_CLOCK_GETTIME,
            in("x0") ts as *mut Timespec,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret as isize
}

// ============================================================================
// Directory Syscalls (TEAM_176: Directory Iteration)
// ============================================================================

/// TEAM_176: Syscall number for getdents.
pub const SYS_GETDENTS: u64 = 14;

/// TEAM_176: Dirent64 structure for directory entries.
/// Matches Linux ABI layout.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Dirent64 {
    /// Inode number
    pub d_ino: u64,
    /// Offset to next entry
    pub d_off: i64,
    /// Length of this record
    pub d_reclen: u16,
    /// File type
    pub d_type: u8,
    // d_name follows (null-terminated, variable length)
}

/// TEAM_176: File type constants for d_type field.
pub mod d_type {
    pub const DT_UNKNOWN: u8 = 0;
    pub const DT_FIFO: u8 = 1;
    pub const DT_CHR: u8 = 2;
    pub const DT_DIR: u8 = 4;
    pub const DT_BLK: u8 = 6;
    pub const DT_REG: u8 = 8;
    pub const DT_LNK: u8 = 10;
    pub const DT_SOCK: u8 = 12;
}

/// TEAM_176: Read directory entries.
///
/// # Arguments
/// * `fd` - Directory file descriptor
/// * `buf` - Buffer to read entries into
///
/// # Returns
/// Number of bytes read on success, 0 at end of directory, or negative error code.
#[inline]
pub fn getdents(fd: usize, buf: &mut [u8]) -> isize {
    let ret: i64;
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_GETDENTS,
            in("x0") fd,
            in("x1") buf.as_mut_ptr(),
            in("x2") buf.len(),
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret as isize
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
