//! TEAM_073: System Call Handler for LevitateOS.
//!
//! Implements the syscall dispatch table and individual syscall handlers.
//! Uses a custom ABI (not Linux-compatible) as decided in Phase 2.
//!
//! ## Syscall ABI
//! - Syscall number in `x8`
//! - Arguments in `x0-x5` (up to 6 arguments)
//! - Return value in `x0`
//! - Invoked via `svc #0` instruction from EL0

use levitate_hal::{print, println};

/// TEAM_073: Error codes for syscalls.
/// Using negative values like POSIX, but custom numbering.
pub mod errno {
    /// Function not implemented
    pub const ENOSYS: i64 = -1;
    /// Bad file descriptor
    pub const EBADF: i64 = -2;
    /// Bad address (invalid user pointer)
    pub const EFAULT: i64 = -3;
    /// Invalid argument
    #[allow(dead_code)]
    pub const EINVAL: i64 = -4;
}

/// TEAM_073: Syscall numbers (custom ABI).
///
/// These are intentionally different from Linux to avoid confusion
/// and to keep the implementation minimal.
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallNumber {
    /// Read from a file descriptor
    Read = 0,
    /// Write to a file descriptor
    Write = 1,
    /// Exit the process
    Exit = 2,
    /// Get process ID
    GetPid = 3,
    /// Adjust program break (heap)
    Sbrk = 4,
}

impl SyscallNumber {
    /// Convert a raw syscall number to the enum.
    pub fn from_u64(n: u64) -> Option<Self> {
        match n {
            0 => Some(Self::Read),
            1 => Some(Self::Write),
            2 => Some(Self::Exit),
            3 => Some(Self::GetPid),
            4 => Some(Self::Sbrk),
            _ => None,
        }
    }
}

/// TEAM_073: Saved user context during syscall.
///
/// When a syscall is invoked via `svc #0`, the exception handler saves
/// the user's registers so we can access arguments and return values.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SyscallFrame {
    /// General purpose registers x0-x30
    pub regs: [u64; 31],
    /// Stack pointer (SP_EL0)
    pub sp: u64,
    /// Program counter (ELR_EL1 - return address)
    pub pc: u64,
    /// Saved program status (SPSR_EL1)
    pub pstate: u64,
}

impl SyscallFrame {
    /// Get syscall number (x8)
    #[inline]
    pub fn syscall_number(&self) -> u64 {
        self.regs[8]
    }

    /// Get syscall argument 0 (x0)
    #[inline]
    pub fn arg0(&self) -> u64 {
        self.regs[0]
    }

    /// Get syscall argument 1 (x1)
    #[inline]
    pub fn arg1(&self) -> u64 {
        self.regs[1]
    }

    /// Get syscall argument 2 (x2)
    #[inline]
    pub fn arg2(&self) -> u64 {
        self.regs[2]
    }

    /// Get syscall argument 3 (x3)
    #[inline]
    #[allow(dead_code)]
    pub fn arg3(&self) -> u64 {
        self.regs[3]
    }

    /// Get syscall argument 4 (x4)
    #[inline]
    #[allow(dead_code)]
    pub fn arg4(&self) -> u64 {
        self.regs[4]
    }

    /// Get syscall argument 5 (x5)
    #[inline]
    #[allow(dead_code)]
    pub fn arg5(&self) -> u64 {
        self.regs[5]
    }

    /// Set return value (x0)
    #[inline]
    pub fn set_return(&mut self, value: i64) {
        self.regs[0] = value as u64;
    }
}

/// TEAM_073: Main syscall dispatch function.
///
/// Called from the exception handler when an SVC exception is detected.
/// Dispatches to the appropriate handler based on the syscall number.
///
/// # Arguments
/// * `frame` - Mutable reference to the saved user context
///
/// # Returns
/// The return value is stored in `frame.regs[0]` (x0).
pub fn syscall_dispatch(frame: &mut SyscallFrame) {
    let nr = frame.syscall_number();
    let result = match SyscallNumber::from_u64(nr) {
        Some(SyscallNumber::Read) => sys_read(
            frame.arg0() as usize, // fd
            frame.arg1() as usize, // buf
            frame.arg2() as usize, // len
        ),
        Some(SyscallNumber::Write) => sys_write(
            frame.arg0() as usize, // fd
            frame.arg1() as usize, // buf
            frame.arg2() as usize, // len
        ),
        Some(SyscallNumber::Exit) => sys_exit(frame.arg0() as i32),
        Some(SyscallNumber::GetPid) => sys_getpid(),
        Some(SyscallNumber::Sbrk) => sys_sbrk(frame.arg0() as isize),
        None => {
            // TEAM_073: Unknown syscall - return ENOSYS (Rule 14: Fail Fast, but don't crash)
            println!("[SYSCALL] Unknown syscall number: {}", nr);
            errno::ENOSYS
        }
    };

    frame.set_return(result);
}

// ============================================================================
// Syscall Implementations
// ============================================================================

/// TEAM_081: sys_read - Read from a file descriptor.
///
/// Supports:
/// - fd 0 (stdin) -> Keyboard input (VirtIO keyboard + UART)
///
/// This is a blocking read that waits for at least one character.
fn sys_read(fd: usize, buf: usize, len: usize) -> i64 {
    // Only stdin (fd 0) is supported
    if fd != 0 {
        return errno::EBADF;
    }

    // Validate buffer pointer (must be in user address space)
    if buf >= 0x0000_8000_0000_0000 {
        return errno::EFAULT;
    }

    if len == 0 {
        return 0;
    }

    // TEAM_081: Read from keyboard buffer
    // Try VirtIO keyboard first, then UART
    // Block until at least one character is available
    let mut bytes_read = 0usize;
    let max_read = len.min(4096); // Safety limit

    // SAFETY: We've validated the buffer is in user address space.
    let user_buf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, max_read) };

    // Block until we get at least one character
    while bytes_read == 0 {
        // Poll VirtIO input devices to process any pending events
        crate::input::poll();

        // Try to read from VirtIO keyboard buffer
        while bytes_read < max_read {
            if let Some(ch) = crate::input::read_char() {
                user_buf[bytes_read] = ch as u8;
                bytes_read += 1;
                // For line-buffered input, break on newline
                if ch == '\n' {
                    break;
                }
            } else {
                break;
            }
        }

        // Also check UART input
        if bytes_read < max_read {
            while let Some(byte) = levitate_hal::console::read_byte() {
                // Convert CR to LF for consistency
                let byte = if byte == b'\r' { b'\n' } else { byte };
                user_buf[bytes_read] = byte;
                bytes_read += 1;
                // For line-buffered input, break on newline
                if byte == b'\n' {
                    break;
                }
                if bytes_read >= max_read {
                    break;
                }
            }
        }

        // If no input yet, yield CPU briefly
        if bytes_read == 0 {
            core::hint::spin_loop();
        }
    }

    bytes_read as i64
}

/// TEAM_073: sys_write - Write to a file descriptor.
///
/// Supports:
/// - fd 1 (stdout) -> UART + GPU terminal
/// - fd 2 (stderr) -> UART + GPU terminal
///
/// Per Phase 2 decision: Console I/O goes to both backends.
fn sys_write(fd: usize, buf: usize, len: usize) -> i64 {
    // Validate file descriptor
    if fd != 1 && fd != 2 {
        return errno::EBADF;
    }

    // Validate buffer pointer (must be in user address space)
    // User space is 0x0000_0000_0000_0000 to 0x0000_7FFF_FFFF_FFFF
    if buf >= 0x0000_8000_0000_0000 {
        println!("[SYSCALL] write: buffer 0x{:x} not in user space", buf);
        return errno::EFAULT;
    }

    // Safety check: limit maximum write size
    let len = len.min(4096);

    // SAFETY: We've validated the buffer is in user address space.
    // The user's page tables should have this region mapped.
    // If not, we'll get a page fault (handled by exception handler).
    let slice = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };

    // Convert to string if valid UTF-8, otherwise print as hex
    if let Ok(s) = core::str::from_utf8(slice) {
        // TEAM_115: Use print! macro to go through dual console path (UART + GPU)
        // Previous code used WRITER.lock().write_str() which bypassed GPU output
        print!("{}", s);
    } else {
        // Binary data - print hex
        for byte in slice {
            print!("{:02x}", byte);
        }
    }

    len as i64
}

/// TEAM_073: sys_exit - Terminate the process.
///
/// Per Phase 2 decision: Print error and kill process (Option A).
fn sys_exit(code: i32) -> i64 {
    println!("[SYSCALL] exit({})", code);

    // Mark current task as exited
    // TODO(TEAM_073): Integrate with scheduler to actually terminate
    // For now, just loop forever (will be fixed in Step 5 integration)

    // TEAM_073: This should call task_exit() but we need proper integration first
    loop {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack));
        }
        #[cfg(not(target_arch = "aarch64"))]
        core::hint::spin_loop();
    }
}

/// TEAM_073: sys_getpid - Get process ID.
fn sys_getpid() -> i64 {
    // TODO(TEAM_073): Return actual PID from current UserTask
    // For now, return 1 (first user process)
    1
}

/// TEAM_073: sys_sbrk - Adjust program break (heap allocation).
fn sys_sbrk(_increment: isize) -> i64 {
    // TODO(TEAM_073): Implement heap management
    println!("[SYSCALL] sbrk({}) - not implemented", _increment);
    errno::ENOSYS
}

// ============================================================================
// Exception Handler Integration
// ============================================================================

/// TEAM_073: ESR_EL1 Exception Class for SVC (Supervisor Call).
pub const EC_SVC_AARCH64: u64 = 0b010101;

/// TEAM_073: Extract Exception Class from ESR_EL1.
#[inline]
pub fn esr_exception_class(esr: u64) -> u64 {
    (esr >> 26) & 0x3F
}

/// TEAM_073: Check if exception is an SVC from AArch64.
#[inline]
pub fn is_svc_exception(esr: u64) -> bool {
    esr_exception_class(esr) == EC_SVC_AARCH64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_number_conversion() {
        assert_eq!(SyscallNumber::from_u64(0), Some(SyscallNumber::Read));
        assert_eq!(SyscallNumber::from_u64(1), Some(SyscallNumber::Write));
        assert_eq!(SyscallNumber::from_u64(2), Some(SyscallNumber::Exit));
        assert_eq!(SyscallNumber::from_u64(3), Some(SyscallNumber::GetPid));
        assert_eq!(SyscallNumber::from_u64(4), Some(SyscallNumber::Sbrk));
        assert_eq!(SyscallNumber::from_u64(99), None);
    }

    #[test]
    fn test_esr_exception_class() {
        // EC is bits [31:26]
        let esr_svc = 0b010101 << 26;
        assert_eq!(esr_exception_class(esr_svc), EC_SVC_AARCH64);
        assert!(is_svc_exception(esr_svc));

        let esr_other = 0b100000 << 26;
        assert!(!is_svc_exception(esr_other));
    }

    #[test]
    fn test_syscall_frame() {
        let mut frame = SyscallFrame::default();
        frame.regs[8] = 1; // syscall number = write
        frame.regs[0] = 1; // fd = stdout
        frame.regs[1] = 0x1000; // buf
        frame.regs[2] = 5; // len

        assert_eq!(frame.syscall_number(), 1);
        assert_eq!(frame.arg0(), 1);
        assert_eq!(frame.arg1(), 0x1000);
        assert_eq!(frame.arg2(), 5);

        frame.set_return(-1);
        assert_eq!(frame.regs[0] as i64, -1);
    }
}
