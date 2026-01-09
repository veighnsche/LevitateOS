//! TEAM_208: Synchronization syscalls (futex)
//! TEAM_360: Added ppoll syscall for Eyra/std support
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

// TEAM_360: Poll event constants (matching Linux)
pub const POLLIN: i16 = 0x0001;   // Data to read
pub const POLLPRI: i16 = 0x0002;  // Urgent data
pub const POLLOUT: i16 = 0x0004;  // Writing possible
pub const POLLERR: i16 = 0x0008;  // Error (output only)
pub const POLLHUP: i16 = 0x0010;  // Hang up (output only)
pub const POLLNVAL: i16 = 0x0020; // Invalid fd (output only)

/// TEAM_360: struct pollfd (8 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Pollfd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

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

/// TEAM_360: sys_ppoll - Wait for events on file descriptors.
///
/// This implements the ppoll syscall for Eyra/std support.
/// Currently implements non-blocking poll that checks fd state immediately.
///
/// # Arguments
/// * `fds_ptr` - User pointer to array of pollfd structs
/// * `nfds` - Number of file descriptors
/// * `tmo_ptr` - User pointer to timeout (NULL = block forever, currently ignored)
/// * `sigmask_ptr` - Signal mask (currently ignored)
///
/// # Returns
/// Number of fds with events, 0 on timeout, or negative error
pub fn sys_ppoll(fds_ptr: usize, nfds: usize, _tmo_ptr: usize, _sigmask_ptr: usize) -> i64 {
    use crate::task::fd_table::FdType;

    let task = current_task();
    let ttbr0 = task.ttbr0;

    // Validate nfds (reasonable limit)
    if nfds > 1024 {
        return errno::EINVAL;
    }

    if nfds == 0 {
        return 0;
    }

    let pollfd_size = core::mem::size_of::<Pollfd>();
    let buf_size = nfds * pollfd_size;

    // Validate buffer
    if mm_user::validate_user_buffer(ttbr0, fds_ptr, buf_size, true).is_err() {
        return errno::EFAULT;
    }

    let fd_table = task.fd_table.lock();
    let mut ready_count: i64 = 0;

    for i in 0..nfds {
        let pfd_addr = fds_ptr + i * pollfd_size;

        // Read pollfd from user space
        let pfd = match read_pollfd(ttbr0, pfd_addr) {
            Some(p) => p,
            None => return errno::EFAULT,
        };

        // Determine revents based on fd type
        let revents = if pfd.fd < 0 {
            // Negative fd: ignore, set revents = 0
            0i16
        } else {
            match fd_table.get(pfd.fd as usize) {
                None => {
                    // Invalid fd
                    POLLNVAL
                }
                Some(entry) => {
                    poll_fd_type(&entry.fd_type, pfd.events)
                }
            }
        };

        // Write revents back to user space
        if !write_pollfd_revents(ttbr0, pfd_addr, revents) {
            return errno::EFAULT;
        }

        if revents != 0 {
            ready_count += 1;
        }
    }

    log::trace!(
        "[SYSCALL] ppoll(nfds={}) -> {} ready",
        nfds,
        ready_count
    );

    ready_count
}

/// TEAM_360: Read a pollfd struct from user space
fn read_pollfd(ttbr0: usize, addr: usize) -> Option<Pollfd> {
    let mut bytes = [0u8; 8];
    for i in 0..8 {
        bytes[i] = crate::syscall::read_from_user(ttbr0, addr + i)?;
    }

    Some(Pollfd {
        fd: i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        events: i16::from_ne_bytes([bytes[4], bytes[5]]),
        revents: i16::from_ne_bytes([bytes[6], bytes[7]]),
    })
}

/// TEAM_360: Write revents field back to user pollfd
fn write_pollfd_revents(ttbr0: usize, addr: usize, revents: i16) -> bool {
    let revents_offset = 6; // offset of revents in pollfd struct
    let bytes = revents.to_ne_bytes();

    for (i, &byte) in bytes.iter().enumerate() {
        if !crate::syscall::write_to_user_buf(ttbr0, addr + revents_offset, i, byte) {
            return false;
        }
    }
    true
}

/// TEAM_360: Determine poll events for a given fd type
fn poll_fd_type(fd_type: &crate::task::fd_table::FdType, events: i16) -> i16 {
    use crate::task::fd_table::FdType;

    let mut revents: i16 = 0;

    match fd_type {
        FdType::Stdin => {
            // Stdin: check if input available
            // For now, always report readable (conservative)
            if events & POLLIN != 0 {
                revents |= POLLIN;
            }
        }
        FdType::Stdout | FdType::Stderr => {
            // Stdout/Stderr: always writable
            if events & POLLOUT != 0 {
                revents |= POLLOUT;
            }
        }
        FdType::VfsFile(_) => {
            // Regular files: always ready for read/write
            if events & POLLIN != 0 {
                revents |= POLLIN;
            }
            if events & POLLOUT != 0 {
                revents |= POLLOUT;
            }
        }
        FdType::PipeRead(pipe) => {
            // Pipe read end: readable if data available
            if pipe.has_data() {
                if events & POLLIN != 0 {
                    revents |= POLLIN;
                }
            }
            // Check for hangup (write end closed)
            // For now, don't report POLLHUP
        }
        FdType::PipeWrite(pipe) => {
            // Pipe write end: writable if not full
            if pipe.has_space() {
                if events & POLLOUT != 0 {
                    revents |= POLLOUT;
                }
            }
        }
        FdType::PtyMaster(_) | FdType::PtySlave(_) => {
            // PTY: treat like terminal - always ready
            if events & POLLIN != 0 {
                revents |= POLLIN;
            }
            if events & POLLOUT != 0 {
                revents |= POLLOUT;
            }
        }
    }

    revents
}
