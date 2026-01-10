//! TEAM_233: File descriptor duplication syscalls.
//! TEAM_404: Added lseek, dup2, chdir, fchdir, ftruncate for coreutils compatibility.
//! TEAM_413: Updated to use syscall helper abstractions.
//! TEAM_415: Refactored ioctl to use helper functions.
//! TEAM_421: Updated all functions to return SyscallResult.

use crate::memory::user as mm_user;
// TEAM_413: Import new syscall helpers
// TEAM_415: Added ioctl helpers
// TEAM_420: Direct linux_raw_sys import, no shims
// TEAM_421: Import SyscallResult
use crate::syscall::{
    get_fd, is_valid_fd,
    ioctl_get_termios, ioctl_read_termios, ioctl_write_i32, ioctl_read_i32,
    ioctl_write_u32, ioctl_read_u32,
    SyscallResult,
};
use crate::task::current_task;
use crate::task::fd_table::FdType;
use linux_raw_sys::errno::{
    EBADF, EFAULT, EFBIG, EINVAL, EIO, EISDIR, EMFILE, ENOENT, ENOSPC, ENOSYS, ENOTTY, EROFS, ESPIPE,
};

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
/// TEAM_421: Updated to return SyscallResult.
pub fn sys_fcntl(fd: i32, cmd: i32, arg: usize) -> SyscallResult {
    // TEAM_413: Use is_valid_fd helper for validation
    if !is_valid_fd(fd as usize) {
        return Err(EBADF);
    }

    let task = current_task();

    match cmd {
        F_DUPFD => {
            // Duplicate fd to >= arg
            let mut fd_table = task.fd_table.lock();
            match fd_table.dup(fd as usize) {
                Some(new_fd) => Ok(new_fd as i64),
                None => Err(EMFILE),
            }
        }
        F_GETFD => {
            // Get close-on-exec flag (stub: always return 0)
            Ok(0)
        }
        F_SETFD => {
            // Set close-on-exec flag (stub: ignore)
            Ok(0)
        }
        F_GETFL => {
            // Get file status flags (stub: return O_RDWR)
            Ok(2) // O_RDWR
        }
        F_SETFL => {
            // Set file status flags (stub: ignore)
            Ok(0)
        }
        F_SETPIPE_SZ => {
            // Set pipe buffer size (stub: succeed with requested size)
            log::trace!("[SYSCALL] fcntl({}, F_SETPIPE_SZ, {})", fd, arg);
            Ok(arg as i64)
        }
        F_GETPIPE_SZ => {
            // Get pipe buffer size (stub: return default 64KB)
            log::trace!("[SYSCALL] fcntl({}, F_GETPIPE_SZ)", fd);
            Ok(65536)
        }
        _ => {
            log::warn!("[SYSCALL] fcntl({}, {}, {}) - unsupported cmd", fd, cmd, arg);
            Err(EINVAL)
        }
    }
}

/// TEAM_233: sys_dup - Duplicate a file descriptor to lowest available.
/// TEAM_421: Updated to return SyscallResult.
///
/// Returns new fd on success, EBADF on failure.
pub fn sys_dup(oldfd: usize) -> SyscallResult {
    let task = current_task();
    let mut fd_table = task.fd_table.lock();

    match fd_table.dup(oldfd) {
        Some(newfd) => Ok(newfd as i64),
        None => Err(EBADF),
    }
}

/// TEAM_233: sys_dup3 - Duplicate a file descriptor to specific number.
/// TEAM_421: Updated to return SyscallResult.
///
/// If newfd is already open, it is closed first.
/// Returns newfd on success, EINVAL/EBADF on failure.
pub fn sys_dup3(oldfd: usize, newfd: usize, _flags: u32) -> SyscallResult {
    if oldfd == newfd {
        return Err(EINVAL);
    }

    let task = current_task();
    let mut fd_table = task.fd_table.lock();

    match fd_table.dup_to(oldfd, newfd) {
        Some(fd) => Ok(fd as i64),
        None => Err(EBADF),
    }
}

/// TEAM_233: sys_pipe2 - Create a pipe.
/// TEAM_421: Updated to return SyscallResult.
///
/// Creates a pipe and returns two file descriptors in pipefd array.
/// pipefd[0] is the read end, pipefd[1] is the write end.
///
/// Returns 0 on success, EFAULT/EMFILE on failure.
pub fn sys_pipe2(pipefd_ptr: usize, _flags: u32) -> SyscallResult {
    use crate::fs::pipe::Pipe;
    use crate::memory::user as mm_user;
    use crate::task::fd_table::FdType;

    let task = current_task();

    // Validate user buffer (2 * sizeof(i32) = 8 bytes)
    if mm_user::validate_user_buffer(task.ttbr0, pipefd_ptr, 8, true).is_err() {
        return Err(EFAULT);
    }

    // Create the pipe
    let pipe = Pipe::new();

    // Allocate file descriptors
    let (read_fd, write_fd) = {
        let mut fd_table = task.fd_table.lock();

        let read_fd = match fd_table.alloc(FdType::PipeRead(pipe.clone())) {
            Some(fd) => fd,
            None => return Err(EMFILE),
        };

        let write_fd = match fd_table.alloc(FdType::PipeWrite(pipe.clone())) {
            Some(fd) => fd,
            None => {
                // Clean up read fd
                fd_table.close(read_fd);
                return Err(EMFILE);
            }
        };

        (read_fd, write_fd)
    };

    // Write fds to user space
    // TEAM_416: Replace unwrap() with proper error handling for panic safety
    let ptr = match mm_user::user_va_to_kernel_ptr(task.ttbr0, pipefd_ptr) {
        Some(p) => p,
        None => return Err(EFAULT),
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
    Ok(0)
}

/// TEAM_244: sys_isatty - Check if fd refers to a terminal.
/// TEAM_413: Updated to use get_fd helper.
/// TEAM_421: Updated to return SyscallResult.
///
/// Returns 1 if tty, 0 if not, EBADF on error.
pub fn sys_isatty(fd: i32) -> SyscallResult {
    // TEAM_413: Use get_fd helper
    let entry = get_fd(fd as usize)?;

    match &entry.fd_type {
        FdType::Stdin | FdType::Stdout | FdType::Stderr | FdType::PtySlave(_) => Ok(1),
        _ => Ok(0),
    }
}

/// TEAM_247: sys_ioctl - Control device.
/// TEAM_413: Updated to use get_fd and struct helpers.
/// TEAM_415: Refactored to use ioctl helper functions.
/// TEAM_421: Updated to return SyscallResult.
///
/// Returns 0 on success, errno on failure.
pub fn sys_ioctl(fd: usize, request: u64, arg: usize) -> SyscallResult {
    use crate::fs::tty::{CONSOLE_TTY, TCGETS, TCSETS, TCSETSF, TCSETSW, TIOCGPTN, TIOCSPTLCK};

    // TEAM_413: Use get_fd helper
    let entry = get_fd(fd)?;

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
                    Ok(0)
                }
                Err(e) => Err(e),
            },
            TIOCGPGRP => {
                let fg_pgid = *crate::task::FOREGROUND_PID.lock();
                ioctl_write_i32(task.ttbr0, arg, fg_pgid as i32)
            }
            TIOCSPGRP => match ioctl_read_i32(task.ttbr0, arg) {
                Ok(pgid) => {
                    *crate::task::FOREGROUND_PID.lock() = pgid as usize;
                    Ok(0)
                }
                Err(e) => Err(e),
            },
            TIOCSCTTY => Ok(0), // Set controlling terminal (stub - just succeed)
            _ => Err(EINVAL),
        },
        FdType::PtyMaster(pair) => match request {
            TIOCGPTN => ioctl_write_u32(task.ttbr0, arg, pair.id as u32),
            TIOCSPTLCK => match ioctl_read_u32(task.ttbr0, arg) {
                Ok(val) => {
                    *pair.locked.lock() = val != 0;
                    Ok(0)
                }
                Err(e) => Err(e),
            },
            _ => Err(EINVAL),
        },
        FdType::PtySlave(pair) => match request {
            TCGETS => {
                let termios = pair.tty.lock().termios;
                ioctl_get_termios(task.ttbr0, arg, &termios)
            }
            TCSETS | TCSETSW | TCSETSF => match ioctl_read_termios(task.ttbr0, arg) {
                Ok(new_termios) => {
                    pair.tty.lock().termios = new_termios;
                    Ok(0)
                }
                Err(e) => Err(e),
            },
            _ => Err(EINVAL),
        },
        _ => Err(ENOTTY),
    }
}

/// TEAM_404: sys_lseek - Reposition file offset.
/// TEAM_413: Updated to use get_fd helper.
/// TEAM_421: Updated to return SyscallResult.
///
/// Returns new offset on success, errno on failure.
pub fn sys_lseek(fd: usize, offset: i64, whence: i32) -> SyscallResult {
    use crate::fs::vfs::ops::SeekWhence;

    // TEAM_413: Use get_fd helper
    let entry = get_fd(fd)?;

    match &entry.fd_type {
        FdType::VfsFile(file) => {
            let seek_whence = match whence {
                SEEK_SET => SeekWhence::Set,
                SEEK_CUR => SeekWhence::Cur,
                SEEK_END => SeekWhence::End,
                _ => return Err(EINVAL),
            };

            match file.seek(offset, seek_whence) {
                Ok(new_offset) => Ok(new_offset as i64),
                Err(_) => Err(EINVAL),
            }
        }
        FdType::Stdin | FdType::Stdout | FdType::Stderr => Err(ESPIPE),
        FdType::PipeRead(_) | FdType::PipeWrite(_) => Err(ESPIPE),
        _ => Err(EINVAL),
    }
}

/// TEAM_404: sys_dup2 - Duplicate fd to specific number (legacy version).
/// TEAM_421: Updated to return SyscallResult.
///
/// Maps to dup3 with flags=0.
pub fn sys_dup2(oldfd: usize, newfd: usize) -> SyscallResult {
    if oldfd == newfd {
        // Unlike dup3, dup2 returns newfd if oldfd == newfd (and oldfd is valid)
        let task = current_task();
        let fd_table = task.fd_table.lock();
        if fd_table.get(oldfd).is_some() {
            return Ok(newfd as i64);
        } else {
            return Err(EBADF);
        }
    }
    sys_dup3(oldfd, newfd, 0)
}

/// TEAM_404: sys_chdir - Change current working directory.
/// TEAM_421: Updated to return SyscallResult.
///
/// Returns 0 on success, errno on failure.
pub fn sys_chdir(path_ptr: usize) -> SyscallResult {
    let task = current_task();
    let mut path_buf = [0u8; 256];

    let path = crate::syscall::read_user_cstring(task.ttbr0, path_ptr, &mut path_buf)?;

    // Update task's cwd (assume path exists - proper validation TODO)
    let mut cwd = task.cwd.lock();
    cwd.clear();
    cwd.push_str(path);
    Ok(0)
}

/// TEAM_404: sys_fchdir - Change cwd by fd.
/// TEAM_421: Updated to return SyscallResult.
///
/// Stub: returns ENOSYS (not commonly used by coreutils).
pub fn sys_fchdir(_fd: usize) -> SyscallResult {
    // TODO: Implement when directory fd tracking is added
    Err(ENOSYS)
}

/// TEAM_410: sys_truncate - Truncate file by path to specified length.
/// TEAM_421: Updated to return SyscallResult.
///
/// Truncates the file at the given path to the specified length.
/// If the file is longer, extra data is discarded. If shorter, the file is extended with zeros.
pub fn sys_truncate(pathname: usize, length: i64) -> SyscallResult {
    use crate::syscall::read_user_cstring;
    use crate::fs::vfs::dentry::dcache;
    use crate::fs::vfs::error::VfsError;

    // Length must be non-negative
    if length < 0 {
        return Err(EINVAL);
    }

    let task = current_task();

    // TEAM_418: Use PATH_MAX from SSOT
    let mut path_buf = [0u8; linux_raw_sys::general::PATH_MAX as usize];
    let path_str = read_user_cstring(task.ttbr0, pathname, &mut path_buf)?;

    // Look up the file in the VFS
    let dentry = match dcache().lookup(path_str) {
        Ok(d) => d,
        Err(VfsError::NotFound) => return Err(ENOENT),
        Err(_) => return Err(EIO),
    };

    let inode = match dentry.get_inode() {
        Some(i) => i,
        None => return Err(ENOENT),
    };

    // Must be a regular file
    if !inode.is_file() {
        return if inode.is_dir() { Err(EISDIR) } else { Err(EINVAL) };
    }

    // TEAM_410: Call the VFS truncate operation
    match inode.truncate(length as u64) {
        Ok(()) => Ok(0),
        Err(VfsError::NotSupported) => Err(EROFS), // Read-only filesystem
        Err(VfsError::NoSpace) => Err(ENOSPC),
        Err(VfsError::FileTooLarge) => Err(EFBIG),
        Err(_) => Err(EIO),
    }
}

/// TEAM_410: sys_ftruncate - Truncate file to specified length by fd.
/// TEAM_413: Updated to use get_fd helper.
/// TEAM_421: Updated to return SyscallResult.
///
/// Truncates the file referred to by fd to the specified length.
/// The file must be open for writing.
pub fn sys_ftruncate(fd: usize, length: i64) -> SyscallResult {
    use crate::fs::vfs::error::VfsError;

    // Length must be non-negative
    if length < 0 {
        return Err(EINVAL);
    }

    // TEAM_413: Use get_fd helper
    let entry = get_fd(fd)?;

    // TEAM_410: Get the inode from the file and truncate
    match &entry.fd_type {
        FdType::VfsFile(file) => {
            // Check if file is open for writing
            if !file.flags.is_writable() {
                return Err(EINVAL); // POSIX says EINVAL if not open for writing
            }

            // Must be a regular file
            if !file.inode.is_file() {
                return if file.inode.is_dir() { Err(EISDIR) } else { Err(EINVAL) };
            }

            match file.inode.truncate(length as u64) {
                Ok(()) => Ok(0),
                Err(VfsError::NotSupported) => Err(EROFS),
                Err(VfsError::NoSpace) => Err(ENOSPC),
                Err(VfsError::FileTooLarge) => Err(EFBIG),
                Err(_) => Err(EIO),
            }
        }
        // Other fd types (pipes, stdin/stdout, etc.) cannot be truncated
        _ => Err(EINVAL),
    }
}

/// TEAM_409: sys_pread64 - Read from fd at offset without changing file position.
/// TEAM_413: Updated to use get_fd helper.
/// TEAM_421: Updated to return SyscallResult.
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
/// Number of bytes read on success, errno on failure.
pub fn sys_pread64(fd: usize, buf_ptr: usize, count: usize, offset: i64) -> SyscallResult {
    use crate::memory::user as mm_user;
    use crate::fs::vfs::error::VfsError;

    if count == 0 {
        return Ok(0);
    }
    if offset < 0 {
        return Err(EINVAL);
    }

    let task = current_task();

    // TEAM_413: Use get_fd helper
    let entry = get_fd(fd)?;

    match &entry.fd_type {
        FdType::VfsFile(file) => {
            // Check readable
            if !file.flags.is_readable() {
                return Err(EBADF);
            }

            // Validate user buffer
            if mm_user::validate_user_buffer(task.ttbr0, buf_ptr, count, true).is_err() {
                return Err(EFAULT);
            }

            // Read directly from inode at specified offset (without changing file.offset)
            let mut kbuf = alloc::vec![0u8; count];
            match file.inode.read(offset as u64, &mut kbuf) {
                Ok(n) => {
                    // Copy to user space
                    // TEAM_416: Replace unwrap() with proper error handling for panic safety
                    let dest = match mm_user::user_va_to_kernel_ptr(task.ttbr0, buf_ptr) {
                        Some(p) => p,
                        None => return Err(EFAULT),
                    };
                    unsafe {
                        core::ptr::copy_nonoverlapping(kbuf.as_ptr(), dest, n);
                    }
                    file.inode.touch_atime();
                    Ok(n as i64)
                }
                Err(VfsError::BadFd) => Err(EBADF),
                Err(_) => Err(EIO),
            }
        }
        // Pipes and special files don't support positioned I/O
        FdType::PipeRead(_) | FdType::PipeWrite(_) => Err(ESPIPE),
        FdType::Stdin | FdType::Stdout | FdType::Stderr => Err(ESPIPE),
        _ => Err(EINVAL),
    }
}

/// TEAM_409: sys_pwrite64 - Write to fd at offset without changing file position.
/// TEAM_413: Updated to use get_fd helper.
/// TEAM_421: Updated to return SyscallResult.
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
/// Number of bytes written on success, errno on failure.
pub fn sys_pwrite64(fd: usize, buf_ptr: usize, count: usize, offset: i64) -> SyscallResult {
    use crate::memory::user as mm_user;
    use crate::fs::vfs::error::VfsError;

    if count == 0 {
        return Ok(0);
    }
    if offset < 0 {
        return Err(EINVAL);
    }

    let task = current_task();

    // TEAM_413: Use get_fd helper
    let entry = get_fd(fd)?;

    match &entry.fd_type {
        FdType::VfsFile(file) => {
            // Check writable
            if !file.flags.is_writable() {
                return Err(EBADF);
            }

            // Validate user buffer
            if mm_user::validate_user_buffer(task.ttbr0, buf_ptr, count, false).is_err() {
                return Err(EFAULT);
            }

            // Copy from user space
            let mut kbuf = alloc::vec![0u8; count];
            // TEAM_416: Replace unwrap() with proper error handling for panic safety
            let src = match mm_user::user_va_to_kernel_ptr(task.ttbr0, buf_ptr) {
                Some(p) => p,
                None => return Err(EFAULT),
            };
            unsafe {
                core::ptr::copy_nonoverlapping(src, kbuf.as_mut_ptr(), count);
            }

            // Write directly to inode at specified offset (without changing file.offset)
            match file.inode.write(offset as u64, &kbuf) {
                Ok(n) => {
                    file.inode.touch_mtime();
                    Ok(n as i64)
                }
                Err(VfsError::BadFd) => Err(EBADF),
                Err(VfsError::NotSupported) => Err(EROFS),
                Err(_) => Err(EIO),
            }
        }
        // Pipes and special files don't support positioned I/O
        FdType::PipeRead(_) | FdType::PipeWrite(_) => Err(ESPIPE),
        FdType::Stdin | FdType::Stdout | FdType::Stderr => Err(ESPIPE),
        _ => Err(EINVAL),
    }
}

// ============================================================================
// TEAM_406: chmod/chown syscalls (no-op for single-user OS per Q6)
// TEAM_415: Consolidated with helper for pathname validation.
// TEAM_421: Updated all functions to return SyscallResult.
// ============================================================================

/// TEAM_415: Validate that a user pathname pointer is readable.
/// TEAM_421: Returns SyscallResult - Ok(0) on success, Err(EFAULT) if pathname cannot be read.
#[inline]
fn validate_user_pathname(pathname: usize) -> SyscallResult {
    let task = current_task();
    let mut buf = [0u8; 256];
    crate::syscall::read_user_cstring(task.ttbr0, pathname, &mut buf)?;
    Ok(0)
}

/// TEAM_406: sys_chmod - No-op for single-user OS (per design decision Q6).
/// TEAM_421: Updated to return SyscallResult.
pub fn sys_chmod(pathname: usize, _mode: u32) -> SyscallResult {
    validate_user_pathname(pathname)
}

/// TEAM_406: sys_fchmod - No-op for single-user OS.
/// TEAM_421: Updated to return SyscallResult.
pub fn sys_fchmod(fd: usize, _mode: u32) -> SyscallResult {
    if is_valid_fd(fd) { Ok(0) } else { Err(EBADF) }
}

/// TEAM_406: sys_chown - No-op for single-user OS.
/// TEAM_421: Updated to return SyscallResult.
pub fn sys_chown(pathname: usize, _owner: u32, _group: u32) -> SyscallResult {
    validate_user_pathname(pathname)
}

/// TEAM_406: sys_fchown - No-op for single-user OS.
/// TEAM_421: Updated to return SyscallResult.
pub fn sys_fchown(fd: usize, _owner: u32, _group: u32) -> SyscallResult {
    if is_valid_fd(fd) { Ok(0) } else { Err(EBADF) }
}

/// TEAM_406: sys_fchmodat - No-op for single-user OS.
/// TEAM_421: Updated to return SyscallResult.
pub fn sys_fchmodat(_dirfd: i32, pathname: usize, _mode: u32, _flags: i32) -> SyscallResult {
    validate_user_pathname(pathname)
}

/// TEAM_406: sys_fchownat - No-op for single-user OS.
/// TEAM_421: Updated to return SyscallResult.
pub fn sys_fchownat(_dirfd: i32, pathname: usize, _owner: u32, _group: u32, _flags: i32) -> SyscallResult {
    validate_user_pathname(pathname)
}
