//! Process management
//! TEAM_275: Refactored to use arch::syscallN

use crate::arch;
use crate::sysno::{
    __NR_clone, __NR_execve, __NR_exit, __NR_getpid, __NR_getppid, __NR_reboot,
    __NR_set_tid_address, __NR_wait4, SYS_GET_FOREGROUND, SYS_SET_FOREGROUND, SYS_SPAWN,
    SYS_SPAWN_ARGS,
};

// ... constants unchanged (CLONE_*, shutdown_flags) ...

/// TEAM_186: ArgvEntry for spawn_args syscall.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ArgvEntry {
    /// Pointer to argument string
    pub ptr: *const u8,
    /// Length of argument string
    pub len: usize,
}

/// TEAM_142: Shutdown commands.
pub mod shutdown_flags {
    pub use linux_raw_sys::general::{
        LINUX_REBOOT_CMD_HALT, LINUX_REBOOT_CMD_POWER_OFF, LINUX_REBOOT_CMD_RESTART,
    };
    // TEAM_310: Legacy flags for compatibility
    pub const NORMAL: u32 = LINUX_REBOOT_CMD_POWER_OFF;
    pub const VERBOSE: u32 = 0;
}

/// Exit the process.
///
/// # Arguments
/// * `code` - Exit code (0 = success)
#[inline]
pub fn exit(code: i32) -> ! {
    arch::syscall_exit(__NR_exit as u64, code as u64)
}

/// Get current process ID.
#[inline]
pub fn getpid() -> i64 {
    arch::syscall0(__NR_getpid as u64)
}

/// TEAM_217: Get parent process ID.
#[inline]
pub fn getppid() -> i64 {
    arch::syscall0(__NR_getppid as u64)
}

/// TEAM_228: Create a new thread (clone syscall).
#[inline]
pub fn clone(
    flags: u64,
    stack: usize,
    parent_tid: *mut i32,
    tls: usize,
    child_tid: *mut i32,
) -> isize {
    arch::syscall5(
        __NR_clone as u64,
        flags,
        stack as u64,
        parent_tid as u64,
        tls as u64,
        child_tid as u64,
    ) as isize
}

/// TEAM_228: Set pointer to thread ID (cleared on exit).
#[inline]
pub fn set_tid_address(tidptr: *mut i32) -> isize {
    arch::syscall1(__NR_set_tid_address as u64, tidptr as u64) as isize
}

/// Spawn a new process from a path.
#[inline]
pub fn spawn(path: &str) -> isize {
    arch::syscall2(SYS_SPAWN as u64, path.as_ptr() as u64, path.len() as u64) as isize
}

/// TEAM_186: Spawn a process with command-line arguments.
#[inline]
pub fn spawn_args(path: &str, argv: &[&str]) -> isize {
    // Build ArgvEntry array on stack (max 16 args)
    let mut entries = [ArgvEntry {
        ptr: core::ptr::null(),
        len: 0,
    }; 16];
    let argc = argv.len().min(16);
    for (i, arg) in argv.iter().take(argc).enumerate() {
        entries[i] = ArgvEntry {
            ptr: arg.as_ptr(),
            len: arg.len(),
        };
    }

    arch::syscall4(
        SYS_SPAWN_ARGS as u64,
        path.as_ptr() as u64,
        path.len() as u64,
        entries.as_ptr() as u64,
        argc as u64,
    ) as isize
}

/// Replace current process with a new one from a path.
#[inline]
pub fn exec(path: &str) -> isize {
    arch::syscall2(__NR_execve as u64, path.as_ptr() as u64, path.len() as u64) as isize
}

/// TEAM_188: Wait for a child process to exit.
#[inline]
pub fn waitpid(pid: i32, status: Option<&mut i32>) -> isize {
    let status_ptr = match status {
        Some(s) => s as *mut i32 as u64,
        None => 0,
    };
    arch::syscall2(__NR_wait4 as u64, pid as u64, status_ptr) as isize
}

/// TEAM_220: Set the foreground process for shell control.
#[inline]
pub fn set_foreground(pid: usize) -> isize {
    arch::syscall1(SYS_SET_FOREGROUND as u64, pid as u64) as isize
}

/// TEAM_244: Get the foreground process PID.
#[inline]
pub fn get_foreground() -> isize {
    arch::syscall0(SYS_GET_FOREGROUND as u64) as isize
}

/// TEAM_142: Graceful system shutdown.
#[inline]
pub fn shutdown(flags: u32) -> ! {
    arch::syscall_exit(__NR_reboot as u64, flags as u64)
}
