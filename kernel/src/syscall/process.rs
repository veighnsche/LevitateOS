use crate::memory::user as mm_user;

use crate::syscall::errno;
use los_hal::println;

/// TEAM_073: sys_exit - Terminate the process.
pub fn sys_exit(code: i32) -> i64 {
    println!("[SYSCALL] exit({})", code);

    // TEAM_188: Wake waiters before exiting
    let task = crate::task::current_task();
    let pid = task.id.0;
    let waiters = crate::task::process_table::mark_exited(pid, code);
    for waiter in waiters {
        waiter.set_state(crate::task::TaskState::Ready);
        crate::task::scheduler::SCHEDULER.add_task(waiter);
    }

    crate::task::task_exit();
}

/// TEAM_073: sys_getpid - Get process ID.
pub fn sys_getpid() -> i64 {
    crate::task::current_task().id.0 as i64
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
    if mm_user::validate_user_buffer(task.ttbr0, path_ptr, path_len, false).is_err() {
        return errno::EFAULT;
    }

    let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr as *const u8, path_len) };
    let path = match core::str::from_utf8(path_bytes) {
        Ok(s) => s,
        Err(_) => return errno::EINVAL,
    };

    println!("[SYSCALL] spawn('{}')", path);

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

    match crate::task::process::spawn_from_elf(elf_data) {
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
            println!("[SYSCALL] spawn failed: {:?}", e);
            -1
        }
    }
}

/// TEAM_120: sys_exec - Replace current process with one from initramfs.
pub fn sys_exec(path_ptr: usize, path_len: usize) -> i64 {
    let path_len = path_len.min(256);
    let task = crate::task::current_task();
    if mm_user::validate_user_buffer(task.ttbr0, path_ptr, path_len, false).is_err() {
        return errno::EFAULT;
    }

    let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr as *const u8, path_len) };
    let path = match core::str::from_utf8(path_bytes) {
        Ok(s) => s,
        Err(_) => return errno::EINVAL,
    };

    println!("[SYSCALL] exec('{}')", path);

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

    println!("[SYSCALL] exec is currently a stub");
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
    let path_len = path_len.min(256);
    let task = crate::task::current_task();
    if mm_user::validate_user_buffer(task.ttbr0, path_ptr, path_len, false).is_err() {
        return errno::EFAULT;
    }
    let path_bytes = unsafe { core::slice::from_raw_parts(path_ptr as *const u8, path_len) };
    let path = match core::str::from_utf8(path_bytes) {
        Ok(s) => s,
        Err(_) => return errno::EINVAL,
    };

    // 3. Validate and read argv entries
    // TEAM_188: Use checked arithmetic to prevent overflow
    let entry_size = core::mem::size_of::<UserArgvEntry>();
    let argv_size = match argc.checked_mul(entry_size) {
        Some(size) => size,
        None => return errno::EINVAL,
    };
    if argc > 0
        && mm_user::validate_user_buffer(task.ttbr0, argv_ptr, argv_size, false)
            .is_err()
    {
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

        // Validate arg string
        let arg_len = entry.len.min(MAX_ARG_LEN);
        if mm_user::validate_user_buffer(task.ttbr0, entry.ptr, arg_len, false)
            .is_err()
        {
            return errno::EFAULT;
        }

        let arg_bytes = unsafe { core::slice::from_raw_parts(entry.ptr as *const u8, arg_len) };
        match core::str::from_utf8(arg_bytes) {
            Ok(s) => args.push(alloc::string::String::from(s)),
            Err(_) => return errno::EINVAL,
        }
    }

    println!("[SYSCALL] spawn_args('{}', argc={})", path, argc);
    for (i, arg) in args.iter().enumerate() {
        println!("[SYSCALL]   argv[{}] = '{}'", i, arg);
    }

    // 5. Find executable and copy ELF data
    // TEAM_188: Copy ELF data to release lock before spawning (prevents deadlock)
    let elf_data_copy = {
        let archive_lock = crate::fs::INITRAMFS.lock();
        let archive = match archive_lock.as_ref() {
            Some(a) => a,
            None => return errno::ENOSYS,
        };

        let mut elf_data = None;
        // TEAM_212: Strip leading '/' for initramfs lookup
        let lookup_name = path.strip_prefix('/').unwrap_or(path);
        for entry in archive.archive.iter() {
            if entry.name == lookup_name {
                elf_data = Some(entry.data);
                break;
            }
        }

        match elf_data {
            Some(d) => alloc::vec::Vec::from(d),
            None => return crate::syscall::errno_file::ENOENT,
        }
        // Lock released here
    };

    // 6. Convert args to &str slice
    let arg_refs: alloc::vec::Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    // 7. Spawn with args (lock no longer held)
    match crate::task::process::spawn_from_elf_with_args(&elf_data_copy, &arg_refs, &[]) {
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
            println!("[SYSCALL] spawn_args failed: {:?}", e);
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
            if mm_user::validate_user_buffer(current.ttbr0, status_ptr, 4, true)
                .is_ok()
            {
                if let Some(ptr) =
                    mm_user::user_va_to_kernel_ptr(current.ttbr0, status_ptr)
                {
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
            if mm_user::validate_user_buffer(current.ttbr0, status_ptr, 4, true)
                .is_ok()
            {
                if let Some(ptr) =
                    mm_user::user_va_to_kernel_ptr(current.ttbr0, status_ptr)
                {
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
