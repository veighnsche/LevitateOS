//! Process group and session syscalls.
//!
//! TEAM_394: Process group and session management.
//! TEAM_417: Extracted from process.rs.

use crate::syscall::errno;
use core::sync::atomic::Ordering;

/// TEAM_394: sys_setpgid - Set process group ID.
///
/// # Arguments
/// * `pid` - Process to modify (0 = calling process)
/// * `pgid` - New process group (0 = use pid as pgid)
///
/// # Returns
/// 0 on success, negative errno on failure
pub fn sys_setpgid(pid: i32, pgid: i32) -> i64 {
    let task = crate::task::current_task();
    let current_pid = task.id.0;

    // pid 0 means current process
    let target_pid = if pid == 0 {
        current_pid
    } else {
        pid as usize
    };

    // pgid 0 means use target pid as pgid
    let new_pgid = if pgid == 0 {
        target_pid
    } else {
        pgid as usize
    };

    // For simplicity, only allow setting own process group
    if target_pid != current_pid {
        // Would need to look up target process in process table
        // For now, only support setting own pgid
        return errno::ESRCH;
    }

    task.pgid.store(new_pgid, Ordering::Release);
    log::trace!("[SYSCALL] setpgid({}, {}) -> 0", pid, pgid);
    0
}

/// TEAM_394: sys_getpgid - Get process group ID.
///
/// # Arguments
/// * `pid` - Process to query (0 = calling process)
///
/// # Returns
/// Process group ID on success, negative errno on failure
pub fn sys_getpgid(pid: i32) -> i64 {
    let task = crate::task::current_task();
    let current_pid = task.id.0;

    let target_pid = if pid == 0 {
        current_pid
    } else {
        pid as usize
    };

    // For simplicity, only support querying own pgid
    if target_pid != current_pid {
        return errno::ESRCH;
    }

    let pgid = task.pgid.load(Ordering::Acquire);
    // If pgid is 0, return current pid (process is its own group leader)
    let result = if pgid == 0 { current_pid } else { pgid };
    log::trace!("[SYSCALL] getpgid({}) -> {}", pid, result);
    result as i64
}

/// TEAM_394: sys_getpgrp - Get process group of calling process.
///
/// Equivalent to getpgid(0).
pub fn sys_getpgrp() -> i64 {
    sys_getpgid(0)
}

/// TEAM_394: sys_setsid - Create session and set process group ID.
///
/// Creates a new session if the calling process is not a process group leader.
/// The calling process becomes the session leader and process group leader.
///
/// # Returns
/// New session ID (= pid) on success, negative errno on failure
pub fn sys_setsid() -> i64 {
    let task = crate::task::current_task();
    let pid = task.id.0;
    let current_pgid = task.pgid.load(Ordering::Acquire);

    // Cannot create new session if already a process group leader
    // (pgid == pid means we're the leader of our group)
    if current_pgid == pid {
        return -1; // EPERM
    }

    // Create new session: pid becomes both pgid and sid
    task.pgid.store(pid, Ordering::Release);
    task.sid.store(pid, Ordering::Release);

    log::trace!("[SYSCALL] setsid() -> {}", pid);
    pid as i64
}
