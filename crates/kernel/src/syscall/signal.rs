//! TEAM_216: Signal-related syscalls for LevitateOS.

use crate::arch::SyscallFrame;
use crate::syscall::errno;
use crate::task::{TaskState, current_task, scheduler};
use core::sync::atomic::Ordering;

/// TEAM_220: Signal constants
pub const SIGINT: i32 = 2;
pub const SIGKILL: i32 = 9;
pub const SIGCHLD: i32 = 17;
pub const SIGCONT: i32 = 18;

/// TEAM_216: Send a signal to a process.
pub fn sys_kill(pid: i32, sig: i32) -> i64 {
    if sig < 0 || sig >= 32 {
        return errno::EINVAL;
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
            return 0;
        }
    }
    errno::ENOENT
}

/// TEAM_220: Send a signal to the current foreground process.
pub fn signal_foreground_process(sig: i32) {
    let fg_pid = *crate::task::FOREGROUND_PID.lock();
    log::debug!("signal_foreground_process: sig={} fg_pid={}", sig, fg_pid);
    if fg_pid != 0 {
        let res = sys_kill(fg_pid as i32, sig);
        log::debug!("sys_kill result: {}", res);
    } else {
        log::debug!("No foreground process to signal");
    }
}

/// TEAM_216: Wait for any signal to arrive.
pub fn sys_pause() -> i64 {
    let task = current_task();
    log::trace!("[SIGNAL] pause() for PID={}", task.id.0);

    // Mark task as blocked and yield.
    // It will be woken up when a signal is delivered via sys_kill.
    task.set_state(TaskState::Blocked);
    scheduler::SCHEDULER.schedule();

    // pause() returns only when interrupted by a signal, and always returns -1/EINTR
    -4 // EINTR (Linux standard for pause)
}

/// TEAM_216: Register a signal handler.
pub fn sys_sigaction(sig: i32, handler_addr: usize, restorer_addr: usize) -> i64 {
    if sig < 0 || sig >= 32 {
        return errno::EINVAL;
    }

    let task = current_task();
    let mut handlers = task.signal_handlers.lock();
    handlers[sig as usize] = handler_addr;

    // Record the signal trampoline (restorer) if provided
    if restorer_addr != 0 {
        task.signal_trampoline
            .store(restorer_addr, Ordering::Release);
    }

    0
}

/// TEAM_216: Restore context after signal handler execution.
pub fn sys_sigreturn(frame: &mut SyscallFrame) -> i64 {
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
    frame.regs[0] as i64
}

/// TEAM_216: Examine and change blocked signals.
pub fn sys_sigprocmask(how: i32, set_addr: usize, oldset_addr: usize) -> i64 {
    let task = current_task();
    let ttbr0 = task.ttbr0;

    // 1. If oldset_addr is provided, return the current mask
    if oldset_addr != 0 {
        let current_mask = task.blocked_signals.load(Ordering::Acquire);
        for i in 0..4 {
            let byte = (current_mask >> (i * 8)) as u8;
            if !crate::syscall::write_to_user_buf(ttbr0, oldset_addr, i, byte) {
                return errno::EFAULT;
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
                return errno::EFAULT;
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
            _ => return errno::EINVAL,
        }
    }

    0
}
