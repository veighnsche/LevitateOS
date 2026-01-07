//! TEAM_162: x86_64 Context Stub
//! TEAM_258: Added compatible fields for shared code

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Context {
    // x86_64 context fields (named for AArch64 compatibility)
    pub x19: u64,       // rbx equivalent
    pub x20: u64,       // r12 equivalent
    pub x21: u64,       // r13 equivalent
    pub x22: u64,       // r14 equivalent
    pub x23: u64,       // r15 equivalent
    pub x24: u64,       // rbp equivalent
    pub x25: u64,       // spare
    pub x26: u64,       // spare
    pub x27: u64,       // spare
    pub x28: u64,       // spare
    pub x29: u64,       // spare
    pub lr: u64,        // Return address (RIP)
    pub sp: u64,        // Stack Pointer (RSP)
    pub tpidr_el0: u64, // Thread Local Storage (FS base on x86_64)
}

impl Context {
    pub fn new(stack_top: usize, _entry_wrapper: usize) -> Self {
        Self {
            sp: stack_top as u64,
            lr: task_entry_trampoline as *const () as u64,
            x19: _entry_wrapper as u64,
            ..Default::default()
        }
    }

    // TEAM_258: Abstract TLS setting for architecture independence
    // On x86_64 this will eventually set FS base via MSR
    pub fn set_tls(&mut self, addr: u64) {
        self.tpidr_el0 = addr;
    }
}

pub unsafe fn enter_user_mode(_entry_point: usize, _user_sp: usize) -> ! {
    unimplemented!("x86_64 enter_user_mode")
}

// Stubs for asm globals
unsafe extern "C" {
    pub fn cpu_switch_to(old: *mut Context, new: *const Context);
    pub fn task_entry_trampoline();
}

// Global asm stub if needed, but for now we rely on panic
#[unsafe(no_mangle)]
pub unsafe extern "C" fn x86_cpu_switch_to_stub() {
    unimplemented!("cpu_switch_to");
}
