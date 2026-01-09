//! TEAM_233: File descriptor duplication syscalls.

use crate::syscall::{errno, errno_file};
use crate::task::current_task;
use crate::task::fd_table::FdType;

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
            None => return errno_file::EMFILE,
        };

        let write_fd = match fd_table.alloc(FdType::PipeWrite(pipe.clone())) {
            Some(fd) => fd,
            None => {
                // Clean up read fd
                fd_table.close(read_fd);
                return errno_file::EMFILE;
            }
        };

        (read_fd, write_fd)
    };

    // Write fds to user space
    if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, pipefd_ptr) {
        unsafe {
            let fds = ptr as *mut [i32; 2];
            (*fds)[0] = read_fd as i32;
            (*fds)[1] = write_fd as i32;
        }
    } else {
        // Should not happen after validate_user_buffer, but handle it
        let mut fd_table = task.fd_table.lock();
        fd_table.close(read_fd);
        fd_table.close(write_fd);
        return errno::EFAULT;
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
