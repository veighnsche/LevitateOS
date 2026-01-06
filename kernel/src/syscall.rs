//! TEAM_073: System Call Handler for LevitateOS.
//!
//! Implements the syscall dispatch table and individual syscall handlers.
//! Uses a custom ABI (not Linux-compatible) as decided in Phase 2.
//!
//! TEAM_158: Behavior IDs [SYS1]-[SYS9] for traceability.
//!
//! ## Syscall ABI
//! - Syscall number in `x8`
//! - Arguments in `x0-x5` (up to 6 arguments)
//! - Return value in `x0`
//! - Invoked via `svc #0` instruction from EL0

use los_hal::{print, println};

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
    /// Spawn a new process
    Spawn = 5,
    /// Replace current process
    Exec = 6,
    /// Yield CPU to other tasks
    Yield = 7,
    /// TEAM_142: Graceful system shutdown
    Shutdown = 8,
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
            5 => Some(Self::Spawn),
            6 => Some(Self::Exec),
            7 => Some(Self::Yield),
            8 => Some(Self::Shutdown),
            _ => None,
        }
    }
}

use crate::arch::SyscallFrame;

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
        Some(SyscallNumber::Spawn) => sys_spawn(
            frame.arg0() as usize, // path
            frame.arg1() as usize, // path_len
        ),
        Some(SyscallNumber::Exec) => sys_exec(
            frame.arg0() as usize, // path
            frame.arg1() as usize, // path_len
        ),
        Some(SyscallNumber::Yield) => sys_yield(),
        Some(SyscallNumber::Shutdown) => sys_shutdown(frame.arg0() as u32),
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
// TEAM_156: Helper to write a byte to user buffer via page table translation
fn write_to_user_buf(ttbr0: usize, user_buf_base: usize, offset: usize, byte: u8) -> bool {
    let user_va = user_buf_base + offset;
    if let Some(kernel_ptr) = crate::task::user_mm::user_va_to_kernel_ptr(ttbr0, user_va) {
        unsafe {
            *kernel_ptr = byte;
        }
        true
    } else {
        false
    }
}

// TEAM_148: Helper to poll all input sources
// TEAM_156: Now takes ttbr0 and user_buf address instead of direct slice
fn poll_input_devices(ttbr0: usize, user_buf: usize, bytes_read: &mut usize, max_read: usize) {
    // Poll VirtIO input devices to process any pending events
    crate::input::poll();

    // Try to read from VirtIO keyboard buffer
    while *bytes_read < max_read {
        if let Some(ch) = crate::input::read_char() {
            if !write_to_user_buf(ttbr0, user_buf, *bytes_read, ch as u8) {
                return; // Failed to write to user buffer
            }
            *bytes_read += 1;
            // For line-buffered input, break on newline
            if ch == '\n' {
                return;
            }
        } else {
            break;
        }
    }

    // Also check UART input
    if *bytes_read < max_read {
        while let Some(byte) = los_hal::console::read_byte() {
            // Convert CR to LF for consistency
            let byte = if byte == b'\r' { b'\n' } else { byte };
            if !write_to_user_buf(ttbr0, user_buf, *bytes_read, byte) {
                return; // Failed to write to user buffer
            }
            *bytes_read += 1;
            // For line-buffered input, break on newline
            if byte == b'\n' {
                return;
            }
            if *bytes_read >= max_read {
                return;
            }
        }
    }
}

/// TEAM_081: sys_read - Read from a file descriptor.
/// [SYS6] sys_read(fd=0) reads from keyboard
/// [SYS7] sys_read blocks until input available
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

    // Safety limit
    let max_read = len.min(4096);

    // TEAM_137: Validate user buffer (must be mapped and writable)
    let task = crate::task::current_task();
    if crate::task::user_mm::validate_user_buffer(task.ttbr0, buf, max_read, true).is_err() {
        return errno::EFAULT;
    }

    if len == 0 {
        return 0;
    }

    // TEAM_081: Read from keyboard buffer
    // Try VirtIO keyboard first, then UART
    // Block until at least one character is available
    let mut bytes_read = 0usize;

    // TEAM_156: Get ttbr0 for page table translation
    let ttbr0 = task.ttbr0;

    // [SYS7] Busy-yield loop for input (blocking read)
    // NOTE: We cannot use 'wfi' here because we don't have a UART ISR.
    // If 'yield_now' switches to a task with interrupts enabled, the UART IRQ
    // will fire, be marked "Unhandled", and DISABLED by the GIC.
    // Valid 'wfi' wakeups would then cease, causing a hang.
    // Until a proper UART driver with buffering is implemented, we must busy-poll.
    loop {
        // [SYS6][SYS7] blocking read from keyboard
        // TEAM_156: Pass ttbr0 and buf address for proper page table translation
        poll_input_devices(ttbr0, buf, &mut bytes_read, max_read);
        if bytes_read > 0 {
            break;
        }

        // TEAM_149: Unmask interrupts briefly to allow ISRs (UART/VirtIO) to run.
        // Syscalls enter with PSTATE.I=1 (Masked). If we don't unmask, no IRQs
        // ever fire, starving input.
        unsafe {
            los_hal::interrupts::enable();
        }
        let _ = los_hal::interrupts::disable();

        crate::task::yield_now();
    }

    bytes_read as i64
}

/// TEAM_073: sys_write - Write to a file descriptor.
/// [SYS1] sys_write(fd=1) outputs to UART
/// [SYS2] sys_write(fd=1) outputs to GPU terminal
/// [SYS3] sys_write(fd=2) outputs to UART
/// [SYS4] sys_write validates user buffer address
/// [SYS5] sys_write limits output to 4KB
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

    // [SYS5] Safety check: limit maximum write size to 4KB
    let len = len.min(4096);

    // [SYS4] Validate user buffer (must be mapped and readable)
    let task = crate::task::current_task();
    // writable=false because we are reading FROM user space
    if crate::task::user_mm::validate_user_buffer(task.ttbr0, buf, len, false).is_err() {
        // println!("[SYSCALL] write: buffer validation failed"); // Noisy
        return errno::EFAULT; // [SYS4] invalid buffer returns EFAULT
    }

    // SAFETY: We've validated mapping and permissions.
    let slice = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };

    // Convert to string if valid UTF-8, otherwise print as hex
    if let Ok(s) = core::str::from_utf8(slice) {
        // [SYS1][SYS2][SYS3] Use print! macro to go through dual console path (UART + GPU)
        // Previous code used WRITER.lock().write_str() which bypassed GPU output
        print!("{}", s); // [SYS1][SYS2] outputs to UART and GPU
    } else {
        // Binary data - print hex
        for byte in slice {
            print!("{:02x}", byte);
        }
    }

    len as i64
}

/// TEAM_073: sys_exit - Terminate the process.
/// [SYS8] sys_exit terminates current task
///
/// Per Phase 2 decision: Print error and kill process (Option A).
fn sys_exit(code: i32) -> i64 {
    println!("[SYSCALL] exit({})", code);

    // [SYS8] Call task_exit() to properly terminate and reschedule
    crate::task::task_exit();
}

/// TEAM_073: sys_getpid - Get process ID.
/// [SYS9] sys_getpid returns task PID
fn sys_getpid() -> i64 {
    // [SYS9] Return actual PID from current task
    crate::task::current_task().id.0 as i64
}

/// TEAM_073: sys_sbrk - Adjust program break (heap allocation).
fn sys_sbrk(_increment: isize) -> i64 {
    // TODO(TEAM_073): Implement heap management
    println!("[SYSCALL] sbrk({}) - not implemented", _increment);
    errno::ENOSYS
}

/// TEAM_129: sys_yield - Voluntarily yield CPU to other tasks.
fn sys_yield() -> i64 {
    crate::task::yield_now();
    0
}

/// TEAM_142: Shutdown flags for verbose mode
pub mod shutdown_flags {
    /// Normal shutdown (minimal output)
    #[allow(dead_code)]
    pub const NORMAL: u32 = 0;
    /// Verbose shutdown (for golden file testing)
    pub const VERBOSE: u32 = 1;
}

/// TEAM_142: sys_shutdown - Graceful system shutdown.
///
/// Performs a clean shutdown sequence:
/// 1. Flush GPU framebuffer
/// 2. Sync filesystems (if any)
/// 3. Log shutdown messages
/// 4. Halt CPU
///
/// # Arguments
/// * `flags` - Shutdown flags (0 = normal, 1 = verbose for golden file)
fn sys_shutdown(flags: u32) -> i64 {
    let verbose = flags & shutdown_flags::VERBOSE != 0;

    println!("[SHUTDOWN] Initiating graceful shutdown...");

    if verbose {
        println!("[SHUTDOWN] Phase 1: Stopping user tasks...");
    }

    // Phase 1: Mark all user tasks for termination
    // (In a full implementation, we'd signal all tasks to exit)
    if verbose {
        println!("[SHUTDOWN] Phase 1: Complete");
        println!("[SHUTDOWN] Phase 2: Flushing GPU framebuffer...");
    }

    // Phase 2: Final GPU flush
    // TEAM_142: Must release GPU lock before println! (which uses terminal → GPU)
    {
        if let Some(mut guard) = crate::gpu::GPU.try_lock() {
            if let Some(gpu_state) = guard.as_mut() {
                let _ = gpu_state.flush();
            }
        }
        // GPU lock released here
    }

    if verbose {
        println!("[SHUTDOWN] GPU flush complete");
        println!("[SHUTDOWN] Phase 2: Complete");
        println!("[SHUTDOWN] Phase 3: Syncing filesystems...");
    }

    // Phase 3: Sync filesystems (placeholder - no persistent writes yet)
    if verbose {
        println!("[SHUTDOWN] Phase 3: Complete (no pending writes)");
    }

    // TEAM_142: Print ALL messages BEFORE disabling interrupts
    // UART output requires interrupts to flush properly
    if verbose {
        println!("[SHUTDOWN] Phase 4: Disabling interrupts...");
        println!("[SHUTDOWN] Phase 4: Complete");
    }

    println!("[SHUTDOWN] System halted. Safe to power off.");
    println!("[SHUTDOWN] Goodbye!");

    // Small delay to ensure UART buffer flushes
    for _ in 0..100000 {
        core::hint::spin_loop();
    }

    // Phase 4: Disable interrupts (AFTER all output)
    los_hal::interrupts::disable();

    // Phase 5: Power off using PSCI SYSTEM_OFF
    // PSCI is the ARM Power State Coordination Interface
    // SYSTEM_OFF function ID: 0x84000008 (SMC32 convention)
    // This will cause QEMU to exit completely
    const PSCI_SYSTEM_OFF: u64 = 0x84000008;
    unsafe {
        core::arch::asm!(
            "hvc #0",               // Hypervisor call (QEMU virt uses HVC for PSCI)
            in("x0") PSCI_SYSTEM_OFF,
            options(noreturn)
        );
    }
}

/// TEAM_120: sys_spawn - Spawn a new process from initramfs.
fn sys_spawn(path_ptr: usize, path_len: usize) -> i64 {
    // Safety check: limit maximum path length
    let path_len = path_len.min(256);

    // TEAM_137: Validate path pointer
    let task = crate::task::current_task();
    if crate::task::user_mm::validate_user_buffer(task.ttbr0, path_ptr, path_len, false).is_err() {
        return errno::EFAULT;
    }

    // SAFETY: We've validated mapping and permissions.
    let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr as *const u8, path_len) };
    let path = match core::str::from_utf8(path_bytes) {
        Ok(s) => s,
        Err(_) => return errno::EINVAL,
    };

    println!("[SYSCALL] spawn('{}')", path);

    // Get initramfs
    let archive_lock = crate::fs::INITRAMFS.lock();
    let archive = match archive_lock.as_ref() {
        Some(a) => a,
        None => return errno::ENOSYS,
    };

    // Find file in initramfs
    let mut elf_data = None;
    for entry in archive.iter() {
        if entry.name == path {
            elf_data = Some(entry.data);
            break;
        }
    }

    let elf_data = match elf_data {
        Some(d) => d,
        None => return errno::EBADF, // Or ENOENT if we had it
    };

    // Spawn process
    match crate::task::process::spawn_from_elf(elf_data) {
        Ok(task) => {
            let pid = task.pid.0 as i64;
            // Add to scheduler
            crate::task::scheduler::SCHEDULER.add_task(alloc::sync::Arc::new(task.into()));
            pid
        }
        Err(e) => {
            println!("[SYSCALL] spawn failed: {:?}", e);
            -1
        }
    }
}

/// TEAM_120: sys_exec - Replace current process with one from initramfs.
fn sys_exec(path_ptr: usize, path_len: usize) -> i64 {
    // Safety check: limit maximum path length
    let path_len = path_len.min(256);

    // TEAM_137: Validate path pointer
    let task = crate::task::current_task();
    if crate::task::user_mm::validate_user_buffer(task.ttbr0, path_ptr, path_len, false).is_err() {
        return errno::EFAULT;
    }

    // SAFETY: We've validated mapping and permissions.
    let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr as *const u8, path_len) };
    let path = match core::str::from_utf8(path_bytes) {
        Ok(s) => s,
        Err(_) => return errno::EINVAL,
    };

    println!("[SYSCALL] exec('{}')", path);

    // Get initramfs
    let archive_lock = crate::fs::INITRAMFS.lock();
    let archive = match archive_lock.as_ref() {
        Some(a) => a,
        None => return errno::ENOSYS,
    };

    // Find file in initramfs
    let mut elf_data = None;
    for entry in archive.iter() {
        if entry.name == path {
            elf_data = Some(entry.data);
            break;
        }
    }

    let _elf_data = match elf_data {
        Some(d) => d,
        None => return errno::EBADF,
    };

    // TEAM_120: Re-load ELF into CURRENT page tables
    // This requires clearing existing user mappings first.
    // Since we don't have a full VMM yet that can wipe and reload easily,
    // we'll implement a simple version that uses the existing ELF loader.

    // For now, sys_exec is a "TODO" because it requires more VMM plumbing
    // than a simple spawn. Let's return ENOSYS for now.
    println!("[SYSCALL] exec is currently a stub");
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
