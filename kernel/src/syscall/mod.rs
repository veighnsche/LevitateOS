//! TEAM_171: Modular System Call Handler for LevitateOS.

pub mod fs;
pub mod mm;
pub mod process;
pub mod sys;
pub mod time;

use crate::arch::SyscallFrame;
use los_hal::println;

/// TEAM_073: Error codes for syscalls.
pub mod errno {
    pub const ENOSYS: i64 = -1;
    pub const EBADF: i64 = -2;
    pub const EFAULT: i64 = -3;
    pub const EINVAL: i64 = -4;
    /// TEAM_192: Read-only file system
    pub const EROFS: i64 = -30;
}

/// TEAM_168: Additional errno values for file operations.
pub mod errno_file {
    /// No such file or directory
    pub const ENOENT: i64 = -5;
    /// Too many open files
    pub const EMFILE: i64 = -6;
    /// TEAM_176: Not a directory
    pub const ENOTDIR: i64 = -7;
}

#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallNumber {
    Read = 0,
    Write = 1,
    Exit = 2,
    GetPid = 3,
    Sbrk = 4,
    Spawn = 5,
    Exec = 6,
    Yield = 7,
    Shutdown = 8,
    Openat = 9,
    Close = 10,
    Fstat = 11,
    Nanosleep = 12,
    ClockGettime = 13,
    /// TEAM_176: Read directory entries
    Getdents = 14,
    /// TEAM_186: Spawn process with arguments
    SpawnArgs = 15,
    /// TEAM_188: Wait for child process
    Waitpid = 16,
    /// TEAM_192: Get current working directory
    Getcwd = 17,
    /// TEAM_192: Create directory
    Mkdirat = 34,
    /// TEAM_192: Remove file or directory
    Unlinkat = 35,
    /// TEAM_192: Rename/move file or directory
    Renameat = 38,
}

impl SyscallNumber {
    pub fn from_u64(n: u64) -> Option<Self> {
        match n {
            0 => Some(Self::Read),
            1 => Some(Self::Write),
            2 => Some(Self::Exit),
            3 => Some(Self::GetPid),
            4 => Some(Self::Sbrk),
            5 => Some(Self::Spawn),
            6 => Some(Self::Exec),
            7 => Some(Self::Yield),
            8 => Some(Self::Shutdown),
            9 => Some(Self::Openat),
            10 => Some(Self::Close),
            11 => Some(Self::Fstat),
            12 => Some(Self::Nanosleep),
            13 => Some(Self::ClockGettime),
            14 => Some(Self::Getdents),
            15 => Some(Self::SpawnArgs),
            16 => Some(Self::Waitpid),
            17 => Some(Self::Getcwd),
            34 => Some(Self::Mkdirat),
            35 => Some(Self::Unlinkat),
            38 => Some(Self::Renameat),
            _ => None,
        }
    }
}

/// TEAM_168: Stat structure returned by fstat.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Stat {
    pub st_size: u64,
    pub st_mode: u32,
    pub _pad: u32,
}

/// TEAM_170: Timespec structure for clock_gettime.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Timespec {
    pub tv_sec: u64,
    pub tv_nsec: u64,
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
        Some(SyscallNumber::Openat) => fs::sys_openat(
            frame.arg0() as usize,
            frame.arg1() as usize,
            frame.arg2() as u32,
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
        Some(SyscallNumber::Unlinkat) => fs::sys_unlinkat(
            frame.arg0() as i32,
            frame.arg1() as usize,
            frame.arg2() as usize,
            frame.arg3() as u32,
        ),
        Some(SyscallNumber::Renameat) => fs::sys_renameat(
            frame.arg0() as i32,
            frame.arg1() as usize,
            frame.arg2() as usize,
            frame.arg3() as i32,
            frame.arg4() as usize,
            frame.arg5() as usize,
        ),
        None => {
            println!("[SYSCALL] Unknown syscall number: {}", nr);
            errno::ENOSYS
        }
    };

    frame.set_return(result);
}

pub const EC_SVC_AARCH64: u64 = 0b010101;

#[inline]
pub fn esr_exception_class(esr: u64) -> u64 {
    (esr >> 26) & 0x3F
}

#[inline]
pub fn is_svc_exception(esr: u64) -> bool {
    esr_exception_class(esr) == EC_SVC_AARCH64
}

pub(crate) fn write_to_user_buf(
    ttbr0: usize,
    user_buf_base: usize,
    offset: usize,
    byte: u8,
) -> bool {
    let user_va = user_buf_base + offset;
    if let Some(kernel_ptr) = crate::task::user_mm::user_va_to_kernel_ptr(ttbr0, user_va) {
        unsafe {
            *kernel_ptr = byte;
        }
        true
    } else {
        false
    }
}
