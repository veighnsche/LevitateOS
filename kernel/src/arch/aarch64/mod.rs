pub mod boot;
pub mod exceptions;
pub mod task;

pub use self::boot::*;
pub use self::task::*;

use crate::arch::EarlyConsole;

/// TEAM_162: Saved user context during syscall.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SyscallFrame {
    pub regs: [u64; 31],
    pub sp: u64,
    pub pc: u64,
    pub pstate: u64,
    pub ttbr0: u64,
}

impl SyscallFrame {
    pub fn syscall_number(&self) -> u64 {
        self.regs[8]
    }
    pub fn arg0(&self) -> u64 {
        self.regs[0]
    }
    pub fn arg1(&self) -> u64 {
        self.regs[1]
    }
    pub fn arg2(&self) -> u64 {
        self.regs[2]
    }
    pub fn arg3(&self) -> u64 {
        self.regs[3]
    }
    pub fn arg4(&self) -> u64 {
        self.regs[4]
    }
    pub fn arg5(&self) -> u64 {
        self.regs[5]
    }
    pub fn set_return(&mut self, value: i64) {
        self.regs[0] = value as u64;
    }
}

pub struct AArch64EarlyConsole;

impl EarlyConsole for AArch64EarlyConsole {
    fn write_str(&self, s: &str) {
        los_hal::print!("{}", s);
    }
}

static EARLY_CONSOLE: AArch64EarlyConsole = AArch64EarlyConsole;

#[unsafe(no_mangle)]
pub fn get_early_console() -> Option<&'static dyn EarlyConsole> {
    Some(&EARLY_CONSOLE)
}
