//! TEAM_162: x86_64 Architecture Stub
//!
//! This module provides stubs for x86_64 to verify the architecture abstraction.

use crate::arch::EarlyConsole;

pub struct x86EarlyConsole;

impl EarlyConsole for x86EarlyConsole {
    fn write_str(&self, _s: &str) {
        // unimplemented!("x86_64 early console")
    }
}

static EARLY_CONSOLE: x86EarlyConsole = x86EarlyConsole;

#[no_mangle]
pub fn get_early_console() -> Option<&'static dyn EarlyConsole> {
    Some(&EARLY_CONSOLE)
}

// TEAM_162: Stubs for types that need to be provided by the architecture
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SyscallFrame {
    // x86_64 registers would go here
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Context {
    // x86_64 context would go here
}

pub unsafe fn enter_user_mode(_entry_point: usize, _user_sp: usize) -> ! {
    unimplemented!("x86_64 enter_user_mode")
}

pub unsafe fn switch_mmu_config(_config_phys: usize) {
    // unimplemented!("x86_64 switch_mmu_config")
}
