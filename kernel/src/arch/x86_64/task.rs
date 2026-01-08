//! TEAM_162: x86_64 Context Stub
//! TEAM_258: Added compatible fields for shared code
//! TEAM_277: Added stub implementations for task switching

use core::arch::global_asm;

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
    pub fn new(stack_top: usize, entry_wrapper: usize) -> Self {
        Self {
            sp: stack_top as u64,
            lr: task_entry_trampoline as usize as u64,
            x19: entry_wrapper as u64,
            ..Default::default()
        }
    }

    pub fn set_tls(&mut self, addr: u64) {
        self.tpidr_el0 = addr;
    }
}

/// TEAM_293: Enter Ring 3 userspace using sysretq.
/// RCX = entry point (becomes RIP after sysretq)
/// R11 = RFLAGS (should have IF=1 for interrupts)
/// RSP = user stack pointer
pub unsafe fn enter_user_mode(entry_point: usize, user_sp: usize) -> ! {
    unsafe {
        core::arch::asm!(
            "mov rsp, {sp}",        // Set user stack
            "mov rcx, {entry}",     // sysretq: RCX -> RIP
            "mov r11, 0x202",       // sysretq: R11 -> RFLAGS (IF=1, bit1=1 reserved)
            "sysretq",
            sp = in(reg) user_sp,
            entry = in(reg) entry_point,
            options(noreturn)
        );
    }
}

// TEAM_277: External references that will be defined in global_asm below
unsafe extern "C" {
    pub fn cpu_switch_to(old: *mut Context, new: *const Context);
}

// TEAM_293: Task entry trampoline - called after context switch to new task
// Entry wrapper address is in rbx (restored from context.x19)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn task_entry_trampoline() {
    // Get entry wrapper from rbx (was context.x19)
    let entry_wrapper: extern "C" fn();
    unsafe {
        core::arch::asm!("mov {}, rbx", out(reg) entry_wrapper);
    }

    // Call the entry wrapper (which will call enter_user_mode or run kernel task)
    entry_wrapper();

    // If the entry wrapper returns, exit the task
    crate::task::task_exit();
}

// TEAM_277: Context switch stub
// Saves callee-saved registers to old context, restores from new context
global_asm!(
    ".global cpu_switch_to",
    "cpu_switch_to:",
    // Save callee-saved registers to old context (rdi points to old Context)
    "mov [rdi + 0], rbx",  // x19 = rbx
    "mov [rdi + 8], r12",  // x20 = r12
    "mov [rdi + 16], r13", // x21 = r13
    "mov [rdi + 24], r14", // x22 = r14
    "mov [rdi + 32], r15", // x23 = r15
    "mov [rdi + 40], rbp", // x24 = rbp
    "mov [rdi + 88], rsp", // sp
    "lea rax, [rip + 1f]", // Get return address
    "mov [rdi + 80], rax", // lr = return address
    // Restore callee-saved registers from new context (rsi points to new Context)
    "mov rbx, [rsi + 0]",
    "mov r12, [rsi + 8]",
    "mov r13, [rsi + 16]",
    "mov r14, [rsi + 24]",
    "mov r15, [rsi + 32]",
    "mov rbp, [rsi + 40]",
    "mov rsp, [rsi + 88]",
    "mov rax, [rsi + 80]", // lr = new return address
    "jmp rax",             // Jump to new task
    "1:",                  // Return point for context switch back
    "ret"
);
