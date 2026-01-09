//! TEAM_208: Synchronization syscalls (futex)
//!
//! Futex (Fast Userspace Mutex) enables efficient blocking synchronization
//! in userspace. Tasks wait for a memory location's value to change without
//! burning CPU cycles.

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};
use los_hal::IrqSafeLock;

use crate::memory::user as mm_user;
use crate::syscall::errno;
use crate::task::scheduler::SCHEDULER;
use crate::task::{TaskControlBlock, TaskState, current_task, yield_now};

/// TEAM_208: Futex operations
pub const FUTEX_WAIT: usize = 0;
pub const FUTEX_WAKE: usize = 1;

/// TEAM_208: A task waiting on a futex
struct FutexWaiter {
    task: Arc<TaskControlBlock>,
}

/// TEAM_208: Global wait list: virtual address → list of waiters
/// Uses BTreeMap instead of HashMap (no hashbrown dependency)
static FUTEX_WAITERS: IrqSafeLock<BTreeMap<usize, Vec<FutexWaiter>>> =
    IrqSafeLock::new(BTreeMap::new());

/// TEAM_208: sys_futex - Fast userspace mutex operations
///
/// # Arguments
/// - `addr`: User virtual address of the futex word (must be 4-byte aligned)
/// - `op`: Operation (FUTEX_WAIT or FUTEX_WAKE)
/// - `val`: Expected value (for WAIT) or max waiters to wake (for WAKE)
/// - `_timeout`: Timeout in nanoseconds (currently ignored)
/// - `_addr2`: Second address (for REQUEUE, currently ignored)
///
/// # Returns
/// - FUTEX_WAIT: 0 on success, EAGAIN if value mismatch, EFAULT if bad address
/// - FUTEX_WAKE: Number of tasks woken
pub fn sys_futex(addr: usize, op: usize, val: usize, _timeout: usize, _addr2: usize) -> i64 {
    match op {
        FUTEX_WAIT => futex_wait(addr, val as u32),
        FUTEX_WAKE => futex_wake(addr, val),
        _ => errno::EINVAL,
    }
}

/// TEAM_208: Block the current task if *addr == expected
fn futex_wait(addr: usize, expected: u32) -> i64 {
    // Must be 4-byte aligned
    if addr % 4 != 0 {
        return errno::EINVAL;
    }

    let task = current_task();
    let ttbr0 = task.ttbr0;

    // Read the current value at the user address
    let Some(kernel_ptr) = mm_user::user_va_to_kernel_ptr(ttbr0, addr) else {
        return errno::EFAULT;
    };

    // Read atomically
    // SAFETY: We validated the address is mapped and aligned
    let current_val = unsafe {
        let atomic_ptr = kernel_ptr as *const AtomicU32;
        (*atomic_ptr).load(Ordering::SeqCst)
    };

    // If value doesn't match, return immediately
    if current_val != expected {
        return -11; // EAGAIN
    }

    // Add to wait list and block
    {
        let mut waiters = FUTEX_WAITERS.lock();
        waiters
            .entry(addr)
            .or_insert_with(Vec::new)
            .push(FutexWaiter { task: task.clone() });

        // Mark task as blocked
        task.set_state(TaskState::Blocked);
    }

    // Yield to scheduler - we won't be picked up again until unblocked
    yield_now();

    0
}

/// TEAM_208: Wake up to `count` tasks waiting on addr
/// TEAM_230: Made public for CLONE_CHILD_CLEARTID thread exit handling
pub fn futex_wake(addr: usize, count: usize) -> i64 {
    let mut woken = 0usize;

    let mut waiters = FUTEX_WAITERS.lock();

    if let Some(queue) = waiters.get_mut(&addr) {
        while !queue.is_empty() && woken < count {
            let waiter = queue.swap_remove(0);
            // Mark task as ready and add back to scheduler
            waiter.task.set_state(TaskState::Ready);
            SCHEDULER.add_task(waiter.task);
            woken += 1;
        }

        // Clean up empty queue
        if queue.is_empty() {
            waiters.remove(&addr);
        }
    }

    woken as i64
}
