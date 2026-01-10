use crate::memory::user as mm_user;

use crate::syscall::errno;
use core::sync::atomic::Ordering;
use los_hal::IrqSafeLock;
use los_utils::cpio::CpioEntryType;

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

    // TEAM_381: Follow symlinks when resolving executable path
    let mut lookup_name = alloc::string::String::from(path);
    let mut elf_data = None;
    const MAX_SYMLINK_DEPTH: usize = 8;

    for depth in 0..MAX_SYMLINK_DEPTH {
        let mut found_entry = None;
        for entry in archive.archive.iter() {
            if entry.name == lookup_name {
                found_entry = Some(entry);
                break;
            }
        }

        match found_entry {
            Some(entry) => {
                match entry.entry_type {
                    CpioEntryType::File => {
                        elf_data = Some(entry.data);
                        break;
                    }
                    CpioEntryType::Symlink => {
                        match core::str::from_utf8(entry.data) {
                            Ok(target) => {
                                log::trace!("[SYSCALL] spawn: {} -> {} (depth {})", lookup_name, target, depth);
                                lookup_name = alloc::string::String::from(target);
                            }
                            Err(_) => return errno::ENOENT,
                        }
                    }
                    _ => return errno::ENOENT,
                }
            }
            None => return errno::EBADF,
        }
    }

    let elf_data = match elf_data {
        Some(d) => d,
        None => return errno::ELOOP,
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

        // TEAM_212: Strip leading '/' for initramfs lookup
        // TEAM_381: Follow symlinks when resolving executable path
        let mut lookup_name = alloc::string::String::from(path.strip_prefix('/').unwrap_or(&path));
        let mut elf_data = None;
        const MAX_SYMLINK_DEPTH: usize = 8;

        for depth in 0..MAX_SYMLINK_DEPTH {
            let mut found_entry = None;
            for entry in archive.archive.iter() {
                if entry.name == lookup_name {
                    found_entry = Some(entry);
                    break;
                }
            }

            match found_entry {
                Some(entry) => {
                    match entry.entry_type {
                        CpioEntryType::File => {
                            // Found the actual executable
                            elf_data = Some(entry.data);
                            break;
                        }
                        CpioEntryType::Symlink => {
                            // Follow the symlink - data contains target path
                            match core::str::from_utf8(entry.data) {
                                Ok(target) => {
                                    log::trace!(
                                        "[SYSCALL] spawn_args: {} -> {} (depth {})",
                                        lookup_name,
                                        target,
                                        depth
                                    );
                                    lookup_name = alloc::string::String::from(target);
                                }
                                Err(_) => {
                                    log::debug!("[SYSCALL] spawn_args: invalid symlink target");
                                    return errno::ENOENT;
                                }
                            }
                        }
                        _ => {
                            log::debug!("[SYSCALL] spawn_args: '{}' is not executable", lookup_name);
                            return errno::ENOENT;
                        }
                    }
                }
                None => {
                    log::debug!("[SYSCALL] spawn_args: '{}' not found in initramfs", lookup_name);
                    return errno::ENOENT;
                }
            }
        }

        match elf_data {
            Some(d) => alloc::vec::Vec::from(d),
            None => {
                log::debug!("[SYSCALL] spawn_args: symlink loop or max depth exceeded");
                return errno::ELOOP;
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

// ============================================================================
// TEAM_350: Eyra Prerequisites - Simple Syscalls
// ============================================================================

/// TEAM_350: sys_gettid - Get thread ID.
///
/// Returns the caller's thread ID (TID). In a single-threaded process,
/// this is the same as the PID.
pub fn sys_gettid() -> i64 {
    crate::task::current_task().id.0 as i64
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

/// TEAM_350: sys_getuid - Get real user ID.
///
/// LevitateOS is single-user, always returns 0 (root).
pub fn sys_getuid() -> i64 {
    0
}

/// TEAM_350: sys_geteuid - Get effective user ID.
///
/// LevitateOS is single-user, always returns 0 (root).
pub fn sys_geteuid() -> i64 {
    0
}

/// TEAM_350: sys_getgid - Get real group ID.
///
/// LevitateOS is single-user, always returns 0 (root group).
pub fn sys_getgid() -> i64 {
    0
}

/// TEAM_350: sys_getegid - Get effective group ID.
///
/// LevitateOS is single-user, always returns 0 (root group).
pub fn sys_getegid() -> i64 {
    0
}

// ============================================================================
// TEAM_350: arch_prctl (x86_64 only) - Set architecture-specific thread state
// ============================================================================

/// TEAM_350: arch_prctl codes for x86_64
#[cfg(target_arch = "x86_64")]
pub mod arch_prctl_codes {
    pub const ARCH_SET_GS: i32 = 0x1001;
    pub const ARCH_SET_FS: i32 = 0x1002;
    pub const ARCH_GET_FS: i32 = 0x1003;
    pub const ARCH_GET_GS: i32 = 0x1004;
}

/// TEAM_350: sys_arch_prctl - Set architecture-specific thread state (x86_64).
///
/// Used primarily for setting the FS base register for TLS (Thread Local Storage).
///
/// # Arguments
/// * `code` - ARCH_SET_FS, ARCH_GET_FS, ARCH_SET_GS, ARCH_GET_GS
/// * `addr` - Address to set/get
///
/// # Returns
/// 0 on success, negative errno on failure.
#[cfg(target_arch = "x86_64")]
pub fn sys_arch_prctl(code: i32, addr: usize) -> i64 {
    use arch_prctl_codes::*;

    log::trace!("[SYSCALL] arch_prctl(code=0x{:x}, addr=0x{:x})", code, addr);

    match code {
        ARCH_SET_FS => {
            // TEAM_350: Set FS base register for TLS
            // SAFETY: Writing to FS_BASE MSR is safe if addr is valid
            unsafe {
                // IA32_FS_BASE MSR = 0xC0000100
                core::arch::asm!(
                    "wrmsr",
                    in("ecx") 0xC000_0100u32,
                    in("eax") (addr as u32),
                    in("edx") ((addr >> 32) as u32),
                    options(nostack, preserves_flags)
                );
            }
            // TEAM_409: Store in BOTH task.tls AND context.fs_base for context switch restore
            // The context switch assembly restores from context.fs_base, not task.tls
            let task = crate::task::current_task();
            task.tls.store(addr, core::sync::atomic::Ordering::Release);
            // SAFETY: We're modifying our own context which won't be used until we context switch out
            unsafe {
                let ctx_ptr = &task.context as *const _ as *mut crate::arch::Context;
                (*ctx_ptr).set_tls(addr as u64);
            }
            0
        }
        ARCH_GET_FS => {
            // TEAM_350: Get FS base register
            let task = crate::task::current_task();
            if addr != 0 {
                if mm_user::validate_user_buffer(task.ttbr0, addr, 8, true).is_ok() {
                    if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, addr) {
                        let fs_base = task.tls.load(core::sync::atomic::Ordering::Acquire);
                        // SAFETY: validate_user_buffer confirmed address is writable
                        unsafe {
                            *(ptr as *mut u64) = fs_base as u64;
                        }
                        return 0;
                    }
                }
                return errno::EFAULT;
            }
            0
        }
        ARCH_SET_GS => {
            // TEAM_350: Set GS base register (less commonly used)
            unsafe {
                // IA32_GS_BASE MSR = 0xC0000101
                core::arch::asm!(
                    "wrmsr",
                    in("ecx") 0xC000_0101u32,
                    in("eax") (addr as u32),
                    in("edx") ((addr >> 32) as u32),
                    options(nostack, preserves_flags)
                );
            }
            0
        }
        ARCH_GET_GS => {
            // TEAM_350: Get GS base - read from MSR
            let task = crate::task::current_task();
            if addr != 0 {
                if mm_user::validate_user_buffer(task.ttbr0, addr, 8, true).is_ok() {
                    if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, addr) {
                        let gs_base: u64;
                        unsafe {
                            let lo: u32;
                            let hi: u32;
                            core::arch::asm!(
                                "rdmsr",
                                in("ecx") 0xC000_0101u32,
                                out("eax") lo,
                                out("edx") hi,
                                options(nostack, preserves_flags)
                            );
                            gs_base = ((hi as u64) << 32) | (lo as u64);
                            *(ptr as *mut u64) = gs_base;
                        }
                        return 0;
                    }
                }
                return errno::EFAULT;
            }
            0
        }
        _ => errno::EINVAL,
    }
}

/// TEAM_350: sys_arch_prctl stub for non-x86_64 architectures.
#[cfg(not(target_arch = "x86_64"))]
pub fn sys_arch_prctl(_code: i32, _addr: usize) -> i64 {
    // arch_prctl is x86_64-specific
    errno::ENOSYS
}

/// TEAM_394: sys_setpgid - Set process group ID.
///
/// # Arguments
/// * `pid` - Process to modify (0 = calling process)
/// * `pgid` - New process group (0 = use pid as pgid)
///
/// # Returns
/// 0 on success, negative errno on failure
pub fn sys_setpgid(pid: i32, pgid: i32) -> i64 {
    use core::sync::atomic::Ordering;

    let task = crate::task::current_task();
    let current_pid = task.id.0;

    // pid 0 means current process
    let target_pid = if pid == 0 { current_pid } else { pid as usize };

    // pgid 0 means use target pid as pgid
    let new_pgid = if pgid == 0 { target_pid } else { pgid as usize };

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
    use core::sync::atomic::Ordering;

    let task = crate::task::current_task();
    let current_pid = task.id.0;

    let target_pid = if pid == 0 { current_pid } else { pid as usize };

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
    use core::sync::atomic::Ordering;

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

// ============================================================================
// TEAM_406: System identification and file creation mask
// ============================================================================

/// TEAM_406: Linux utsname structure for sys_uname.
#[repr(C)]
pub struct Utsname {
    pub sysname: [u8; 65],
    pub nodename: [u8; 65],
    pub release: [u8; 65],
    pub version: [u8; 65],
    pub machine: [u8; 65],
    pub domainname: [u8; 65],
}

impl Default for Utsname {
    fn default() -> Self {
        Self {
            sysname: [0; 65],
            nodename: [0; 65],
            release: [0; 65],
            version: [0; 65],
            machine: [0; 65],
            domainname: [0; 65],
        }
    }
}

/// TEAM_406: Copy a string into a fixed-size array, null-terminated.
fn str_to_array<const N: usize>(s: &str) -> [u8; N] {
    let mut arr = [0u8; N];
    let bytes = s.as_bytes();
    let len = bytes.len().min(N - 1); // Leave room for null terminator
    arr[..len].copy_from_slice(&bytes[..len]);
    arr
}

/// TEAM_406: sys_uname - Get system identification.
///
/// Fills the utsname structure with system information.
///
/// # Arguments
/// * `buf` - User pointer to utsname structure
///
/// # Returns
/// 0 on success, negative errno on failure.
pub fn sys_uname(buf: usize) -> i64 {
    let task = crate::task::current_task();
    
    // Validate user buffer
    let size = core::mem::size_of::<Utsname>();
    if mm_user::validate_user_buffer(task.ttbr0, buf, size, true).is_err() {
        return errno::EFAULT;
    }
    
    #[cfg(target_arch = "x86_64")]
    const MACHINE: &str = "x86_64";
    #[cfg(target_arch = "aarch64")]
    const MACHINE: &str = "aarch64";
    
    let utsname = Utsname {
        sysname: str_to_array("LevitateOS"),
        nodename: str_to_array("levitate"),
        release: str_to_array("0.1.0"),
        version: str_to_array("0.1.0"),
        machine: str_to_array(MACHINE),
        domainname: str_to_array("(none)"),
    };
    
    // Copy to user space byte by byte
    let bytes = unsafe {
        core::slice::from_raw_parts(
            &utsname as *const Utsname as *const u8,
            size,
        )
    };
    
    for (i, &byte) in bytes.iter().enumerate() {
        if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, buf + i) {
            unsafe { *ptr = byte; }
        } else {
            return errno::EFAULT;
        }
    }
    
    log::trace!("[SYSCALL] uname() -> 0");
    0
}

/// TEAM_406: sys_umask - Set file creation mask.
///
/// Sets the file mode creation mask and returns the old mask.
///
/// # Arguments
/// * `mask` - New file mode creation mask
///
/// # Returns
/// Previous umask value.
pub fn sys_umask(mask: u32) -> i64 {
    let task = crate::task::current_task();
    let old = task.umask.swap(mask & 0o777, Ordering::SeqCst);
    log::trace!("[SYSCALL] umask(0o{:o}) -> 0o{:o}", mask, old);
    old as i64
}

// ============================================================================
// TEAM_409: Resource usage syscalls
// ============================================================================

/// TEAM_409: rusage structure for getrusage syscall.
#[repr(C)]
#[derive(Default)]
pub struct Rusage {
    pub ru_utime: Timeval,     // User time used
    pub ru_stime: Timeval,     // System time used
    pub ru_maxrss: i64,        // Maximum resident set size
    pub ru_ixrss: i64,         // Integral shared memory size
    pub ru_idrss: i64,         // Integral unshared data size
    pub ru_isrss: i64,         // Integral unshared stack size
    pub ru_minflt: i64,        // Page reclaims (soft page faults)
    pub ru_majflt: i64,        // Page faults (hard page faults)
    pub ru_nswap: i64,         // Swaps
    pub ru_inblock: i64,       // Block input operations
    pub ru_oublock: i64,       // Block output operations
    pub ru_msgsnd: i64,        // IPC messages sent
    pub ru_msgrcv: i64,        // IPC messages received
    pub ru_nsignals: i64,      // Signals received
    pub ru_nvcsw: i64,         // Voluntary context switches
    pub ru_nivcsw: i64,        // Involuntary context switches
}

/// TEAM_409: timeval structure for rusage.
#[repr(C)]
#[derive(Default)]
pub struct Timeval {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

/// TEAM_409: sys_getrusage - Get resource usage.
///
/// Returns resource usage statistics for the calling process.
/// Currently returns zeros for most fields (simplified implementation).
///
/// # Arguments
/// * `who` - RUSAGE_SELF (0), RUSAGE_CHILDREN (1), or RUSAGE_THREAD (1)
/// * `usage` - User pointer to rusage struct
///
/// # Returns
/// 0 on success, negative errno on failure.
pub fn sys_getrusage(who: i32, usage: usize) -> i64 {
    const RUSAGE_SELF: i32 = 0;
    const RUSAGE_CHILDREN: i32 = -1;
    const RUSAGE_THREAD: i32 = 1;

    // Validate who argument
    if who != RUSAGE_SELF && who != RUSAGE_CHILDREN && who != RUSAGE_THREAD {
        return errno::EINVAL;
    }

    if usage == 0 {
        return errno::EFAULT;
    }

    let task = crate::task::current_task();
    let rusage_size = core::mem::size_of::<Rusage>();

    if mm_user::validate_user_buffer(task.ttbr0, usage, rusage_size, true).is_err() {
        return errno::EFAULT;
    }

    // Create a zeroed rusage struct (simplified - we don't track these metrics yet)
    let rusage = Rusage::default();

    // Copy to user space
    let dest = mm_user::user_va_to_kernel_ptr(task.ttbr0, usage).unwrap();
    unsafe {
        core::ptr::copy_nonoverlapping(
            &rusage as *const Rusage as *const u8,
            dest,
            rusage_size,
        );
    }

    0
}

// ============================================================================
// TEAM_409: Resource limit syscalls
// ============================================================================

/// TEAM_409: sys_prlimit64 - Get/set resource limits.
///
/// This is a stub implementation that returns sensible defaults.
/// Full resource limiting is not yet implemented.
///
/// # Arguments
/// * `pid` - Process ID (0 = current process)
/// * `resource` - Resource type (RLIMIT_*)
/// * `new_limit` - New limit to set (NULL to only get)
/// * `old_limit` - Buffer for old limit (NULL to only set)
///
/// # Returns
/// 0 on success, negative errno on failure.
pub fn sys_prlimit64(pid: i32, resource: u32, new_limit: usize, old_limit: usize) -> i64 {
    use crate::memory::user as mm_user;
    use crate::syscall::errno;

    // Resource limit constants
    const RLIMIT_NOFILE: u32 = 7;   // Max open files
    const RLIMIT_STACK: u32 = 3;    // Max stack size
    const RLIMIT_AS: u32 = 9;       // Address space limit
    const RLIMIT_FSIZE: u32 = 1;    // Max file size
    const RLIMIT_DATA: u32 = 2;     // Max data segment size
    const RLIMIT_CORE: u32 = 4;     // Max core file size
    const RLIMIT_CPU: u32 = 0;      // CPU time limit
    const RLIMIT_RSS: u32 = 5;      // Max resident set size
    const RLIMIT_NPROC: u32 = 6;    // Max processes
    const RLIMIT_MEMLOCK: u32 = 8;  // Max locked memory

    const RLIM_INFINITY: u64 = u64::MAX;

    // rlimit64 struct: { rlim_cur: u64, rlim_max: u64 }
    #[repr(C)]
    struct Rlimit64 {
        rlim_cur: u64,  // Soft limit
        rlim_max: u64,  // Hard limit
    }

    let task = crate::task::current_task();

    // Only support current process for now
    if pid != 0 && pid != task.id.0 as i32 {
        log::warn!("[SYSCALL] prlimit64: pid {} not supported (only current process)", pid);
        return errno::ESRCH;
    }

    // Default limits (sensible values for a simple OS)
    let default_limit = match resource {
        RLIMIT_NOFILE => Rlimit64 { rlim_cur: 1024, rlim_max: 4096 },
        RLIMIT_STACK => Rlimit64 { rlim_cur: 8 * 1024 * 1024, rlim_max: RLIM_INFINITY },
        RLIMIT_AS => Rlimit64 { rlim_cur: RLIM_INFINITY, rlim_max: RLIM_INFINITY },
        RLIMIT_FSIZE => Rlimit64 { rlim_cur: RLIM_INFINITY, rlim_max: RLIM_INFINITY },
        RLIMIT_DATA => Rlimit64 { rlim_cur: RLIM_INFINITY, rlim_max: RLIM_INFINITY },
        RLIMIT_CORE => Rlimit64 { rlim_cur: 0, rlim_max: RLIM_INFINITY },
        RLIMIT_CPU => Rlimit64 { rlim_cur: RLIM_INFINITY, rlim_max: RLIM_INFINITY },
        RLIMIT_RSS => Rlimit64 { rlim_cur: RLIM_INFINITY, rlim_max: RLIM_INFINITY },
        RLIMIT_NPROC => Rlimit64 { rlim_cur: 1024, rlim_max: 4096 },
        RLIMIT_MEMLOCK => Rlimit64 { rlim_cur: 64 * 1024, rlim_max: 64 * 1024 },
        _ => {
            log::warn!("[SYSCALL] prlimit64: unknown resource {}", resource);
            return errno::EINVAL;
        }
    };

    // Return old limit if requested
    if old_limit != 0 {
        let limit_size = core::mem::size_of::<Rlimit64>();
        if mm_user::validate_user_buffer(task.ttbr0, old_limit, limit_size, true).is_err() {
            return errno::EFAULT;
        }
        let dest = mm_user::user_va_to_kernel_ptr(task.ttbr0, old_limit).unwrap();
        unsafe {
            core::ptr::copy_nonoverlapping(
                &default_limit as *const Rlimit64 as *const u8,
                dest,
                limit_size,
            );
        }
    }

    // Setting new limit is a no-op for now (we don't enforce limits)
    if new_limit != 0 {
        log::trace!("[SYSCALL] prlimit64: ignoring new_limit for resource {}", resource);
    }

    0
}
