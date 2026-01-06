pub mod boot;
pub mod exceptions;
pub mod task;

pub use self::boot::*;
pub use self::task::*;

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
    // TEAM_163: Part of complete syscall ABI (supports up to 6 args per docs)
    #[allow(dead_code)]
    pub fn arg3(&self) -> u64 {
        self.regs[3]
    }
    #[allow(dead_code)]
    pub fn arg4(&self) -> u64 {
        self.regs[4]
    }
    #[allow(dead_code)]
    pub fn arg5(&self) -> u64 {
        self.regs[5]
    }
    #[allow(dead_code)]
    pub fn arg6(&self) -> u64 {
        self.regs[6]
    }
    pub fn set_return(&mut self, value: i64) {
        self.regs[0] = value as u64;
    }
}
// TEAM_163: Removed dead AArch64EarlyConsole (Rule 6: No Dead Code)
