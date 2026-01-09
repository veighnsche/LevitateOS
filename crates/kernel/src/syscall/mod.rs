use crate::memory::user as mm_user;

pub mod fs;
pub mod mm;
pub mod process;
pub mod signal;
pub mod sync;
pub mod sys;
pub mod time;

pub use crate::arch::{Stat, SyscallFrame, SyscallNumber, Timespec, is_svc_exception};

/// TEAM_345: Linux file system constants for *at() syscalls.
pub mod fcntl {
    /// Special value for dirfd meaning "use current working directory"
    pub const AT_FDCWD: i32 = -100;
    /// Don't follow symbolic links
    pub const AT_SYMLINK_NOFOLLOW: u32 = 0x100;
    /// Remove directory instead of file
    pub const AT_REMOVEDIR: u32 = 0x200;
    /// Follow symbolic links (for linkat)
    pub const AT_SYMLINK_FOLLOW: u32 = 0x400;
    /// Suppress terminal automount traversal
    pub const AT_NO_AUTOMOUNT: u32 = 0x800;
    /// Allow empty relative pathname
    pub const AT_EMPTY_PATH: u32 = 0x1000;
}

/// TEAM_073: Error codes for syscalls.
/// TEAM_342: Consolidated errno constants - single source of truth.
pub mod errno {
    pub const ENOENT: i64 = -2;
    pub const EIO: i64 = -5;
    pub const EBADF: i64 = -9;
    pub const ENOMEM: i64 = -12;
    pub const EACCES: i64 = -13;
    pub const EFAULT: i64 = -14;
    pub const EEXIST: i64 = -17;
    pub const EXDEV: i64 = -18;      // Cross-device link
    pub const ENOTDIR: i64 = -20;
    pub const EINVAL: i64 = -22;
    pub const EMFILE: i64 = -24;
    pub const ENOTTY: i64 = -25;
    pub const ERANGE: i64 = -34;     // Result too large
    pub const ENAMETOOLONG: i64 = -36;
    pub const ENOSYS: i64 = -38;
    pub const ENOTEMPTY: i64 = -39;  // Directory not empty
}

/// TEAM_342: Deprecated - use errno module instead. Kept for backward compatibility.
#[deprecated(note = "Use errno module instead")]
pub mod errno_file {
    pub use super::errno::*;
}

pub fn syscall_dispatch(frame: &mut SyscallFrame) {
    let nr = frame.syscall_number();
    let result = match SyscallNumber::from_u64(nr) {
        Some(SyscallNumber::Read) => fs::sys_read(
            frame.arg0() as usize,
            frame.arg1() as usize,
            frame.arg2() as usize,
        ),
        Some(SyscallNumber::Write) => fs::sys_write(
            frame.arg0() as usize,
            frame.arg1() as usize,
            frame.arg2() as usize,
        ),
        Some(SyscallNumber::Exit) => process::sys_exit(frame.arg0() as i32),
        Some(SyscallNumber::GetPid) => process::sys_getpid(),
        Some(SyscallNumber::Sbrk) => mm::sys_sbrk(frame.arg0() as isize),
        Some(SyscallNumber::Spawn) => {
            process::sys_spawn(frame.arg0() as usize, frame.arg1() as usize)
        }
        Some(SyscallNumber::Exec) => {
            process::sys_exec(frame.arg0() as usize, frame.arg1() as usize)
        }
        Some(SyscallNumber::Yield) => process::sys_yield(),
        Some(SyscallNumber::Shutdown) => sys::sys_shutdown(frame.arg0() as u32),
        // TEAM_345: Linux ABI - openat(dirfd, pathname, flags, mode)
        Some(SyscallNumber::Openat) => fs::sys_openat(
            frame.arg0() as i32,
            frame.arg1() as usize,
            frame.arg2() as u32,
            frame.arg3() as u32,
        ),
        Some(SyscallNumber::Close) => fs::sys_close(frame.arg0() as usize),
        Some(SyscallNumber::Fstat) => fs::sys_fstat(frame.arg0() as usize, frame.arg1() as usize),
        Some(SyscallNumber::Nanosleep) => {
            time::sys_nanosleep(frame.arg0() as u64, frame.arg1() as u64)
        }
        Some(SyscallNumber::ClockGettime) => time::sys_clock_gettime(frame.arg0() as usize),
        // TEAM_176: Directory listing syscall
        Some(SyscallNumber::Getdents) => fs::sys_getdents(
            frame.arg0() as usize,
            frame.arg1() as usize,
            frame.arg2() as usize,
        ),
        // TEAM_186: Spawn process with arguments
        Some(SyscallNumber::SpawnArgs) => process::sys_spawn_args(
            frame.arg0() as usize,
            frame.arg1() as usize,
            frame.arg2() as usize,
            frame.arg3() as usize,
        ),
        // TEAM_188: Wait for child process
        Some(SyscallNumber::Waitpid) => {
            process::sys_waitpid(frame.arg0() as i32, frame.arg1() as usize)
        }
        Some(SyscallNumber::Getcwd) => fs::sys_getcwd(frame.arg0() as usize, frame.arg1() as usize),
        Some(SyscallNumber::Mkdirat) => fs::sys_mkdirat(
            frame.arg0() as i32,
            frame.arg1() as usize,
            frame.arg2() as usize,
            frame.arg3() as u32,
        ),
        // TEAM_345: Linux ABI - unlinkat(dirfd, pathname, flags)
        Some(SyscallNumber::Unlinkat) => fs::sys_unlinkat(
            frame.arg0() as i32,
            frame.arg1() as usize,
            frame.arg2() as u32,
        ),
        Some(SyscallNumber::Renameat) => fs::sys_renameat(
            frame.arg0() as i32,
            frame.arg1() as usize,
            frame.arg2() as usize,
            frame.arg3() as i32,
            frame.arg4() as usize,
            frame.arg5() as usize,
        ),
        // TEAM_345: Linux ABI - utimensat(dirfd, pathname, times, flags)
        Some(SyscallNumber::Utimensat) => fs::sys_utimensat(
            frame.arg0() as i32,
            frame.arg1() as usize,
            frame.arg2() as usize,
            frame.arg3() as u32,
        ),
        // TEAM_345: Linux ABI - symlinkat(target, newdirfd, linkpath)
        Some(SyscallNumber::Symlinkat) => fs::sys_symlinkat(
            frame.arg0() as usize,
            frame.arg1() as i32,
            frame.arg2() as usize,
        ),
        // TEAM_345: Linux ABI - readlinkat(dirfd, pathname, buf, bufsiz)
        Some(SyscallNumber::Readlinkat) => fs::sys_readlinkat(
            frame.arg0() as i32,
            frame.arg1() as usize,
            frame.arg2() as usize,
            frame.arg3() as usize,
        ),
        // TEAM_206: Mount/Umount
        Some(SyscallNumber::Mount) => fs::sys_mount(
            frame.arg0() as usize,
            frame.arg1() as usize,
            frame.arg2() as usize,
            frame.arg3() as usize,
            frame.arg4() as usize,
        ),
        Some(SyscallNumber::Umount) => fs::sys_umount(frame.arg0() as usize, frame.arg1() as usize),
        // TEAM_208: Futex syscall
        Some(SyscallNumber::Futex) => {
            let addr = frame.arg0() as usize;
            let op = frame.arg1() as usize;
            let val = frame.arg2() as usize;
            let timeout = frame.arg3() as usize;
            let addr2 = frame.arg4() as usize;
            crate::syscall::sync::sys_futex(addr, op, val, timeout, addr2)
        }
        Some(SyscallNumber::GetPpid) => process::sys_getppid(),
        Some(SyscallNumber::Writev) => fs::sys_writev(
            frame.arg0() as usize,
            frame.arg1() as usize,
            frame.arg2() as usize,
        ),
        Some(SyscallNumber::Readv) => fs::sys_readv(
            frame.arg0() as usize,
            frame.arg1() as usize,
            frame.arg2() as usize,
        ),
        // TEAM_345: Linux ABI - linkat(olddirfd, oldpath, newdirfd, newpath, flags)
        Some(SyscallNumber::Linkat) => fs::sys_linkat(
            frame.arg0() as i32,
            frame.arg1() as usize,
            frame.arg2() as i32,
            frame.arg3() as usize,
            frame.arg4() as u32,
        ),
        // TEAM_216: Signal Handling syscalls
        Some(SyscallNumber::Kill) => signal::sys_kill(frame.arg0() as i32, frame.arg1() as i32),
        Some(SyscallNumber::Pause) => signal::sys_pause(),
        Some(SyscallNumber::SigAction) => signal::sys_sigaction(
            frame.arg0() as i32,
            frame.arg1() as usize,
            frame.arg2() as usize,
        ),
        Some(SyscallNumber::SigReturn) => signal::sys_sigreturn(frame),
        Some(SyscallNumber::SigProcMask) => signal::sys_sigprocmask(
            frame.arg0() as i32,
            frame.arg1() as usize,
            frame.arg2() as usize,
        ),
        Some(SyscallNumber::SetForeground) => process::sys_set_foreground(frame.arg0() as usize),
        Some(SyscallNumber::GetForeground) => process::sys_get_foreground(),
        Some(SyscallNumber::Isatty) => fs::sys_isatty(frame.arg0() as i32),
        // TEAM_228: Memory management syscalls
        Some(SyscallNumber::Mmap) => mm::sys_mmap(
            frame.arg0() as usize,
            frame.arg1() as usize,
            frame.arg2() as u32,
            frame.arg3() as u32,
            frame.arg4() as i32,
            frame.arg5() as usize,
        ),
        Some(SyscallNumber::Munmap) => mm::sys_munmap(frame.arg0() as usize, frame.arg1() as usize),
        Some(SyscallNumber::Mprotect) => mm::sys_mprotect(
            frame.arg0() as usize,
            frame.arg1() as usize,
            frame.arg2() as u32,
        ),
        // TEAM_228: Threading syscalls
        Some(SyscallNumber::Clone) => process::sys_clone(
            frame.arg0() as u64,
            frame.arg1() as usize,
            frame.arg2() as usize,
            frame.arg3() as usize,
            frame.arg4() as usize,
            frame, // TEAM_230: Pass frame to clone registers
        ),
        Some(SyscallNumber::SetTidAddress) => process::sys_set_tid_address(frame.arg0() as usize),
        // TEAM_233: Pipe and dup syscalls
        Some(SyscallNumber::Dup) => fs::sys_dup(frame.arg0() as usize),
        Some(SyscallNumber::Dup3) => fs::sys_dup3(
            frame.arg0() as usize,
            frame.arg1() as usize,
            frame.arg2() as u32,
        ),
        Some(SyscallNumber::Pipe2) => fs::sys_pipe2(frame.arg0() as usize, frame.arg1() as u32),
        Some(SyscallNumber::Ioctl) => fs::sys_ioctl(
            frame.arg0() as usize,
            frame.arg1() as u64,
            frame.arg2() as usize,
        ),
        None => {
            log::warn!("[SYSCALL] Unknown syscall number: {}", nr);
            errno::ENOSYS
        }
    };

    frame.set_return(result);
}

pub(crate) fn write_to_user_buf(
    ttbr0: usize,
    user_buf_base: usize,
    offset: usize,
    byte: u8,
) -> bool {
    let user_va = user_buf_base + offset;
    if let Some(kernel_ptr) = mm_user::user_va_to_kernel_ptr(ttbr0, user_va) {
        // SAFETY: user_va_to_kernel_ptr ensures the address is mapped and valid.
        unsafe {
            *kernel_ptr = byte;
        }
        true
    } else {
        false
    }
}

pub(crate) fn read_from_user(ttbr0: usize, user_va: usize) -> Option<u8> {
    if let Some(kernel_ptr) = mm_user::user_va_to_kernel_ptr(ttbr0, user_va) {
        // SAFETY: user_va_to_kernel_ptr ensures the address is mapped and valid.
        Some(unsafe { *kernel_ptr })
    } else {
        None
    }
}

/// TEAM_226: Copy a string from user space into a kernel buffer.
///
/// Validates the user buffer and copies bytes through kernel-accessible pointers.
/// This is the safe pattern for reading user memory from syscalls.
///
/// # Arguments
/// * `ttbr0` - User page table physical address
/// * `user_ptr` - User virtual address of string
/// * `len` - Length of string to copy
/// * `buf` - Kernel buffer to copy into
///
/// # Returns
/// * `Ok(&str)` - Valid UTF-8 string slice from buffer
/// * `Err(errno)` - EFAULT if copy fails, EINVAL if not valid UTF-8
pub fn copy_user_string<'a>(
    ttbr0: usize,
    user_ptr: usize,
    len: usize,
    buf: &'a mut [u8],
) -> Result<&'a str, i64> {
    let len = len.min(buf.len());
    if mm_user::validate_user_buffer(ttbr0, user_ptr, len, false).is_err() {
        return Err(errno::EFAULT);
    }
    for i in 0..len {
        if let Some(ptr) = mm_user::user_va_to_kernel_ptr(ttbr0, user_ptr + i) {
            // SAFETY: user_va_to_kernel_ptr ensures the address is mapped and valid.
            buf[i] = unsafe { *ptr };
        } else {
            return Err(errno::EFAULT);
        }
    }
    core::str::from_utf8(&buf[..len]).map_err(|_| errno::EINVAL)
}

/// TEAM_345: Read a null-terminated C string from user space into a kernel buffer.
///
/// This is the Linux ABI-compatible version that scans for null terminator.
/// Used for syscalls that accept `const char *pathname` arguments.
///
/// # Arguments
/// * `ttbr0` - User page table physical address
/// * `user_ptr` - User virtual address of null-terminated string
/// * `buf` - Kernel buffer to copy into (max path length)
///
/// # Returns
/// * `Ok(&str)` - Valid UTF-8 string slice from buffer (without null terminator)
/// * `Err(errno)` - EFAULT if copy fails, EINVAL if not valid UTF-8, ENAMETOOLONG if no null found
pub fn read_user_cstring<'a>(
    ttbr0: usize,
    user_ptr: usize,
    buf: &'a mut [u8],
) -> Result<&'a str, i64> {
    for i in 0..buf.len() {
        match mm_user::user_va_to_kernel_ptr(ttbr0, user_ptr + i) {
            Some(ptr) => {
                // SAFETY: user_va_to_kernel_ptr ensures the address is mapped and valid.
                let byte = unsafe { *ptr };
                if byte == 0 {
                    // Found null terminator - return the string up to this point
                    return core::str::from_utf8(&buf[..i]).map_err(|_| errno::EINVAL);
                }
                buf[i] = byte;
            }
            None => return Err(errno::EFAULT),
        }
    }
    // Buffer full without finding null terminator
    Err(errno::ENAMETOOLONG)
}
