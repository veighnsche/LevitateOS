//! TEAM_233: File descriptor duplication syscalls.
//! TEAM_404: Added lseek, dup2, chdir, fchdir, ftruncate for coreutils compatibility.
//! TEAM_413: Updated to use syscall helper abstractions.
//! TEAM_415: Refactored ioctl to use helper functions.

use crate::memory::user as mm_user;
use crate::syscall::errno;
// TEAM_413: Import new syscall helpers
// TEAM_415: Added ioctl helpers
use crate::syscall::{
    get_fd, is_valid_fd,
    ioctl_get_termios, ioctl_read_termios, ioctl_write_i32, ioctl_read_i32,
    ioctl_write_u32, ioctl_read_u32,
};
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
/// TEAM_413: Updated to use is_valid_fd helper.
pub fn sys_fcntl(fd: i32, cmd: i32, arg: usize) -> i64 {
    // TEAM_413: Use is_valid_fd helper for validation
    if !is_valid_fd(fd as usize) {
        return errno::EBADF;
    }

    let task = current_task();

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
    // TEAM_416: Replace unwrap() with proper error handling for panic safety
    let ptr = match mm_user::user_va_to_kernel_ptr(task.ttbr0, pipefd_ptr) {
        Some(p) => p,
        None => return errno::EFAULT,
    };
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
/// TEAM_413: Updated to use get_fd helper.
///
/// Returns 1 if tty, 0 if not, negative errno on error.
pub fn sys_isatty(fd: i32) -> i64 {
    // TEAM_413: Use get_fd helper
    let entry = match get_fd(fd as usize) {
        Ok(e) => e,
        Err(e) => return e,
    };

    match &entry.fd_type {
        FdType::Stdin | FdType::Stdout | FdType::Stderr | FdType::PtySlave(_) => 1,
        _ => 0,
    }
}

/// TEAM_247: sys_ioctl - Control device.
/// TEAM_413: Updated to use get_fd and struct helpers.
/// TEAM_415: Refactored to use ioctl helper functions.
///
/// Returns 0 on success, negative errno on failure.
pub fn sys_ioctl(fd: usize, request: u64, arg: usize) -> i64 {
    use crate::fs::tty::{CONSOLE_TTY, TCGETS, TCSETS, TCSETSF, TCSETSW, TIOCGPTN, TIOCSPTLCK};

    // TEAM_413: Use get_fd helper
    let entry = match get_fd(fd) {
        Ok(e) => e,
        Err(e) => return e,
    };

    let task = current_task();

    match &entry.fd_type {
        FdType::Stdin | FdType::Stdout | FdType::Stderr => match request {
            TCGETS => {
                let termios = CONSOLE_TTY.lock().termios;
                ioctl_get_termios(task.ttbr0, arg, &termios)
            }
            TCSETS | TCSETSW | TCSETSF => match ioctl_read_termios(task.ttbr0, arg) {
                Ok(new_termios) => {
                    CONSOLE_TTY.lock().termios = new_termios;
                    0
                }
                Err(e) => e,
            },
            TIOCGPGRP => {
                let fg_pgid = *crate::task::FOREGROUND_PID.lock();
                ioctl_write_i32(task.ttbr0, arg, fg_pgid as i32)
            }
            TIOCSPGRP => match ioctl_read_i32(task.ttbr0, arg) {
                Ok(pgid) => {
                    *crate::task::FOREGROUND_PID.lock() = pgid as usize;
                    0
                }
                Err(e) => e,
            },
            TIOCSCTTY => 0, // Set controlling terminal (stub - just succeed)
            _ => errno::EINVAL,
        },
        FdType::PtyMaster(pair) => match request {
            TIOCGPTN => ioctl_write_u32(task.ttbr0, arg, pair.id as u32),
            TIOCSPTLCK => match ioctl_read_u32(task.ttbr0, arg) {
                Ok(val) => {
                    *pair.locked.lock() = val != 0;
                    0
                }
                Err(e) => e,
            },
            _ => errno::EINVAL,
        },
        FdType::PtySlave(pair) => match request {
            TCGETS => {
                let termios = pair.tty.lock().termios;
                ioctl_get_termios(task.ttbr0, arg, &termios)
            }
            TCSETS | TCSETSW | TCSETSF => match ioctl_read_termios(task.ttbr0, arg) {
                Ok(new_termios) => {
                    pair.tty.lock().termios = new_termios;
                    0
                }
                Err(e) => e,
            },
            _ => errno::EINVAL,
        },
        _ => errno::ENOTTY,
    }
}

/// TEAM_404: sys_lseek - Reposition file offset.
/// TEAM_413: Updated to use get_fd helper.
///
/// Returns new offset on success, negative errno on failure.
pub fn sys_lseek(fd: usize, offset: i64, whence: i32) -> i64 {
    use crate::fs::vfs::ops::SeekWhence;

    // TEAM_413: Use get_fd helper
    let entry = match get_fd(fd) {
        Ok(e) => e,
        Err(e) => return e,
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

/// TEAM_410: sys_truncate - Truncate file by path to specified length.
///
/// Truncates the file at the given path to the specified length.
/// If the file is longer, extra data is discarded. If shorter, the file is extended with zeros.
pub fn sys_truncate(pathname: usize, length: i64) -> i64 {
    use crate::syscall::read_user_cstring;
    use crate::fs::vfs::dentry::dcache;
    use crate::fs::vfs::error::VfsError;
    
    // Length must be non-negative
    if length < 0 {
        return errno::EINVAL;
    }
    
    let task = current_task();
    
    // TEAM_418: Use PATH_MAX from SSOT
    let mut path_buf = [0u8; crate::syscall::constants::PATH_MAX];
    let path_str = match read_user_cstring(task.ttbr0, pathname, &mut path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };
    
    // Look up the file in the VFS
    let dentry = match dcache().lookup(path_str) {
        Ok(d) => d,
        Err(VfsError::NotFound) => return errno::ENOENT,
        Err(_) => return errno::EIO,
    };
    
    let inode = match dentry.get_inode() {
        Some(i) => i,
        None => return errno::ENOENT,
    };
    
    // Must be a regular file
    if !inode.is_file() {
        return if inode.is_dir() { errno::EISDIR } else { errno::EINVAL };
    }
    
    // TEAM_410: Call the VFS truncate operation
    match inode.truncate(length as u64) {
        Ok(()) => 0,
        Err(VfsError::NotSupported) => errno::EROFS, // Read-only filesystem
        Err(VfsError::NoSpace) => errno::ENOSPC,
        Err(VfsError::FileTooLarge) => errno::EFBIG,
        Err(_) => errno::EIO,
    }
}

/// TEAM_410: sys_ftruncate - Truncate file to specified length by fd.
/// TEAM_413: Updated to use get_fd helper.
///
/// Truncates the file referred to by fd to the specified length.
/// The file must be open for writing.
pub fn sys_ftruncate(fd: usize, length: i64) -> i64 {
    use crate::fs::vfs::error::VfsError;

    // Length must be non-negative
    if length < 0 {
        return errno::EINVAL;
    }

    // TEAM_413: Use get_fd helper
    let entry = match get_fd(fd) {
        Ok(e) => e,
        Err(e) => return e,
    };

    // TEAM_410: Get the inode from the file and truncate
    match &entry.fd_type {
        FdType::VfsFile(file) => {
            // Check if file is open for writing
            if !file.flags.is_writable() {
                return errno::EINVAL; // POSIX says EINVAL if not open for writing
            }
            
            // Must be a regular file
            if !file.inode.is_file() {
                return if file.inode.is_dir() { errno::EISDIR } else { errno::EINVAL };
            }
            
            match file.inode.truncate(length as u64) {
                Ok(()) => 0,
                Err(VfsError::NotSupported) => errno::EROFS,
                Err(VfsError::NoSpace) => errno::ENOSPC,
                Err(VfsError::FileTooLarge) => errno::EFBIG,
                Err(_) => errno::EIO,
            }
        }
        // Other fd types (pipes, stdin/stdout, etc.) cannot be truncated
        _ => errno::EINVAL,
    }
}

/// TEAM_409: sys_pread64 - Read from fd at offset without changing file position.
/// TEAM_413: Updated to use get_fd helper.
///
/// Reads up to `count` bytes from file descriptor `fd` at offset `offset`
/// into the buffer at `buf_ptr`. The file offset is not changed.
///
/// # Arguments
/// * `fd` - File descriptor to read from
/// * `buf_ptr` - User buffer to read into
/// * `count` - Maximum bytes to read
/// * `offset` - File offset to read from
///
/// # Returns
/// Number of bytes read on success, negative errno on failure.
pub fn sys_pread64(fd: usize, buf_ptr: usize, count: usize, offset: i64) -> i64 {
    use crate::memory::user as mm_user;
    use crate::fs::vfs::error::VfsError;

    if count == 0 {
        return 0;
    }
    if offset < 0 {
        return errno::EINVAL;
    }

    let task = current_task();

    // TEAM_413: Use get_fd helper
    let entry = match get_fd(fd) {
        Ok(e) => e,
        Err(e) => return e,
    };

    match &entry.fd_type {
        FdType::VfsFile(file) => {
            // Check readable
            if !file.flags.is_readable() {
                return errno::EBADF;
            }

            // Validate user buffer
            if mm_user::validate_user_buffer(task.ttbr0, buf_ptr, count, true).is_err() {
                return errno::EFAULT;
            }

            // Read directly from inode at specified offset (without changing file.offset)
            let mut kbuf = alloc::vec![0u8; count];
            match file.inode.read(offset as u64, &mut kbuf) {
                Ok(n) => {
                    // Copy to user space
                    // TEAM_416: Replace unwrap() with proper error handling for panic safety
                    let dest = match mm_user::user_va_to_kernel_ptr(task.ttbr0, buf_ptr) {
                        Some(p) => p,
                        None => return errno::EFAULT,
                    };
                    unsafe {
                        core::ptr::copy_nonoverlapping(kbuf.as_ptr(), dest, n);
                    }
                    file.inode.touch_atime();
                    n as i64
                }
                Err(VfsError::BadFd) => errno::EBADF,
                Err(_) => errno::EIO,
            }
        }
        // Pipes and special files don't support positioned I/O
        FdType::PipeRead(_) | FdType::PipeWrite(_) => errno::ESPIPE,
        FdType::Stdin | FdType::Stdout | FdType::Stderr => errno::ESPIPE,
        _ => errno::EINVAL,
    }
}

/// TEAM_409: sys_pwrite64 - Write to fd at offset without changing file position.
/// TEAM_413: Updated to use get_fd helper.
///
/// Writes up to `count` bytes from the buffer at `buf_ptr` to file descriptor `fd`
/// at offset `offset`. The file offset is not changed.
///
/// # Arguments
/// * `fd` - File descriptor to write to
/// * `buf_ptr` - User buffer to write from
/// * `count` - Maximum bytes to write
/// * `offset` - File offset to write at
///
/// # Returns
/// Number of bytes written on success, negative errno on failure.
pub fn sys_pwrite64(fd: usize, buf_ptr: usize, count: usize, offset: i64) -> i64 {
    use crate::memory::user as mm_user;
    use crate::fs::vfs::error::VfsError;

    if count == 0 {
        return 0;
    }
    if offset < 0 {
        return errno::EINVAL;
    }

    let task = current_task();

    // TEAM_413: Use get_fd helper
    let entry = match get_fd(fd) {
        Ok(e) => e,
        Err(e) => return e,
    };

    match &entry.fd_type {
        FdType::VfsFile(file) => {
            // Check writable
            if !file.flags.is_writable() {
                return errno::EBADF;
            }

            // Validate user buffer
            if mm_user::validate_user_buffer(task.ttbr0, buf_ptr, count, false).is_err() {
                return errno::EFAULT;
            }

            // Copy from user space
            let mut kbuf = alloc::vec![0u8; count];
            // TEAM_416: Replace unwrap() with proper error handling for panic safety
            let src = match mm_user::user_va_to_kernel_ptr(task.ttbr0, buf_ptr) {
                Some(p) => p,
                None => return errno::EFAULT,
            };
            unsafe {
                core::ptr::copy_nonoverlapping(src, kbuf.as_mut_ptr(), count);
            }

            // Write directly to inode at specified offset (without changing file.offset)
            match file.inode.write(offset as u64, &kbuf) {
                Ok(n) => {
                    file.inode.touch_mtime();
                    n as i64
                }
                Err(VfsError::BadFd) => errno::EBADF,
                Err(VfsError::NotSupported) => errno::EROFS,
                Err(_) => errno::EIO,
            }
        }
        // Pipes and special files don't support positioned I/O
        FdType::PipeRead(_) | FdType::PipeWrite(_) => errno::ESPIPE,
        FdType::Stdin | FdType::Stdout | FdType::Stderr => errno::ESPIPE,
        _ => errno::EINVAL,
    }
}

// ============================================================================
// TEAM_406: chmod/chown syscalls (no-op for single-user OS per Q6)
// TEAM_415: Consolidated with helper for pathname validation.
// ============================================================================

/// TEAM_415: Validate that a user pathname pointer is readable.
/// Returns 0 on success, EFAULT if the pathname cannot be read.
#[inline]
fn validate_user_pathname(pathname: usize) -> i64 {
    let task = current_task();
    let mut buf = [0u8; 256];
    if crate::syscall::read_user_cstring(task.ttbr0, pathname, &mut buf).is_err() {
        errno::EFAULT
    } else {
        0
    }
}

/// TEAM_406: sys_chmod - No-op for single-user OS (per design decision Q6).
pub fn sys_chmod(pathname: usize, _mode: u32) -> i64 {
    validate_user_pathname(pathname)
}

/// TEAM_406: sys_fchmod - No-op for single-user OS.
pub fn sys_fchmod(fd: usize, _mode: u32) -> i64 {
    if is_valid_fd(fd) { 0 } else { errno::EBADF }
}

/// TEAM_406: sys_chown - No-op for single-user OS.
pub fn sys_chown(pathname: usize, _owner: u32, _group: u32) -> i64 {
    validate_user_pathname(pathname)
}

/// TEAM_406: sys_fchown - No-op for single-user OS.
pub fn sys_fchown(fd: usize, _owner: u32, _group: u32) -> i64 {
    if is_valid_fd(fd) { 0 } else { errno::EBADF }
}

/// TEAM_406: sys_fchmodat - No-op for single-user OS.
pub fn sys_fchmodat(_dirfd: i32, pathname: usize, _mode: u32, _flags: i32) -> i64 {
    validate_user_pathname(pathname)
}

/// TEAM_406: sys_fchownat - No-op for single-user OS.
pub fn sys_fchownat(_dirfd: i32, pathname: usize, _owner: u32, _group: u32, _flags: i32) -> i64 {
    validate_user_pathname(pathname)
}
