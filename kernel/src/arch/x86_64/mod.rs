//! TEAM_162: x86_64 Architecture Stub
//!
//! This module provides stubs for x86_64 to verify the architecture abstraction.

pub mod boot;
pub mod cpu;
pub mod exceptions;
pub mod power;
pub mod task;
pub mod time;

// Re-export Context and other items from task
pub use self::boot::*;
pub use self::exceptions::*;
pub use self::task::*;

pub const ELF_MACHINE: u16 = 62; // EM_X86_64

#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallNumber {
    // x86_64 syscalls will go here
    Read = 0,
    Write = 1,
    Exit = 60,
    // Add others as stubs
}

impl SyscallNumber {
    pub fn from_u64(n: u64) -> Option<Self> {
        match n {
            0 => Some(Self::Read),
            1 => Some(Self::Write),
            60 => Some(Self::Exit),
            _ => None,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Stat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_nlink: u64,
    pub st_mode: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub __pad0: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atime: i64,
    pub st_atime_nsec: u64,
    pub st_mtime: i64,
    pub st_mtime_nsec: u64,
    pub st_ctime: i64,
    pub st_ctime_nsec: u64,
    pub __unused: [i64; 3],
}

#[inline]
pub fn is_svc_exception(_esr: u64) -> bool {
    false
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

/// TEAM_247: Number of control characters in termios.
pub const NCCS: usize = 32;

/// x86_64 Termios stub
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Termios {
    pub c_iflag: u32,
    pub c_oflag: u32,
    pub c_cflag: u32,
    pub c_lflag: u32,
    pub c_line: u8,
    pub c_cc: [u8; NCCS],
    pub c_ispeed: u32,
    pub c_ospeed: u32,
}

impl Termios {
    pub const INITIAL_TERMIOS: Termios = Termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_line: 0,
        c_cc: [0u8; NCCS],
        c_ispeed: 0,
        c_ospeed: 0,
    };
}

// TEAM_162: Stubs for types that need to be provided by the architecture
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SyscallFrame {
    pub regs: [u64; 31],
    pub sp: u64,
}

impl SyscallFrame {
    pub fn syscall_number(&self) -> u64 {
        0
    }
    pub fn arg0(&self) -> u64 {
        0
    }
    pub fn arg1(&self) -> u64 {
        0
    }
    pub fn arg2(&self) -> u64 {
        0
    }
    pub fn arg3(&self) -> u64 {
        0
    }
    pub fn arg4(&self) -> u64 {
        0
    }
    pub fn arg5(&self) -> u64 {
        0
    }
    pub fn arg6(&self) -> u64 {
        0
    }
    pub fn set_return(&mut self, _value: i64) {}
}

pub unsafe fn switch_mmu_config(_config_phys: usize) {
    // unimplemented!("x86_64 switch_mmu_config")
}
