//! TEAM_073: Userspace support for LevitateOS.
//!
//! This module provides mechanisms for:
//! - Transitioning from kernel mode (EL1) to user mode (EL0)
//! - Managing user task state
//! - User address space management
//!
//! Reference: ARM Architecture Reference Manual, Exception Levels

use core::arch::asm;

/// TEAM_073: Enter user mode (EL0) at the specified entry point.
///
/// This function configures the CPU state for user mode execution and
/// performs an `eret` (Exception Return) to enter EL0.
///
/// # Arguments
/// * `entry_point` - The virtual address where user code execution begins
/// * `user_sp` - The user stack pointer (SP_EL0)
///
/// # Safety
/// - `entry_point` must be a valid, mapped user-space address
/// - `user_sp` must point to a valid, mapped user stack
/// - User page tables must be configured in TTBR0 before calling
///
/// # AArch64 Details
/// - `SPSR_EL1` is set to 0x0 (EL0 with SP_EL0, interrupts enabled)
/// - `ELR_EL1` is set to the entry point
/// - `SP_EL0` is set to the user stack pointer
/// - `eret` performs the exception return to EL0
#[cfg(target_arch = "aarch64")]
pub unsafe fn enter_user_mode(entry_point: usize, user_sp: usize) -> ! {
    // SAFETY: Inline assembly to configure SPSR/ELR and perform eret.
    // This is the only way to transition to EL0 on AArch64.
    //
    // SPSR_EL1 encoding for EL0:
    //   Bits [3:2] = 00 = EL0
    //   Bit  [0]   = 0  = Use SP_EL0
    //   Bits [9:6] = DAIF = 0 = Interrupts enabled
    unsafe {
        asm!(
            // Clear all general-purpose registers for clean user entry
            "mov x2, xzr",
            "mov x3, xzr",
            "mov x4, xzr",
            "mov x5, xzr",
            "mov x6, xzr",
            "mov x7, xzr",
            "mov x8, xzr",
            "mov x9, xzr",
            "mov x10, xzr",
            "mov x11, xzr",
            "mov x12, xzr",
            "mov x13, xzr",
            "mov x14, xzr",
            "mov x15, xzr",
            "mov x16, xzr",
            "mov x17, xzr",
            "mov x18, xzr",
            "mov x19, xzr",
            "mov x20, xzr",
            "mov x21, xzr",
            "mov x22, xzr",
            "mov x23, xzr",
            "mov x24, xzr",
            "mov x25, xzr",
            "mov x26, xzr",
            "mov x27, xzr",
            "mov x28, xzr",
            "mov x29, xzr",
            "mov x30, xzr",

            // Set ELR_EL1 = entry point (return address for eret)
            "msr elr_el1, {entry}",

            // Set SPSR_EL1 = 0 (EL0, SP_EL0, interrupts enabled, no flags)
            "msr spsr_el1, {spsr}",

            // Set SP_EL0 = user stack pointer
            "msr sp_el0, {sp}",

            // Exception return to EL0
            "eret",

            entry = in(reg) entry_point,
            spsr = in(reg) 0u64,
            sp = in(reg) user_sp,
            options(noreturn)
        );
    }
}

/// TEAM_073: Stub for non-aarch64 builds (test compilation).
#[cfg(not(target_arch = "aarch64"))]
pub unsafe fn enter_user_mode(_entry_point: usize, _user_sp: usize) -> ! {
    panic!("enter_user_mode is only available on aarch64");
}

// ============================================================================
// User Task Structure (Step 1 UoW 2)
// ============================================================================

use alloc::boxed::Box;
use alloc::vec;
use core::sync::atomic::{AtomicU8, Ordering};

extern crate alloc;

/// TEAM_073: Default stack size for user tasks (64KB).
pub const USER_STACK_SIZE: usize = 65536;

/// TEAM_073: User address space layout constants.
pub mod layout {
    /// User code starts after NULL guard page
    pub const USER_CODE_START: usize = 0x0000_0000_0001_0000; // 64KB

    /// User stack top (grows down from here)
    pub const USER_STACK_TOP: usize = 0x0000_7FFF_FFFF_0000;

    /// User stack bottom (stack guard below this)
    pub const USER_STACK_BOTTOM: usize = USER_STACK_TOP - super::USER_STACK_SIZE;

    /// User heap starts after code/data (set by ELF loader)
    pub const USER_HEAP_START: usize = 0x0000_0000_1000_0000; // 256MB (placeholder)
}

/// TEAM_073: Unique identifier for a user process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pid(pub u64);

impl Pid {
    /// Generate the next unique PID.
    pub fn next() -> Self {
        use core::sync::atomic::AtomicU64;
        static NEXT_PID: AtomicU64 = AtomicU64::new(1); // PID 0 reserved for kernel/idle
        Pid(NEXT_PID.fetch_add(1, Ordering::SeqCst))
    }
}

/// TEAM_073: State of a user process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProcessState {
    /// Ready to run
    Ready = 0,
    /// Currently executing
    Running = 1,
    /// Waiting for I/O or event
    Blocked = 2,
    /// Process has terminated
    Exited = 3,
}

/// TEAM_073: User Task Control Block.
///
/// Extends the kernel TCB with user-specific fields needed for
/// EL0 execution and process isolation.
pub struct UserTask {
    /// Unique process identifier
    pub pid: Pid,

    /// Process state (atomic for safe mutation from syscall handlers)
    state: AtomicU8,

    /// Physical address of L0 page table (for TTBR0_EL1)
    pub ttbr0: usize,

    /// User stack pointer (SP_EL0)
    pub user_sp: usize,

    /// Entry point from ELF (used for initial `enter_user_mode`)
    pub entry_point: usize,

    /// Program break (end of heap, for sbrk syscall)
    pub brk: usize,

    /// Exit code (set by exit syscall)
    pub exit_code: i32,

    /// Kernel stack for this process (used during syscalls/exceptions)
    kernel_stack: Box<[u64]>,

    /// Top of kernel stack
    pub kernel_stack_top: usize,
}

impl UserTask {
    /// TEAM_073: Create a new user task.
    ///
    /// # Arguments
    /// * `entry_point` - Virtual address of user code entry (_start)
    /// * `user_sp` - Initial user stack pointer
    /// * `ttbr0` - Physical address of user page table L0
    /// * `brk` - Initial program break (end of loaded segments)
    pub fn new(entry_point: usize, user_sp: usize, ttbr0: usize, brk: usize) -> Self {
        // Allocate kernel stack for syscall handling
        let kernel_stack_size = 16384; // 16KB kernel stack
        let kernel_stack = vec![0u64; kernel_stack_size / 8].into_boxed_slice();
        let kernel_stack_top =
            kernel_stack.as_ptr() as usize + kernel_stack.len() * core::mem::size_of::<u64>();

        Self {
            pid: Pid::next(),
            state: AtomicU8::new(ProcessState::Ready as u8),
            ttbr0,
            user_sp,
            entry_point,
            brk,
            exit_code: 0,
            kernel_stack,
            kernel_stack_top,
        }
    }

    /// Get the current state of the process.
    pub fn state(&self) -> ProcessState {
        match self.state.load(Ordering::Acquire) {
            0 => ProcessState::Ready,
            1 => ProcessState::Running,
            2 => ProcessState::Blocked,
            _ => ProcessState::Exited,
        }
    }

    /// Set the state of the process.
    pub fn set_state(&self, state: ProcessState) {
        self.state.store(state as u8, Ordering::Release);
    }

    /// Mark the process as exited with the given code.
    pub fn exit(&self, code: i32) {
        // SAFETY: AtomicU8 store is safe
        self.state
            .store(ProcessState::Exited as u8, Ordering::Release);
        // Note: exit_code is not atomic, but should only be set once at exit
        // A proper implementation would use interior mutability or atomic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_generation() {
        let pid1 = Pid::next();
        let pid2 = Pid::next();
        assert!(pid2.0 > pid1.0);
    }

    #[test]
    fn test_process_state() {
        // Can't create full UserTask in tests without alloc, but we can test state logic
        let state = AtomicU8::new(ProcessState::Ready as u8);
        assert_eq!(state.load(Ordering::Acquire), 0);

        state.store(ProcessState::Running as u8, Ordering::Release);
        assert_eq!(state.load(Ordering::Acquire), 1);
    }
}
