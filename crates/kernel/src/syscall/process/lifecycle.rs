//! Process lifecycle syscalls.
//!
//! TEAM_073: Core process syscalls (exit, getpid, spawn).
//! TEAM_120: Process spawning from initramfs.
//! TEAM_186: Spawn with arguments.
//! TEAM_188: Waitpid implementation.
//! TEAM_414: Helper function extraction.
//! TEAM_417: Extracted from process.rs.

use crate::memory::user as mm_user;
use crate::syscall::errno;
use crate::task::fd_table::FdTable;
use los_hal::IrqSafeLock;
use los_utils::cpio::CpioEntryType;

// ============================================================================
// TEAM_414: Process Syscall Helpers
// ============================================================================

/// TEAM_414: Maximum symlink depth when resolving executables.
const MAX_SYMLINK_DEPTH: usize = 8;

/// TEAM_414: Resolve an executable path from initramfs, following symlinks.
///
/// Returns a copy of the ELF data to avoid holding the INITRAMFS lock.
fn resolve_initramfs_executable(path: &str) -> Result<alloc::vec::Vec<u8>, i64> {
    let archive_lock = crate::fs::INITRAMFS.lock();
    let archive = match archive_lock.as_ref() {
        Some(a) => a,
        None => return Err(errno::ENOSYS),
    };

    // Strip leading '/' for initramfs lookup
    let mut lookup_name = alloc::string::String::from(path.strip_prefix('/').unwrap_or(path));

    for depth in 0..MAX_SYMLINK_DEPTH {
        let mut found_entry = None;
        for entry in archive.archive.iter() {
            if entry.name == lookup_name {
                found_entry = Some(entry);
                break;
            }
        }

        match found_entry {
            Some(entry) => match entry.entry_type {
                CpioEntryType::File => {
                    // Found the actual executable - return a copy
                    return Ok(alloc::vec::Vec::from(entry.data));
                }
                CpioEntryType::Symlink => {
                    // Follow the symlink
                    match core::str::from_utf8(entry.data) {
                        Ok(target) => {
                            log::trace!(
                                "[PROCESS] resolve: {} -> {} (depth {})",
                                lookup_name,
                                target,
                                depth
                            );
                            lookup_name = alloc::string::String::from(target);
                        }
                        Err(_) => return Err(errno::ENOENT),
                    }
                }
                _ => return Err(errno::ENOENT),
            },
            None => return Err(errno::ENOENT),
        }
    }

    // Exceeded max symlink depth
    Err(errno::ELOOP)
}

/// TEAM_414: Clone the current task's FD table for a child process.
fn clone_fd_table_for_child() -> IrqSafeLock<FdTable> {
    let task = crate::task::current_task();
    let flags = los_hal::interrupts::disable();
    let parent_fds = task.fd_table.lock().clone();
    los_hal::interrupts::restore(flags);
    IrqSafeLock::new(parent_fds)
}

/// TEAM_414: Register a newly spawned process and add it to the scheduler.
fn register_spawned_process(new_task: crate::task::user::UserTask) -> i64 {
    let pid = new_task.pid.0 as i64;
    let parent_pid = crate::task::current_task().id.0;
    let child_pid = new_task.pid.0 as usize;
    let tcb: crate::task::TaskControlBlock = new_task.into();
    let task_arc = alloc::sync::Arc::new(tcb);
    crate::task::process_table::register_process(child_pid, parent_pid, task_arc.clone());
    crate::task::scheduler::SCHEDULER.add_task(task_arc);
    pid
}

/// TEAM_414: Write an exit status to a user-space pointer.
fn write_exit_status(ttbr0: usize, status_ptr: usize, exit_code: i32) -> Result<(), i64> {
    if status_ptr == 0 {
        return Ok(());
    }

    if mm_user::validate_user_buffer(ttbr0, status_ptr, 4, true).is_err() {
        return Err(errno::EFAULT);
    }

    match mm_user::user_va_to_kernel_ptr(ttbr0, status_ptr) {
        Some(ptr) => {
            // SAFETY: validate_user_buffer confirmed address is writable
            unsafe {
                *(ptr as *mut i32) = exit_code;
            }
            Ok(())
        }
        None => Err(errno::EFAULT),
    }
}

// ============================================================================
// Process Lifecycle Syscalls
// ============================================================================

/// TEAM_073: sys_exit - Terminate the process.
pub fn sys_exit(code: i32) -> i64 {
    log::trace!("[SYSCALL] exit({})", code);

    // TEAM_188: Wake waiters before exiting
    let task = crate::task::current_task();
    let pid = task.id.0;
    let waiters = crate::task::process_table::mark_exited(pid, code);
    for waiter in waiters {
        waiter.set_state(crate::task::TaskState::Ready);
        crate::task::scheduler::SCHEDULER.add_task(waiter);
    }

    // TEAM_333: Close all FDs immediately to unblock parents reading pipes
    task.fd_table.lock().close_all();

    crate::task::task_exit();
}

/// TEAM_073: sys_getpid - Get process ID.
pub fn sys_getpid() -> i64 {
    crate::task::current_task().id.0 as i64
}

/// TEAM_217: sys_getppid - Get parent process ID.
pub fn sys_getppid() -> i64 {
    let current = crate::task::current_task();
    crate::task::process_table::PROCESS_TABLE
        .lock()
        .get(&current.id.0)
        .map(|e| e.parent_pid as i64)
        .unwrap_or(0)
}

/// TEAM_129: sys_yield - Voluntarily yield CPU to other tasks.
pub fn sys_yield() -> i64 {
    crate::task::yield_now();
    0
}

/// TEAM_350: sys_exit_group - Terminate all threads in the process.
///
/// Unlike sys_exit which only terminates the calling thread, exit_group
/// terminates all threads in the thread group (process).
///
/// For now, LevitateOS doesn't have multi-threaded processes with shared
/// resources, so this behaves the same as sys_exit.
pub fn sys_exit_group(status: i32) -> i64 {
    log::trace!("[SYSCALL] exit_group({})", status);
    // TEAM_350: For now, same as exit since we don't track thread groups
    sys_exit(status)
}

/// TEAM_120: sys_spawn - Spawn a new process from initramfs.
/// TEAM_414: Refactored to use helper functions.
pub fn sys_spawn(path_ptr: usize, path_len: usize) -> i64 {
    let path_len = path_len.min(256);
    let task = crate::task::current_task();

    // Read path from user space
    let mut path_buf = [0u8; 256];
    let path =
        match crate::syscall::copy_user_string(task.ttbr0, path_ptr, path_len, &mut path_buf) {
            Ok(s) => s,
            Err(e) => return e,
        };

    log::trace!("[SYSCALL] spawn('{}')", path);

    // TEAM_414: Use helper to resolve executable with symlink following
    let elf_data = match resolve_initramfs_executable(path) {
        Ok(data) => data,
        Err(e) => return e,
    };

    // TEAM_414: Use helper to clone FD table
    let new_fd_table = clone_fd_table_for_child();

    match crate::task::process::spawn_from_elf(&elf_data, new_fd_table) {
        Ok(new_task) => register_spawned_process(new_task),
        Err(e) => {
            log::debug!("[SYSCALL] spawn failed: {:?}", e);
            -1
        }
    }
}

/// TEAM_120: sys_exec - Replace current process with one from initramfs.
pub fn sys_exec(path_ptr: usize, path_len: usize) -> i64 {
    let path_len = path_len.min(256);
    let task = crate::task::current_task();

    // TEAM_226: Use safe copy through kernel pointers
    let mut path_buf = [0u8; 256];
    let path =
        match crate::syscall::copy_user_string(task.ttbr0, path_ptr, path_len, &mut path_buf) {
            Ok(s) => s,
            Err(e) => return e,
        };

    log::trace!("[SYSCALL] exec('{}')", path);

    let archive_lock = crate::fs::INITRAMFS.lock();
    let archive = match archive_lock.as_ref() {
        Some(sb) => sb,
        None => return errno::ENOSYS,
    };

    let mut elf_data = None;
    for entry in archive.archive.iter() {
        if entry.name == path {
            elf_data = Some(entry.data);
            break;
        }
    }

    let _elf_data = match elf_data {
        Some(d) => d,
        None => return errno::EBADF,
    };

    log::warn!("[SYSCALL] exec is currently a stub");
    errno::ENOSYS
}

// ============================================================================
// TEAM_186: Spawn with Arguments
// ============================================================================

/// TEAM_186: Argv entry from userspace.
#[repr(C)]
#[derive(Copy, Clone)]
struct UserArgvEntry {
    ptr: usize,
    len: usize,
}

/// TEAM_186: Maximum number of arguments
const MAX_ARGC: usize = 16;
/// TEAM_186: Maximum length of a single argument
const MAX_ARG_LEN: usize = 256;

/// TEAM_186: sys_spawn_args - Spawn a new process with arguments.
/// TEAM_414: Refactored to use helper functions.
pub fn sys_spawn_args(path_ptr: usize, path_len: usize, argv_ptr: usize, argc: usize) -> i64 {
    // 1. Validate argc
    if argc > MAX_ARGC {
        return errno::EINVAL;
    }

    // 2. Validate and read path
    let path_len = path_len.min(256);
    let task = crate::task::current_task();
    let mut path_buf = [0u8; 256];
    let path =
        match crate::syscall::copy_user_string(task.ttbr0, path_ptr, path_len, &mut path_buf) {
            Ok(s) => s,
            Err(e) => {
                log::debug!("[SYSCALL] spawn_args: Invalid path: errno={}", e);
                return e;
            }
        };

    // 3. Validate and read argv entries
    let entry_size = core::mem::size_of::<UserArgvEntry>();
    let argv_size = match argc.checked_mul(entry_size) {
        Some(size) => size,
        None => return errno::EINVAL,
    };
    if argc > 0 && mm_user::validate_user_buffer(task.ttbr0, argv_ptr, argv_size, false).is_err() {
        return errno::EFAULT;
    }

    // 4. Parse each argument
    let mut args: alloc::vec::Vec<alloc::string::String> = alloc::vec::Vec::new();
    for i in 0..argc {
        let offset = match i.checked_mul(entry_size) {
            Some(o) => o,
            None => return errno::EINVAL,
        };
        let entry_ptr = match argv_ptr.checked_add(offset) {
            Some(p) => p,
            None => return errno::EINVAL,
        };
        let entry = unsafe {
            let kernel_ptr = mm_user::user_va_to_kernel_ptr(task.ttbr0, entry_ptr);
            match kernel_ptr {
                Some(p) => *(p as *const UserArgvEntry),
                None => return errno::EFAULT,
            }
        };

        let arg_len = entry.len.min(MAX_ARG_LEN);
        let mut arg_buf = [0u8; MAX_ARG_LEN];
        match crate::syscall::copy_user_string(task.ttbr0, entry.ptr, arg_len, &mut arg_buf) {
            Ok(s) => args.push(alloc::string::String::from(s)),
            Err(e) => return e,
        }
    }

    log::trace!("[SYSCALL] spawn_args('{}', argc={})", path, argc);
    for (i, arg) in args.iter().enumerate() {
        log::trace!("[SYSCALL]   argv[{}] = '{}'", i, arg);
    }

    // 5. TEAM_414: Use helper to resolve executable with symlink following
    let elf_data = match resolve_initramfs_executable(path) {
        Ok(data) => data,
        Err(e) => {
            log::debug!("[SYSCALL] spawn_args: resolve failed: errno={}", e);
            return e;
        }
    };

    // 6. Convert args to &str slice
    let arg_refs: alloc::vec::Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    // 7. TEAM_414: Use helper to clone FD table
    let new_fd_table = clone_fd_table_for_child();

    // 8. Spawn with args
    match crate::task::process::spawn_from_elf_with_args(&elf_data, &arg_refs, &[], new_fd_table) {
        Ok(new_task) => register_spawned_process(new_task),
        Err(e) => {
            log::debug!("[SYSCALL] spawn_args: spawn failed for '{}': {:?}", path, e);
            -1
        }
    }
}

// ============================================================================
// TEAM_188: Waitpid
// ============================================================================

/// TEAM_188: Error code for waitpid
const ECHILD: i64 = -10;

/// TEAM_188: sys_waitpid - Wait for a child process to exit.
/// TEAM_414: Refactored to use write_exit_status helper.
pub fn sys_waitpid(pid: i32, status_ptr: usize) -> i64 {
    if pid <= 0 {
        // For now, only support specific PID
        return errno::EINVAL;
    }

    let pid = pid as usize;
    let current = crate::task::current_task();

    // Check if child already exited
    if let Some(exit_code) = crate::task::process_table::try_wait(pid) {
        // TEAM_414: Use helper to write exit status (ignores errors for compat)
        let _ = write_exit_status(current.ttbr0, status_ptr, exit_code);
        crate::task::process_table::reap_zombie(pid);
        return pid as i64;
    }

    // Child still running - block
    if crate::task::process_table::add_waiter(pid, current.clone()).is_err() {
        return ECHILD;
    }

    // Block and schedule
    current.set_state(crate::task::TaskState::Blocked);
    crate::task::scheduler::SCHEDULER.schedule();

    // Woken up - child exited
    if let Some(exit_code) = crate::task::process_table::try_wait(pid) {
        let _ = write_exit_status(current.ttbr0, status_ptr, exit_code);
        crate::task::process_table::reap_zombie(pid);
        return pid as i64;
    }

    ECHILD
}

/// TEAM_220: sys_set_foreground - Set the foreground process for shell control.
pub fn sys_set_foreground(pid: usize) -> i64 {
    *crate::task::FOREGROUND_PID.lock() = pid;
    0
}

/// TEAM_244: sys_get_foreground - Get the foreground process PID.
pub fn sys_get_foreground() -> i64 {
    *crate::task::FOREGROUND_PID.lock() as i64
}
