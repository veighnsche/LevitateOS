//! TEAM_216: Signal-related syscalls for LevitateOS.
//! TEAM_420: Uses linux_raw_sys::errno directly, no shims
//! TEAM_421: Returns SyscallResult, no scattered casts

use crate::arch::SyscallFrame;
use crate::syscall::SyscallResult;
use linux_raw_sys::errno::{EFAULT, EINVAL, ENOENT, ESRCH};
use crate::task::{TaskState, current_task, scheduler};
use core::sync::atomic::Ordering;

/// TEAM_220: Signal constants
pub const SIGINT: i32 = 2;
pub const SIGKILL: i32 = 9;
pub const SIGCHLD: i32 = 17;
pub const SIGCONT: i32 = 18;

/// TEAM_216: Send a signal to a process.
/// TEAM_421: Returns SyscallResult
pub fn sys_kill(pid: i32, sig: i32) -> SyscallResult {
    if sig < 0 || sig >= 32 {
        return Err(EINVAL);
    }

    let task_id = pid as usize;
    let table = crate::task::process_table::PROCESS_TABLE.lock();
    if let Some(entry) = table.get(&task_id) {
        if let Some(task) = &entry.task {
            task.pending_signals.fetch_or(1 << sig, Ordering::Release);

            // Wake up if blocked (e.g. in sys_pause)
            if task.get_state() == TaskState::Blocked {
                task.set_state(TaskState::Ready);
                scheduler::SCHEDULER.add_task(task.clone());
            }
            return Ok(0);
        }
    }
    Err(ENOENT)
}

/// TEAM_220: Send a signal to the current foreground process.
pub fn signal_foreground_process(sig: i32) {
    let fg_pid = *crate::task::FOREGROUND_PID.lock();
    log::debug!("signal_foreground_process: sig={} fg_pid={}", sig, fg_pid);
    if fg_pid != 0 {
        let res = sys_kill(fg_pid as i32, sig);
        log::debug!("sys_kill result: {:?}", res);
    } else {
        log::debug!("No foreground process to signal");
    }
}

/// TEAM_216: Wait for any signal to arrive.
/// TEAM_421: Returns SyscallResult (always returns -EINTR when interrupted)
pub fn sys_pause() -> SyscallResult {
    let task = current_task();
    log::trace!("[SIGNAL] pause() for PID={}", task.id.0);

    // Mark task as blocked and yield.
    // It will be woken up when a signal is delivered via sys_kill.
    task.set_state(TaskState::Blocked);
    scheduler::SCHEDULER.schedule();

    // pause() returns only when interrupted by a signal, and always returns -1/EINTR
    Err(4) // EINTR (Linux standard for pause)
}

/// TEAM_216: Register a signal handler.
/// TEAM_421: Returns SyscallResult
pub fn sys_sigaction(sig: i32, handler_addr: usize, restorer_addr: usize) -> SyscallResult {
    if sig < 0 || sig >= 32 {
        return Err(EINVAL);
    }

    let task = current_task();
    let mut handlers = task.signal_handlers.lock();
    handlers[sig as usize] = handler_addr;

    // Record the signal trampoline (restorer) if provided
    if restorer_addr != 0 {
        task.signal_trampoline
            .store(restorer_addr, Ordering::Release);
    }

    Ok(0)
}

/// TEAM_216: Restore context after signal handler execution.
/// TEAM_421: Returns SyscallResult
pub fn sys_sigreturn(frame: &mut SyscallFrame) -> SyscallResult {
    let task = current_task();
    let ttbr0 = task.ttbr0;
    let user_sp = frame.sp;

    let sig_frame_size = core::mem::size_of::<SyscallFrame>();
    let mut original_frame = SyscallFrame::default();
    let frame_ptr = (&mut original_frame as *mut SyscallFrame) as *mut u8;

    // Copy the original frame back from userspace stack
    for i in 0..sig_frame_size {
        if let Some(byte) = crate::syscall::read_from_user(ttbr0, user_sp as usize + i) {
            unsafe {
                *frame_ptr.add(i) = byte;
            }
        } else {
            log::error!(
                "[SIGNAL] PID={} ERROR: Failed to read sigreturn frame from user stack",
                task.id.0
            );
            crate::task::task_exit();
        }
    }

    // Restore the original frame state
    *frame = original_frame;

    // The return value will be placed in frame.regs[0] by syscall_dispatch.
    // We want x0 to be the original x0.
    Ok(frame.regs[0] as i64)
}

/// TEAM_360: sys_sigaltstack - Set/get signal stack.
/// TEAM_421: Returns SyscallResult
///
/// This is a stub that returns success without actually managing
/// an alternate signal stack. Most programs can function without it.
///
/// # Arguments
/// * `ss` - User pointer to new signal stack (or NULL)
/// * `old_ss` - User pointer to store old signal stack (or NULL)
///
/// # Returns
/// Ok(0) on success
#[allow(unused_variables)]
pub fn sys_sigaltstack(ss: usize, old_ss: usize) -> SyscallResult {
    log::trace!("[SYSCALL] sigaltstack(ss=0x{:x}, old_ss=0x{:x}) -> 0", ss, old_ss);
    // TEAM_360: Stub - alternate signal stack not implemented
    // Return success so programs that call this can continue
    Ok(0)
}

/// TEAM_360: Send a signal to a specific thread.
/// TEAM_421: Returns SyscallResult
///
/// Unlike kill() which targets a process, tkill() targets a specific thread
/// identified by its thread ID (TID).
///
/// # Arguments
/// * `tid` - Thread ID to signal
/// * `sig` - Signal number (0 = just check if thread exists)
///
/// # Returns
/// Ok(0) on success, Err(EINVAL) for invalid args, Err(ESRCH) if thread not found
pub fn sys_tkill(tid: i32, sig: i32) -> SyscallResult {
    // Validate signal number
    if sig < 0 || sig >= 64 {
        return Err(EINVAL);
    }

    // tid must be positive
    if tid <= 0 {
        return Err(EINVAL);
    }

    let target_tid = tid as usize;

    // Look up the thread by TID
    let table = crate::task::process_table::PROCESS_TABLE.lock();

    // Find the task with matching TID
    for (_pid, entry) in table.iter() {
        if let Some(task) = &entry.task {
            if task.id.0 == target_tid {
                // Found the target thread
                if sig == 0 {
                    // sig=0 means just check existence
                    return Ok(0);
                }

                // Deliver the signal
                task.pending_signals.fetch_or(1 << sig, Ordering::Release);

                // Wake up if blocked
                if task.get_state() == TaskState::Blocked {
                    task.set_state(TaskState::Ready);
                    scheduler::SCHEDULER.add_task(task.clone());
                }

                log::trace!("[SYSCALL] tkill(tid={}, sig={}) -> 0", tid, sig);
                return Ok(0);
            }
        }
    }

    // Thread not found
    log::trace!("[SYSCALL] tkill(tid={}, sig={}) -> ESRCH", tid, sig);
    Err(ESRCH)
}

/// TEAM_216: Examine and change blocked signals.
/// TEAM_421: Returns SyscallResult
pub fn sys_sigprocmask(how: i32, set_addr: usize, oldset_addr: usize) -> SyscallResult {
    let task = current_task();
    let ttbr0 = task.ttbr0;

    // 1. If oldset_addr is provided, return the current mask
    if oldset_addr != 0 {
        let current_mask = task.blocked_signals.load(Ordering::Acquire);
        for i in 0..4 {
            let byte = (current_mask >> (i * 8)) as u8;
            if !crate::syscall::write_to_user_buf(ttbr0, oldset_addr, i, byte) {
                return Err(EFAULT);
            }
        }
    }

    // 2. If set_addr is provided, update the mask
    if set_addr != 0 {
        // Read 32-bit mask from userspace
        let mut mask: u32 = 0;
        for i in 0..4 {
            if let Some(byte) = crate::syscall::read_from_user(ttbr0, set_addr + i) {
                mask |= (byte as u32) << (i * 8);
            } else {
                return Err(EFAULT);
            }
        }

        match how {
            0 => {
                // SIG_BLOCK
                task.blocked_signals.fetch_or(mask, Ordering::Release);
            }
            1 => {
                // SIG_UNBLOCK
                task.blocked_signals.fetch_and(!mask, Ordering::Release);
            }
            2 => {
                // SIG_SETMASK
                task.blocked_signals.store(mask, Ordering::Release);
            }
            _ => return Err(EINVAL),
        }
    }

    Ok(0)
}
