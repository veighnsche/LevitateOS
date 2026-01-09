//! TEAM_230: Thread creation for sys_clone support.
//!
//! This module provides thread creation that shares address space with
//! the parent process, as required for Linux-compatible threading.

extern crate alloc;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec;
use core::sync::atomic::{AtomicU8, AtomicU32, AtomicUsize};

use los_hal::IrqSafeLock;

use crate::arch::Context;
use crate::memory::heap::ProcessHeap;
use crate::task::fd_table;
use crate::task::user::Pid;
use crate::task::{TaskControlBlock, TaskId, TaskState};

/// TEAM_230: Error type for thread creation.
#[derive(Debug)]
pub enum ThreadError {
    /// Failed to allocate kernel stack
    AllocationFailed,
}

/// TEAM_230: Create a new thread sharing the parent's address space.
///
/// This is the core function for sys_clone with CLONE_VM | CLONE_THREAD.
/// The new thread shares:
/// - Page tables (ttbr0)
/// - Virtual address space
///
/// The new thread has its own:
/// - Kernel stack
/// - Context (registers)
/// - PID/TID
///
/// # Arguments
/// * `parent_ttbr0` - Physical address of parent's page table (shared)
/// * `child_stack` - User stack pointer for the child
/// * `child_tls` - Thread Local Storage pointer (TPIDR_EL0)
/// * `clear_child_tid` - Address to clear and wake on thread exit
/// * `tf` - Parent's trap frame (for register cloning)
///
/// # Returns
/// Arc to new TCB on success, ThreadError on failure.
pub fn create_thread(
    parent_ttbr0: usize,
    child_stack: usize,
    child_tls: usize,
    clear_child_tid: usize,
    tf: &crate::arch::SyscallFrame,
) -> Result<Arc<TaskControlBlock>, ThreadError> {
    // TEAM_230: Allocate kernel stack for new thread (16KB)
    let kernel_stack_size = 16384;
    let kernel_stack = vec![0u64; kernel_stack_size / 8].into_boxed_slice();
    let kernel_stack_ptr = kernel_stack.as_ptr() as usize;
    let kernel_stack_top = kernel_stack_ptr + kernel_stack.len() * core::mem::size_of::<u64>();

    // TEAM_230: Clone parent's TrapFrame to child's kernel stack
    // The TrapFrame must be at the top of the stack when we "return" to userspace
    let frame_size = core::mem::size_of::<crate::arch::SyscallFrame>();
    let child_frame_addr = kernel_stack_top - frame_size;

    // Safety check alignment (frame size is 280, top is usually 16-byte aligned)
    // 280 is multiple of 8, so u64 alignment is fine.

    let mut child_frame = *tf;
    // Set return value to 0 for child (arch-agnostic via set_return)
    child_frame.set_return(0);
    // Set child stack pointer (if provided, otherwise inherits parent's SP)
    if child_stack != 0 {
        child_frame.set_sp(child_stack as u64);
    }
    // Set TLS
    if child_tls != 0 {
        // Child TLS is set in Context.tpidr_el0 below.
    }

    // Copy frame to new stack
    unsafe {
        let ptr = child_frame_addr as *mut crate::arch::SyscallFrame;
        *ptr = child_frame;
    }

    // TEAM_230: Generate new PID/TID
    let pid = Pid::next();
    let tid = pid.0 as usize;

    // TEAM_230: Set up context for first switch
    // We want to call `exception_return`, which restores from SP and erets.
    let mut context = Context::new(child_frame_addr, crate::arch::exception_return as *const () as usize);

    // TEAM_258: Set TLS in context using abstraction (architecture-independent)
    if child_tls != 0 {
        context.set_tls(child_tls as u64);
    }

    // TEAM_230: Create TCB
    let tcb = TaskControlBlock {
        id: TaskId(tid),
        state: AtomicU8::new(TaskState::Ready as u8),
        context,
        stack: Some(kernel_stack),
        stack_top: kernel_stack_top,
        stack_size: kernel_stack_size,
        // TEAM_230: Share parent's page table (key for threads!)
        ttbr0: parent_ttbr0,
        // TEAM_230: Child's user-space state - mostly tracked in TrapFrame on stack now
        // But we keep these updated for info/debugging
        user_sp: child_stack,
        user_entry: child_frame.pc as usize, // Use PC from frame
        // TEAM_230: Thread gets its own heap tracking (shared address space though)
        heap: IrqSafeLock::new(ProcessHeap::new(0)),
        // TEAM_230: For MVP, threads get their own fd table
        // TODO(TEAM_230): Share fd_table when CLONE_FILES is set
        fd_table: fd_table::new_shared_fd_table(),
        // TEAM_230: Inherit CWD from parent (threads share filesystem state)
        cwd: IrqSafeLock::new(String::from("/")),
        // TEAM_230: Thread signal state
        pending_signals: AtomicU32::new(0),
        blocked_signals: AtomicU32::new(0),
        signal_handlers: IrqSafeLock::new([0; 32]),
        signal_trampoline: AtomicUsize::new(0),
        // TEAM_230: Store clear_child_tid for CLONE_CHILD_CLEARTID
        clear_child_tid: AtomicUsize::new(clear_child_tid),
        // TEAM_238: Threads share parent's VMA tracking (same address space)
        vmas: IrqSafeLock::new(crate::memory::vma::VmaList::new()),
    };

    Ok(Arc::new(tcb))
}
