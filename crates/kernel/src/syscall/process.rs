use crate::memory::user as mm_user;

use crate::syscall::errno;
use los_hal::IrqSafeLock;

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

/// TEAM_120: sys_spawn - Spawn a new process from initramfs.
pub fn sys_spawn(path_ptr: usize, path_len: usize) -> i64 {
    let path_len = path_len.min(256);
    let task = crate::task::current_task();

    // TEAM_226: Use safe copy through kernel pointers
    let mut path_buf = [0u8; 256];
    let path = match crate::syscall::copy_user_string(task.ttbr0, path_ptr, path_len, &mut path_buf)
    {
        Ok(s) => s,
        Err(e) => return e,
    };

    log::trace!("[SYSCALL] spawn('{}')", path);

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

    let elf_data = match elf_data {
        Some(d) => d,
        None => return errno::EBADF,
    };

    // TEAM_250: Clone parent's FD table for inheritance
    let flags = los_hal::interrupts::disable();
    let parent_fds = task.fd_table.lock().clone();
    los_hal::interrupts::restore(flags);
    let new_fd_table = IrqSafeLock::new(parent_fds);

    match crate::task::process::spawn_from_elf(elf_data, new_fd_table) {
        Ok(new_task) => {
            let pid = new_task.pid.0 as i64;
            // TEAM_188: Register in process table
            let parent_pid = crate::task::current_task().id.0;
            let child_pid = new_task.pid.0 as usize;
            let tcb: crate::task::TaskControlBlock = new_task.into();
            let task_arc = alloc::sync::Arc::new(tcb);
            crate::task::process_table::register_process(child_pid, parent_pid, task_arc.clone());
            crate::task::scheduler::SCHEDULER.add_task(task_arc);
            pid
        }
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
    let path = match crate::syscall::copy_user_string(task.ttbr0, path_ptr, path_len, &mut path_buf)
    {
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
///
/// # Arguments
/// - path_ptr: Pointer to executable path string
/// - path_len: Length of path string
/// - argv_ptr: Pointer to array of UserArgvEntry structs
/// - argc: Number of arguments
///
/// # Returns
/// PID of spawned process on success, negative errno on failure.
pub fn sys_spawn_args(path_ptr: usize, path_len: usize, argv_ptr: usize, argc: usize) -> i64 {
    // 1. Validate argc
    if argc > MAX_ARGC {
        return errno::EINVAL;
    }

    // 2. Validate and read path
    // TEAM_226: Use safe copy through kernel pointers
    let path_len = path_len.min(256);
    let task = crate::task::current_task();
    let mut path_buf = [0u8; 256];
    let path = match crate::syscall::copy_user_string(task.ttbr0, path_ptr, path_len, &mut path_buf)
    {
        Ok(s) => alloc::string::String::from(s),
        Err(e) => {
            log::debug!("[SYSCALL] spawn_args: Invalid path: errno={}", e);
            return e;
        }
    };

    // 3. Validate and read argv entries
    // TEAM_188: Use checked arithmetic to prevent overflow
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
        // TEAM_188: Use checked arithmetic for entry_ptr calculation
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

        // TEAM_226: Validate and copy arg string safely
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

    // 5. Find executable and copy ELF data
    // TEAM_188: Copy ELF data to release lock before spawning (prevents deadlock)
    let elf_data_copy = {
        let archive_lock = crate::fs::INITRAMFS.lock();
        let archive = match archive_lock.as_ref() {
            Some(a) => a,
            None => {
                log::debug!("[SYSCALL] spawn_args: INITRAMFS not available");
                return errno::ENOSYS;
            }
        };

        let mut elf_data = None;
        // TEAM_212: Strip leading '/' for initramfs lookup
        let lookup_name = path.strip_prefix('/').unwrap_or(&path);
        for entry in archive.archive.iter() {
            if entry.name == lookup_name {
                elf_data = Some(entry.data);
                break;
            }
        }

        match elf_data {
            Some(d) => alloc::vec::Vec::from(d),
            None => {
                log::debug!("[SYSCALL] spawn_args: '{}' not found in initramfs", path);
                return crate::syscall::errno_file::ENOENT;
            }
        }
        // Lock released here
    };

    // 6. Convert args to &str slice
    let arg_refs: alloc::vec::Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    // 7. Spawn with args (lock no longer held)
    // TEAM_250: Clone parent's FD table for inheritance
    let flags = los_hal::interrupts::disable();
    let parent_fds = task.fd_table.lock().clone();
    los_hal::interrupts::restore(flags);
    let new_fd_table = IrqSafeLock::new(parent_fds);

    match crate::task::process::spawn_from_elf_with_args(
        &elf_data_copy,
        &arg_refs,
        &[],
        new_fd_table,
    ) {
        Ok(new_task) => {
            let pid = new_task.pid.0 as i64;
            // TEAM_188: Register in process table
            let parent_pid = crate::task::current_task().id.0;
            let child_pid = new_task.pid.0 as usize;
            let tcb: crate::task::TaskControlBlock = new_task.into();
            let task_arc = alloc::sync::Arc::new(tcb);
            crate::task::process_table::register_process(child_pid, parent_pid, task_arc.clone());
            crate::task::scheduler::SCHEDULER.add_task(task_arc);
            pid
        }
        Err(e) => {
            log::debug!(
                "[SYSCALL] spawn_args: spawn_from_elf_with_args failed for '{}': {:?}",
                path,
                e
            );
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
pub fn sys_waitpid(pid: i32, status_ptr: usize) -> i64 {
    if pid <= 0 {
        // For now, only support specific PID
        return errno::EINVAL;
    }

    let pid = pid as usize;
    let current = crate::task::current_task();

    // Check if child already exited
    if let Some(exit_code) = crate::task::process_table::try_wait(pid) {
        // Write exit code to user if requested
        if status_ptr != 0 {
            // Validate and write
            if mm_user::validate_user_buffer(current.ttbr0, status_ptr, 4, true).is_ok() {
                if let Some(ptr) = mm_user::user_va_to_kernel_ptr(current.ttbr0, status_ptr) {
                    unsafe {
                        *(ptr as *mut i32) = exit_code;
                    }
                }
            }
        }
        // Reap the zombie
        crate::task::process_table::reap_zombie(pid);
        return pid as i64;
    }

    // Child still running - block
    if let Err(_) = crate::task::process_table::add_waiter(pid, current.clone()) {
        return ECHILD; // Process not found or already exited (should have caught above)
    }

    // Block and schedule
    current.set_state(crate::task::TaskState::Blocked);
    crate::task::scheduler::SCHEDULER.schedule();

    // Woken up - child exited
    if let Some(exit_code) = crate::task::process_table::try_wait(pid) {
        if status_ptr != 0 {
            if mm_user::validate_user_buffer(current.ttbr0, status_ptr, 4, true).is_ok() {
                if let Some(ptr) = mm_user::user_va_to_kernel_ptr(current.ttbr0, status_ptr) {
                    unsafe {
                        *(ptr as *mut i32) = exit_code;
                    }
                }
            }
        }
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

// ============================================================================
// TEAM_228: Threading syscalls for std support
// ============================================================================

/// Clone flags (Linux ABI)
pub const CLONE_VM: u64 = 0x00000100;
pub const CLONE_FS: u64 = 0x00000200;
pub const CLONE_FILES: u64 = 0x00000400;
pub const CLONE_SIGHAND: u64 = 0x00000800;
pub const CLONE_THREAD: u64 = 0x00010000;
pub const CLONE_SETTLS: u64 = 0x00080000;
pub const CLONE_PARENT_SETTID: u64 = 0x00100000;
pub const CLONE_CHILD_CLEARTID: u64 = 0x00200000;
pub const CLONE_CHILD_SETTID: u64 = 0x01000000;

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
