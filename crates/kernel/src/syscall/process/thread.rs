//! Threading syscalls.
//!
//! TEAM_228: Threading syscalls for std support.
//! TEAM_230: sys_clone implementation.
//! TEAM_417: Extracted from process.rs.

use crate::memory::user as mm_user;
use crate::syscall::errno;

// Import clone flags from parent module
use super::{
    CLONE_CHILD_CLEARTID, CLONE_CHILD_SETTID, CLONE_PARENT_SETTID, CLONE_SETTLS, CLONE_THREAD,
    CLONE_VM,
};

/// TEAM_230: sys_clone - Create a new thread or process.
///
/// For std::thread support, we only implement the thread case where
/// CLONE_VM | CLONE_THREAD are set.
///
/// # Arguments
/// * `flags` - Clone flags
/// * `stack` - New stack pointer for child
/// * `parent_tid` - Address to write parent TID (if CLONE_PARENT_SETTID)
/// * `tls` - TLS pointer for child (if CLONE_SETTLS)
/// * `child_tid` - Address for child TID operations
///
/// # Returns
/// Child TID to parent, 0 to child, or negative errno.
pub fn sys_clone(
    flags: u64,
    stack: usize,
    parent_tid: usize,
    tls: usize,
    child_tid: usize,
    tf: &crate::arch::SyscallFrame,
) -> i64 {
    log::trace!(
        "[SYSCALL] clone(flags=0x{:x}, stack=0x{:x}, tls=0x{:x})",
        flags,
        stack,
        tls
    );

    // TEAM_230: Check if this is a thread-style clone
    let is_thread = (flags & CLONE_VM != 0) && (flags & CLONE_THREAD != 0);
    if !is_thread {
        // Fork-style not supported yet
        log::warn!("[SYSCALL] clone: fork-style clones not supported");
        return errno::ENOSYS;
    }

    // TEAM_230: Get parent task info
    let parent = crate::task::current_task();
    let parent_ttbr0 = parent.ttbr0;

    // TEAM_230: Determine TLS value
    let thread_tls = if flags & CLONE_SETTLS != 0 { tls } else { 0 };

    // TEAM_230: Determine clear_child_tid address
    let clear_tid = if flags & CLONE_CHILD_CLEARTID != 0 {
        child_tid
    } else {
        0
    };

    // TEAM_230: Create the thread
    // Pass TrapFrame so we can clone register state
    let child =
        match crate::task::thread::create_thread(parent_ttbr0, stack, thread_tls, clear_tid, tf) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("[SYSCALL] clone: create_thread failed: {:?}", e);
                return errno::ENOMEM;
            }
        };

    let child_tid_value = child.id.0;

    // TEAM_230: Handle CLONE_PARENT_SETTID - write child TID to parent's address
    if flags & CLONE_PARENT_SETTID != 0 && parent_tid != 0 {
        if let Some(ptr) = mm_user::user_va_to_kernel_ptr(parent_ttbr0, parent_tid) {
            // SAFETY: user_va_to_kernel_ptr verified the address is mapped
            // and belongs to this task's address space.
            unsafe {
                *(ptr as *mut i32) = child_tid_value as i32;
            }
        }
    }

    // TEAM_230: Handle CLONE_CHILD_SETTID - write child TID to child's address
    // Since CLONE_VM means shared address space, we can write it now
    if flags & CLONE_CHILD_SETTID != 0 && child_tid != 0 {
        if let Some(ptr) = mm_user::user_va_to_kernel_ptr(parent_ttbr0, child_tid) {
            // SAFETY: user_va_to_kernel_ptr verified the address is mapped.
            // Address is in shared address space (CLONE_VM).
            unsafe {
                *(ptr as *mut i32) = child_tid_value as i32;
            }
        }
    }

    // TEAM_230: Register in process table (as child of parent)
    let parent_pid = parent.id.0;
    crate::task::process_table::register_process(child_tid_value, parent_pid, child.clone());

    // TEAM_230: Add child to scheduler
    crate::task::scheduler::SCHEDULER.add_task(child);

    log::trace!(
        "[SYSCALL] clone: created thread TID={} for parent PID={}",
        child_tid_value,
        parent_pid
    );

    // TEAM_230: Return child TID to parent
    child_tid_value as i64
}

/// TEAM_228: sys_set_tid_address - Set pointer to thread ID.
///
/// # Arguments
/// * `tidptr` - Address to store TID, cleared on thread exit
///
/// # Returns
/// Current thread ID.
pub fn sys_set_tid_address(tidptr: usize) -> i64 {
    let task = crate::task::current_task();

    // Store the address for clear-on-exit
    task.clear_child_tid
        .store(tidptr, core::sync::atomic::Ordering::Release);

    // Return current TID
    task.id.0 as i64
}
