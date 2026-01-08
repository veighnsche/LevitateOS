//! TEAM_277: x86_64 Syscall Entry/Exit
//!
//! Implements the syscall infrastructure using the SYSCALL/SYSRET instructions.
//!
//! x86_64 syscall convention:
//! - rax: syscall number (input), return value (output)
//! - rdi, rsi, rdx, r10, r8, r9: arguments 1-6
//! - rcx: saved RIP (clobbered)
//! - r11: saved RFLAGS (clobbered)

use core::arch::{asm, naked_asm};

/// GDT segment selectors
pub const GDT_KERNEL_CODE: u64 = 0x08;

/// MSR addresses
const IA32_STAR: u32 = 0xC000_0081;
const IA32_LSTAR: u32 = 0xC000_0082;
const IA32_FMASK: u32 = 0xC000_0084;
const IA32_EFER: u32 = 0xC000_0080;

/// EFER flags
const EFER_SCE: u64 = 1 << 0;

/// RFLAGS bits to clear on syscall entry
const RFLAGS_IF: u64 = 1 << 9;
const RFLAGS_TF: u64 = 1 << 8;
const RFLAGS_DF: u64 = 1 << 10;

/// Static kernel stack for syscalls (16KB)
#[repr(C, align(16))]
pub struct SyscallStack {
    pub data: [u8; 16384],
}

#[unsafe(no_mangle)]
pub static mut SYSCALL_STACK: SyscallStack = SyscallStack { data: [0; 16384] };

/// Initialize syscall/sysret MSRs
pub unsafe fn init() {
    // TEAM_293: STAR MSR format: [63:48]=SYSRET base, [47:32]=SYSCALL base
    // SYSRET: User CS = [63:48]+16|3, User SS = [63:48]+8|3
    // We want: User CS = 0x23 (0x20|3), User SS = 0x1B (0x18|3)
    // So [63:48] = 0x10: CS = 0x10+16|3 = 0x23, SS = 0x10+8|3 = 0x1B ✓
    let star = (0x10_u64 << 48) | (GDT_KERNEL_CODE << 32);
    let lstar = syscall_entry as usize as u64;
    let fmask = RFLAGS_IF | RFLAGS_TF | RFLAGS_DF;

    unsafe {
        wrmsr(IA32_STAR, star);
        wrmsr(IA32_LSTAR, lstar);
        wrmsr(IA32_FMASK, fmask);
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | EFER_SCE);
    }

    los_hal::println!(
        "[SYSCALL] x86_64 syscall MSRs initialized, LSTAR=0x{:x}",
        lstar
    );
}

#[inline(always)]
unsafe fn rdmsr(msr: u32) -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") lo,
            out("edx") hi,
            options(nostack, nomem)
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

#[inline(always)]
unsafe fn wrmsr(msr: u32, value: u64) {
    let lo = value as u32;
    let hi = (value >> 32) as u32;
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") lo,
            in("edx") hi,
            options(nostack, nomem)
        );
    }
}

/// Syscall entry point
#[unsafe(naked)]
pub unsafe extern "C" fn syscall_entry() {
    naked_asm!(
        // Save userspace RSP to R15
        "mov r15, rsp",

        // Load kernel stack
        "lea rsp, [rip + {stack} + 16384]",

        // Build SyscallFrame on stack
        "sub rsp, 31*8",   // regs[31] placeholder
        "push 0",          // pstate
        "push r15",        // sp
        "push rcx",        // pc (return address)
        "push 0",          // ttbr0
        "push r15",        // rsp
        "push r15",        // r15
        "push r14",
        "push r13",
        "push r12",
        "push rbp",
        "push rbx",
        "push r11",        // RFLAGS
        "push rcx",        // return address
        "push r9",
        "push r8",
        "push r10",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rax",

        // RDI = pointer to SyscallFrame
        "mov rdi, rsp",

        // Call Rust handler
        "call {handler}",

        // Restore registers
        "pop rax",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop r10",
        "pop r8",
        "pop r9",
        "pop rcx",
        "pop r11",
        "pop rbx",
        "pop rbp",
        "pop r12",
        "pop r13",
        "pop r14",
        "pop r15",
        "add rsp, 8*5",    // skip rsp, ttbr0, pc, sp, pstate
        "add rsp, 31*8",   // skip regs[31]

        // Restore user stack and return
        "mov rsp, r15",
        "sysretq",

        stack = sym SYSCALL_STACK,
        handler = sym syscall_handler,
    );
}

/// Rust syscall handler - called from assembly
#[unsafe(no_mangle)]
pub extern "C" fn syscall_handler(frame: &mut super::SyscallFrame) {
    crate::syscall::syscall_dispatch(frame);
}
