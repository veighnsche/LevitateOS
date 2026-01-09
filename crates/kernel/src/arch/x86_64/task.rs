//! TEAM_162: x86_64 Context Stub
//! TEAM_258: Added compatible fields for shared code
//! TEAM_277: Added stub implementations for task switching
//!
//! Behaviors:
//! - [X86_CTX1-6] cpu_switch_to register save/restore
//! - [X86_CTX7] task_entry_trampoline
//! - [X86_USR1-5] enter_user_mode sysretq transition

use core::arch::global_asm;

/// TEAM_358: FPU/SSE state buffer for FXSAVE/FXRSTOR (512 bytes, 16-byte aligned)
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct FpuState {
    pub data: [u8; 512],
}

impl Default for FpuState {
    fn default() -> Self {
        Self { data: [0u8; 512] }
    }
}

impl core::fmt::Debug for FpuState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FpuState").finish_non_exhaustive()
    }
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, Default)]
pub struct Context {
    // x86_64 context fields (named for AArch64 compatibility where possible)
    pub rbx: u64,        // 0
    pub r12: u64,        // 8
    pub r13: u64,        // 16
    pub r14: u64,        // 24
    pub r15: u64,        // 32
    pub rbp: u64,        // 40
    pub kstack: u64,     // 48: kernel_stack_top
    pub rflags: u64,     // 56: TEAM_299: Save/restore RFLAGS to prevent DF leaks
    pub rip: u64,        // 64: Return address
    pub rsp: u64,        // 72: Stack Pointer
    pub fs_base: u64,    // 80: User TLS (FS base)
    pub user_gs: u64,    // 88: User TLS (GS base - shadow during kernel execution)
    pub spare: [u64; 2], // 96, 104: Padding for 16-byte alignment (total 112 bytes)
    // TEAM_358: FPU/SSE state (offset 112, 512 bytes)
    pub fpu_state: FpuState,
}

impl Context {
    pub fn new(stack_top: usize, entry_wrapper: usize) -> Self {
        Self {
            rsp: stack_top as u64,
            kstack: stack_top as u64,
            rip: task_entry_trampoline as *const () as usize as u64,
            rbx: entry_wrapper as u64,
            rflags: 0x202, // IF=1, bit1=1
            ..Default::default()
        }
    }

    pub fn set_tls(&mut self, addr: u64) {
        self.fs_base = addr;
    }
}

/// TEAM_293: Enter Ring 3 userspace using sysretq.
/// RCX = entry point (becomes RIP after sysretq)
/// R11 = RFLAGS (should have IF=1 for interrupts)
/// RSP = user stack pointer
pub unsafe fn enter_user_mode(entry_point: usize, user_sp: usize) -> ! {
    unsafe {
        core::arch::asm!(
            "swapgs",               // Restore user GS base (or set to 0 initially)
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
// TEAM_358: Added FPU state save/restore with fxsave64/fxrstor64
// Saves callee-saved registers to old context, restores from new context
global_asm!(
    ".global cpu_switch_to",
    "cpu_switch_to:",
    // Save callee-saved registers to old context (rdi points to old Context)
    "mov [rdi + 0], rbx",  // rbx
    "mov [rdi + 8], r12",  // r12
    "mov [rdi + 16], r13", // r13
    "mov [rdi + 24], r14", // r14
    "mov [rdi + 32], r15", // r15
    "mov [rdi + 40], rbp", // rbp
    "pushfq",              // TEAM_299: Save RFLAGS
    "pop rax",
    "mov [rdi + 56], rax", // rflags
    "mov [rdi + 72], rsp", // rsp
    "lea rax, [rip + 1f]", // Get return address
    "mov [rdi + 64], rax", // rip
    // TEAM_299: Save User GS Base (shadow GS)
    "mov ecx, 0xC0000102", // IA32_KERNEL_GS_BASE
    "rdmsr",
    "shl rdx, 32",
    "or rax, rdx",
    "mov [rdi + 88], rax", // user_gs
    // TEAM_358: Save FPU/SSE state (offset 112)
    "fxsave64 [rdi + 112]",
    // Restore callee-saved registers from new context (rsi points to new Context)
    "mov rbx, [rsi + 0]",
    "mov r12, [rsi + 8]",
    "mov r13, [rsi + 16]",
    "mov r14, [rsi + 24]",
    "mov r15, [rsi + 32]",
    "mov rbp, [rsi + 40]",
    "mov rax, [rsi + 56]", // TEAM_299: Restore RFLAGS
    "push rax",
    "popfq",
    "mov rsp, [rsi + 72]",
    "mov rax, [rsi + 48]", // kernel_stack from kstack
    "mov gs:[16], rax",    // Update PCR.kernel_stack (PCR_KSTACK_OFFSET)
    "mov gs:[100], rax",   // Update PCR.tss.rsp0 (PCR_TSS_OFFSET + TSS_RSP0_OFFSET)
    // TEAM_299: Restore TLS (FS_BASE)
    "mov rax, [rsi + 80]", // context.fs_base
    "test rax, rax",
    "jz 2f",
    "mov ecx, 0xC0000100", // IA32_FS_BASE
    "mov rdx, rax",
    "shr rdx, 32",
    "wrmsr",
    "2:",
    // TEAM_299: Restore User GS Base (KERNEL_GS_BASE)
    "mov rax, [rsi + 88]", // context.user_gs
    "mov ecx, 0xC0000102", // IA32_KERNEL_GS_BASE
    "mov rdx, rax",
    "shr rdx, 32",
    "wrmsr",
    // TEAM_358: Restore FPU/SSE state (offset 112)
    "fxrstor64 [rsi + 112]",
    "mov rax, [rsi + 64]", // rip = new return address
    "jmp rax",             // Jump to new task
    "1:",                  // Return point for context switch back
    "ret",
);
