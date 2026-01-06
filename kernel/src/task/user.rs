//! TEAM_073: Userspace support for LevitateOS.
//!
//! This module provides mechanisms for:
//! - Transitioning from kernel mode (EL1) to user mode (EL0)
//! - Managing user task state
//! - User address space management
//!
//! TEAM_158: Behavior IDs [PROC3]-[PROC4] for traceability.
//!
//! Reference: ARM Architecture Reference Manual, Exception Levels

/// [PROC3] Enter user mode (EL0) at the specified entry point.
/// [PROC4] User TTBR0 must be set before calling this.
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
pub use crate::arch::enter_user_mode;

// ============================================================================
// User Task Structure (Step 1 UoW 2)
// ============================================================================

use alloc::boxed::Box;
use alloc::vec;
use core::sync::atomic::{AtomicU8, Ordering};

extern crate alloc;

/// TEAM_073: Default stack size for user tasks (64KB).
#[allow(dead_code)]
pub const USER_STACK_SIZE: usize = 65536;

/// TEAM_073: User address space layout constants.
pub mod layout {
    /// User code starts after NULL guard page
    #[allow(dead_code)]
    pub const USER_CODE_START: usize = 0x0000_0000_0001_0000; // 64KB

    /// User heap starts after code/data (set by ELF loader)
    #[allow(dead_code)]
    pub const USER_HEAP_START: usize = 0x0000_0000_1000_0000; // 256MB (placeholder)

    /// TEAM_166: Maximum heap size per process (64MB)
    pub const USER_HEAP_MAX_SIZE: usize = 64 * 1024 * 1024;
}

use crate::memory::heap::ProcessHeap;

extern crate alloc;
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
    #[allow(dead_code)]
    Ready = 0,
    /// Currently executing
    #[allow(dead_code)]
    Running = 1,
    /// Waiting for I/O or event
    #[allow(dead_code)]
    Blocked = 2,
    /// Process has terminated
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    state: AtomicU8,

    /// Physical address of L0 page table (for TTBR0_EL1)
    pub ttbr0: usize,

    /// User stack pointer (SP_EL0)
    pub user_sp: usize,

    /// Entry point from ELF (used for initial `enter_user_mode`)
    pub entry_point: usize,

    /// TEAM_166: Process heap state for sbrk syscall
    pub heap: ProcessHeap,

    /// Exit code (set by exit syscall)
    #[allow(dead_code)]
    pub exit_code: i32,

    /// Kernel stack for this process (used during syscalls/exceptions)
    #[allow(dead_code)]
    pub kernel_stack: Box<[u64]>,

    /// Top of kernel stack
    #[allow(dead_code)]
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

        // TEAM_166: Initialize heap with brk from ELF loader
        let heap = ProcessHeap::new(brk);

        Self {
            pid: Pid::next(),
            state: AtomicU8::new(ProcessState::Ready as u8),
            ttbr0,
            user_sp,
            entry_point,
            heap, // TEAM_166: Replaces simple brk field
            exit_code: 0,
            kernel_stack,
            kernel_stack_top,
        }
    }

    /// Get the current state of the process.
    #[allow(dead_code)]
    pub fn state(&self) -> ProcessState {
        match self.state.load(Ordering::Acquire) {
            0 => ProcessState::Ready,
            1 => ProcessState::Running,
            2 => ProcessState::Blocked,
            _ => ProcessState::Exited,
        }
    }

    /// Set the state of the process.
    #[allow(dead_code)]
    pub fn set_state(&self, state: ProcessState) {
        self.state.store(state as u8, Ordering::Release);
    }

    /// Mark the process as exited with the given code.
    #[allow(dead_code)]
    pub fn exit(&self, _code: i32) {
        // SAFETY: AtomicU8 store is safe
        self.state.store(
            ProcessState::Exited as u8,
            core::sync::atomic::Ordering::Release,
        );
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
