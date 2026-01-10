//! TEAM_233: File descriptor duplication syscalls.
//! TEAM_404: Added lseek, dup2, chdir, fchdir, ftruncate for coreutils compatibility.

use crate::syscall::errno;
use crate::task::current_task;
use crate::task::fd_table::FdType;

// TEAM_404: lseek whence constants
const SEEK_SET: i32 = 0;
const SEEK_CUR: i32 = 1;
const SEEK_END: i32 = 2;

// TEAM_394: Terminal foreground process group ioctls
const TIOCGPGRP: u64 = 0x540F; // Get foreground process group
const TIOCSPGRP: u64 = 0x5410; // Set foreground process group
const TIOCSCTTY: u64 = 0x540E; // Set controlling terminal

// TEAM_394: fcntl commands
const F_DUPFD: i32 = 0;
const F_GETFD: i32 = 1;
const F_SETFD: i32 = 2;
const F_GETFL: i32 = 3;
const F_SETFL: i32 = 4;
const F_SETPIPE_SZ: i32 = 1031;
const F_GETPIPE_SZ: i32 = 1032;

/// TEAM_394: sys_fcntl - File control operations.
///
/// Stub implementation for brush shell compatibility.
/// Currently supports F_GETFD, F_SETFD, F_GETFL, F_SETFL, F_SETPIPE_SZ, F_GETPIPE_SZ.
pub fn sys_fcntl(fd: i32, cmd: i32, arg: usize) -> i64 {
    let task = current_task();
    let fd_table = task.fd_table.lock();

    // Verify fd is valid
    if fd_table.get(fd as usize).is_none() {
        return errno::EBADF;
    }
    drop(fd_table);

    match cmd {
        F_DUPFD => {
            // Duplicate fd to >= arg
            let mut fd_table = task.fd_table.lock();
            match fd_table.dup(fd as usize) {
                Some(new_fd) => new_fd as i64,
                None => errno::EMFILE,
            }
        }
        F_GETFD => {
            // Get close-on-exec flag (stub: always return 0)
            0
        }
        F_SETFD => {
            // Set close-on-exec flag (stub: ignore)
            0
        }
        F_GETFL => {
            // Get file status flags (stub: return O_RDWR)
            2 // O_RDWR
        }
        F_SETFL => {
            // Set file status flags (stub: ignore)
            0
        }
        F_SETPIPE_SZ => {
            // Set pipe buffer size (stub: succeed with requested size)
            log::trace!("[SYSCALL] fcntl({}, F_SETPIPE_SZ, {})", fd, arg);
            arg as i64
        }
        F_GETPIPE_SZ => {
            // Get pipe buffer size (stub: return default 64KB)
            log::trace!("[SYSCALL] fcntl({}, F_GETPIPE_SZ)", fd);
            65536
        }
        _ => {
            log::warn!("[SYSCALL] fcntl({}, {}, {}) - unsupported cmd", fd, cmd, arg);
            errno::EINVAL
        }
    }
}

/// TEAM_233: sys_dup - Duplicate a file descriptor to lowest available.
///
/// Returns new fd on success, negative errno on failure.
pub fn sys_dup(oldfd: usize) -> i64 {
    let task = current_task();
    let mut fd_table = task.fd_table.lock();

    match fd_table.dup(oldfd) {
        Some(newfd) => newfd as i64,
        None => errno::EBADF,
    }
}

/// TEAM_233: sys_dup3 - Duplicate a file descriptor to specific number.
///
/// If newfd is already open, it is closed first.
/// Returns newfd on success, negative errno on failure.
pub fn sys_dup3(oldfd: usize, newfd: usize, _flags: u32) -> i64 {
    if oldfd == newfd {
        return errno::EINVAL;
    }

    let task = current_task();
    let mut fd_table = task.fd_table.lock();

    match fd_table.dup_to(oldfd, newfd) {
        Some(fd) => fd as i64,
        None => errno::EBADF,
    }
}

/// TEAM_233: sys_pipe2 - Create a pipe.
///
/// Creates a pipe and returns two file descriptors in pipefd array.
/// pipefd[0] is the read end, pipefd[1] is the write end.
///
/// Returns 0 on success, negative errno on failure.
pub fn sys_pipe2(pipefd_ptr: usize, _flags: u32) -> i64 {
    use crate::fs::pipe::Pipe;
    use crate::memory::user as mm_user;
    use crate::task::fd_table::FdType;

    let task = current_task();

    // Validate user buffer (2 * sizeof(i32) = 8 bytes)
    if mm_user::validate_user_buffer(task.ttbr0, pipefd_ptr, 8, true).is_err() {
        return errno::EFAULT;
    }

    // Create the pipe
    let pipe = Pipe::new();

    // Allocate file descriptors
    let (read_fd, write_fd) = {
        let mut fd_table = task.fd_table.lock();

        let read_fd = match fd_table.alloc(FdType::PipeRead(pipe.clone())) {
            Some(fd) => fd,
            None => return errno::EMFILE,
        };

        let write_fd = match fd_table.alloc(FdType::PipeWrite(pipe.clone())) {
            Some(fd) => fd,
            None => {
                // Clean up read fd
                fd_table.close(read_fd);
                return errno::EMFILE;
            }
        };

        (read_fd, write_fd)
    };

    // Write fds to user space
    // SAFETY: validate_user_buffer confirmed buffer is accessible
    let ptr = mm_user::user_va_to_kernel_ptr(task.ttbr0, pipefd_ptr).unwrap();
    unsafe {
        let fds = ptr as *mut [i32; 2];
        (*fds)[0] = read_fd as i32;
        (*fds)[1] = write_fd as i32;
    }

    log::trace!(
        "[SYSCALL] pipe2: created pipe fds [{}, {}]",
        read_fd,
        write_fd
    );
    0
}

/// TEAM_244: sys_isatty - Check if fd refers to a terminal.
///
/// Returns 1 if tty, 0 if not, negative errno on error.
pub fn sys_isatty(fd: i32) -> i64 {
    let task = current_task();
    let fd_table = task.fd_table.lock();
    let entry = match fd_table.get(fd as usize) {
        Some(e) => e,
        None => return errno::EBADF,
    };

    match &entry.fd_type {
        FdType::Stdin | FdType::Stdout | FdType::Stderr | FdType::PtySlave(_) => 1,
        _ => 0,
    }
}

/// TEAM_247: sys_ioctl - Control device.
///
/// Returns 0 on success, negative errno on failure.
pub fn sys_ioctl(fd: usize, request: u64, arg: usize) -> i64 {
    use crate::fs::tty::{
        CONSOLE_TTY, TCGETS, TCSETS, TCSETSF, TCSETSW, TIOCGPTN, TIOCSPTLCK, Termios,
    };
    use crate::memory::user as mm_user;

    let task = current_task();
    let fd_table = task.fd_table.lock();
    let entry = match fd_table.get(fd) {
        Some(e) => e.clone(),
        None => return errno::EBADF,
    };
    drop(fd_table);

    // For now, we only support ioctls on TTY devices (stdin/stdout/stderr)
    match &entry.fd_type {
        FdType::Stdin | FdType::Stdout | FdType::Stderr => {
            match request {
                TCGETS => {
                    // Get terminal attributes
                    let tty = CONSOLE_TTY.lock();
                    let termios = tty.termios;
                    drop(tty);

                    // Copy to user space
                    let size = core::mem::size_of::<Termios>();
                    if mm_user::validate_user_buffer(task.ttbr0, arg, size, true).is_err() {
                        return errno::EFAULT;
                    }

                    if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, arg) {
                        unsafe {
                            let user_termios = ptr as *mut Termios;
                            *user_termios = termios;
                        }
                        0
                    } else {
                        errno::EFAULT
                    }
                }
                TCSETS | TCSETSW | TCSETSF => {
                    // Set terminal attributes
                    let size = core::mem::size_of::<Termios>();
                    if mm_user::validate_user_buffer(task.ttbr0, arg, size, false).is_err() {
                        return errno::EFAULT;
                    }

                    if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, arg) {
                        let new_termios = unsafe { *(ptr as *const Termios) };
                        let mut tty = CONSOLE_TTY.lock();
                        tty.termios = new_termios;
                        0
                    } else {
                        errno::EFAULT
                    }
                }
                // TEAM_394: Get foreground process group
                TIOCGPGRP => {
                    if mm_user::validate_user_buffer(task.ttbr0, arg, core::mem::size_of::<i32>(), true).is_err() {
                        return errno::EFAULT;
                    }
                    let fg_pgid = *crate::task::FOREGROUND_PID.lock();
                    if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, arg) {
                        unsafe { *(ptr as *mut i32) = fg_pgid as i32; }
                        0
                    } else {
                        errno::EFAULT
                    }
                }
                // TEAM_394: Set foreground process group
                TIOCSPGRP => {
                    if mm_user::validate_user_buffer(task.ttbr0, arg, core::mem::size_of::<i32>(), false).is_err() {
                        return errno::EFAULT;
                    }
                    if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, arg) {
                        let pgid = unsafe { *(ptr as *const i32) };
                        *crate::task::FOREGROUND_PID.lock() = pgid as usize;
                        0
                    } else {
                        errno::EFAULT
                    }
                }
                // TEAM_394: Set controlling terminal (stub - just succeed)
                TIOCSCTTY => 0,
                _ => errno::EINVAL,
            }
        }
        FdType::PtyMaster(pair) => match request {
            TIOCGPTN => {
                if mm_user::validate_user_buffer(task.ttbr0, arg, core::mem::size_of::<u32>(), true)
                    .is_err()
                {
                    return errno::EFAULT;
                }
                if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, arg) {
                    unsafe {
                        *(ptr as *mut u32) = pair.id as u32;
                    }
                    0
                } else {
                    errno::EFAULT
                }
            }
            TIOCSPTLCK => {
                if mm_user::validate_user_buffer(
                    task.ttbr0,
                    arg,
                    core::mem::size_of::<u32>(),
                    false,
                )
                .is_err()
                {
                    return errno::EFAULT;
                }
                if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, arg) {
                    let val = unsafe { *(ptr as *const u32) };
                    *pair.locked.lock() = val != 0;
                    0
                } else {
                    errno::EFAULT
                }
            }
            _ => errno::EINVAL,
        },
        FdType::PtySlave(pair) => match request {
            TCGETS => {
                let tty = pair.tty.lock();
                let termios = tty.termios;
                drop(tty);

                let size = core::mem::size_of::<Termios>();
                if mm_user::validate_user_buffer(task.ttbr0, arg, size, true).is_err() {
                    return errno::EFAULT;
                }
                if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, arg) {
                    unsafe {
                        *(ptr as *mut Termios) = termios;
                    }
                    0
                } else {
                    errno::EFAULT
                }
            }
            TCSETS | TCSETSW | TCSETSF => {
                let size = core::mem::size_of::<Termios>();
                if mm_user::validate_user_buffer(task.ttbr0, arg, size, false).is_err() {
                    return errno::EFAULT;
                }
                if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, arg) {
                    let new_termios = unsafe { *(ptr as *const Termios) };
                    pair.tty.lock().termios = new_termios;
                    0
                } else {
                    errno::EFAULT
                }
            }
            _ => errno::EINVAL,
        },
        _ => errno::ENOTTY,
    }
}

/// TEAM_404: sys_lseek - Reposition file offset.
///
/// Returns new offset on success, negative errno on failure.
pub fn sys_lseek(fd: usize, offset: i64, whence: i32) -> i64 {
    use crate::fs::vfs::ops::SeekWhence;
    
    let task = current_task();
    let fd_table = task.fd_table.lock();
    
    let entry = match fd_table.get(fd) {
        Some(e) => e,
        None => return errno::EBADF,
    };
    
    match &entry.fd_type {
        FdType::VfsFile(file) => {
            let seek_whence = match whence {
                SEEK_SET => SeekWhence::Set,
                SEEK_CUR => SeekWhence::Cur,
                SEEK_END => SeekWhence::End,
                _ => return errno::EINVAL,
            };
            
            match file.seek(offset, seek_whence) {
                Ok(new_offset) => new_offset as i64,
                Err(_) => errno::EINVAL,
            }
        }
        FdType::Stdin | FdType::Stdout | FdType::Stderr => errno::ESPIPE,
        FdType::PipeRead(_) | FdType::PipeWrite(_) => errno::ESPIPE,
        _ => errno::EINVAL,
    }
}

/// TEAM_404: sys_dup2 - Duplicate fd to specific number (legacy version).
///
/// Maps to dup3 with flags=0.
pub fn sys_dup2(oldfd: usize, newfd: usize) -> i64 {
    if oldfd == newfd {
        // Unlike dup3, dup2 returns newfd if oldfd == newfd (and oldfd is valid)
        let task = current_task();
        let fd_table = task.fd_table.lock();
        if fd_table.get(oldfd).is_some() {
            return newfd as i64;
        } else {
            return errno::EBADF;
        }
    }
    sys_dup3(oldfd, newfd, 0)
}

/// TEAM_404: sys_chdir - Change current working directory.
///
/// Returns 0 on success, negative errno on failure.
pub fn sys_chdir(path_ptr: usize) -> i64 {
    let task = current_task();
    let mut path_buf = [0u8; 256];
    
    let path = match crate::syscall::read_user_cstring(task.ttbr0, path_ptr, &mut path_buf) {
        Ok(p) => p,
        Err(e) => return e,
    };
    
    // Update task's cwd (assume path exists - proper validation TODO)
    let mut cwd = task.cwd.lock();
    cwd.clear();
    cwd.push_str(path);
    0
}

/// TEAM_404: sys_fchdir - Change cwd by fd.
///
/// Stub: returns ENOSYS (not commonly used by coreutils).
pub fn sys_fchdir(_fd: usize) -> i64 {
    // TODO: Implement when directory fd tracking is added
    errno::ENOSYS
}

/// TEAM_404: sys_ftruncate - Truncate file to specified length.
///
/// Stub: returns 0 (success) for now - full impl needs inode truncate support.
pub fn sys_ftruncate(fd: usize, _length: i64) -> i64 {
    let task = current_task();
    let fd_table = task.fd_table.lock();
    
    // Just verify fd is valid
    if fd_table.get(fd).is_none() {
        return errno::EBADF;
    }
    
    // TODO: Implement actual truncation
    0
}

/// TEAM_404: sys_pread64 - Read from fd at offset without changing file position.
///
/// Stub: returns ENOSYS - full impl needs positioned read support.
pub fn sys_pread64(_fd: usize, _buf_ptr: usize, _count: usize, _offset: i64) -> i64 {
    // TODO: Implement when File has read_at method
    errno::ENOSYS
}

/// TEAM_404: sys_pwrite64 - Write to fd at offset without changing file position.
///
/// Stub: returns ENOSYS - full impl needs positioned write support.
pub fn sys_pwrite64(_fd: usize, _buf_ptr: usize, _count: usize, _offset: i64) -> i64 {
    // TODO: Implement when File has write_at method
    errno::ENOSYS
}

// ============================================================================
// TEAM_406: chmod/chown syscalls (no-op for single-user OS per Q6)
// ============================================================================

/// TEAM_406: sys_chmod - Change file permissions.
///
/// No-op implementation for single-user OS (per design decision Q6).
/// Just validates pathname is readable, doesn't check file exists.
pub fn sys_chmod(pathname: usize, _mode: u32) -> i64 {
    let task = current_task();
    let mut buf = [0u8; 256];
    
    // Validate pathname is readable
    if crate::syscall::read_user_cstring(task.ttbr0, pathname, &mut buf).is_err() {
        return errno::EFAULT;
    }
    
    0 // Success - no actual mode change (single-user OS)
}

/// TEAM_406: sys_fchmod - Change file permissions by file descriptor.
///
/// No-op implementation for single-user OS (per design decision Q6).
pub fn sys_fchmod(fd: usize, _mode: u32) -> i64 {
    let task = current_task();
    let fd_table = task.fd_table.lock();
    
    if fd_table.get(fd).is_none() {
        return errno::EBADF;
    }
    
    0 // Success - no actual mode change (single-user OS)
}

/// TEAM_406: sys_chown - Change file owner and group.
///
/// No-op implementation for single-user OS (per design decision Q6).
pub fn sys_chown(pathname: usize, _owner: u32, _group: u32) -> i64 {
    let task = current_task();
    let mut buf = [0u8; 256];
    
    // Validate pathname is readable
    if crate::syscall::read_user_cstring(task.ttbr0, pathname, &mut buf).is_err() {
        return errno::EFAULT;
    }
    
    0 // Success - no actual ownership change (single-user OS)
}

/// TEAM_406: sys_fchown - Change file owner and group by file descriptor.
///
/// No-op implementation for single-user OS (per design decision Q6).
pub fn sys_fchown(fd: usize, _owner: u32, _group: u32) -> i64 {
    let task = current_task();
    let fd_table = task.fd_table.lock();
    
    if fd_table.get(fd).is_none() {
        return errno::EBADF;
    }
    
    0 // Success - no actual ownership change (single-user OS)
}

/// TEAM_406: sys_fchmodat - Change file permissions at path relative to directory fd.
///
/// No-op implementation for single-user OS (per design decision Q6).
pub fn sys_fchmodat(_dirfd: i32, pathname: usize, _mode: u32, _flags: i32) -> i64 {
    let task = current_task();
    let mut buf = [0u8; 256];
    
    // Validate pathname is readable
    if crate::syscall::read_user_cstring(task.ttbr0, pathname, &mut buf).is_err() {
        return errno::EFAULT;
    }
    
    0 // Success - no actual mode change (single-user OS)
}

/// TEAM_406: sys_fchownat - Change file owner at path relative to directory fd.
///
/// No-op implementation for single-user OS (per design decision Q6).
pub fn sys_fchownat(_dirfd: i32, pathname: usize, _owner: u32, _group: u32, _flags: i32) -> i64 {
    let task = current_task();
    let mut buf = [0u8; 256];
    
    // Validate pathname is readable
    if crate::syscall::read_user_cstring(task.ttbr0, pathname, &mut buf).is_err() {
        return errno::EFAULT;
    }
    
    0 // Success - no actual ownership change (single-user OS)
}
