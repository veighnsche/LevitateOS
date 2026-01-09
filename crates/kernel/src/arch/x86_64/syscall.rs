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

/// Initialize syscall/sysret MSRs
pub unsafe fn init() {
    // TEAM_293: STAR MSR format: [63:48]=SYSRET base, [47:32]=SYSCALL base
    // SYSRET: User CS = [63:48]+16|3, User SS = [63:48]+8|3
    // We want: User CS = 0x23 (0x20|3), User SS = 0x1B (0x18|3)
    // So [63:48] = 0x10: CS = 0x10+16|3 = 0x23, SS = 0x10+8|3 = 0x1B ✓
    let star = (0x10_u64 << 48) | (GDT_KERNEL_CODE << 32);
    let lstar = syscall_entry as *const () as usize as u64;
    let fmask = RFLAGS_IF | RFLAGS_TF | RFLAGS_DF;

    unsafe {
        wrmsr(IA32_STAR, star);
        wrmsr(IA32_LSTAR, lstar);
        wrmsr(IA32_FMASK, fmask);
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | EFER_SCE);
    }

    log::info!(
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
        "swapgs",                            // Swap user GS with kernel PCR
        "mov gs:[8], rsp",                   // Save user RSP to PCR.user_rsp_scratch
        "mov rsp, gs:[16]",                  // Load kernel stack from PCR.kernel_stack
        "and rsp, -16",                      // TEAM_299: Ensure 16-byte alignment

        // 2. Build SyscallFrame (Total size 52 qwords = 416 bytes)
        // Order must match SyscallFrame struct in mod.rs

        // Pushes for regs[31] (indexes 21 to 51)
        "sub rsp, 31*8",

        "push 0",                            // _padding (index 20)
        "push 0",                            // pstate (index 19)
        "push qword ptr gs:[8]",             // sp (index 18)
        "push rcx",                          // pc (index 17)
        "push 0",                            // ttbr0 (index 16)
        "push qword ptr gs:[8]",             // rsp (index 15)
        "push r15",                          // r15 (index 14)
        "push r14",
        "push r13",
        "push r12",
        "push rbp",
        "push rbx",
        "push r11",                          // user rflags (index 8)
        "push rcx",                          // user pc (index 7)
        "push r9",                           // arg5 (index 6)
        "push r8",                           // arg4 (index 5)
        "push r10",                          // arg3 (index 4)
        "push rdx",                          // arg2 (index 3)
        "push rsi",                          // arg1 (index 2)
        "push rdi",                          // arg0 (index 1)
        "push rax",                          // syscall_nr (index 0)

        // RDI = pointer to SyscallFrame
        "mov rdi, rsp",

        // Call Rust handler
        "call {handler}",

        // TEAM_299: Best Practice - Sanitize return address (RCX) and RFLAGS (R11) in frame
        // Do this before popping any registers to avoid RAX corruption.
        "mov rax, [rsp + 7*8]",              // Load frame.rcx (Index 7)
        "test rax, rax",
        "jz 3f",                             // RIP=0 is illegal
        "mov rdx, rax",                      // Check canonical
        "shl rdx, 16",
        "sar rdx, 16",
        "cmp rdx, rax",
        "jne 3f",                            // Non-canonical is illegal

        // Sanitize RFLAGS (Index 8)
        "mov rax, [rsp + 8*8]",              // Load frame.r11
        "and rax, 0x3C7FD7",                 // Mask restricted bits
        "or rax, 0x202",                     // Force IF=1, bit1=1
        "mov [rsp + 8*8], rax",              // Store back sanitized RFLAGS

        // 3. Restore registers
        "pop rax",                           // Restore syscall return value (RAX)
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop r10",
        "pop r8",
        "pop r9",
        "pop rcx",                           // Restore RCX (PC) - already checked
        "pop r11",                           // Restore R11 (RFLAGS) - already sanitized
        "pop rbx",
        "pop rbp",
        "pop r12",
        "pop r13",
        "pop r14",
        "pop r15",

        // TEAM_299: Best Practice - Disable interrupts before restoring User RSP
        "cli",

        // After popping R15, RSP points to 'frame.rsp' (index 15)
        // We need to restore User RSP from here
        "mov rsp, [rsp]",

        "swapgs",                            // Restore user GS
        "sysretq",                           // Return to Ring 3

        "3:",                                // Emergency exit for invalid RIP
        "ud2",                               // Panic via invalid opcode

        handler = sym syscall_handler,
    );
}

/// Rust syscall handler - called from assembly
#[unsafe(no_mangle)]
pub extern "C" fn syscall_handler(frame: &mut super::SyscallFrame) {
    // TEAM_297 BREADCRUMB: INVESTIGATING - Debug trace added but no output seen.
    // Suspicion: los_hal::println! might fail in syscall context or execution doesn't reach here.
    let _pc_before = frame.rcx;
    let _nr = frame.rax;

    // Print entry for all syscalls on x86_64 for debugging
    #[cfg(feature = "verbose-syscalls")]
    {
        let task = crate::task::current_task();
        let pid = task.id.0;
        los_hal::println!("[SYSCALL][{}] ENTER nr={} rcx={:x}", pid, _nr, _pc_before,);
    }

    crate::syscall::syscall_dispatch(frame);

    // Print exit for all syscalls on x86_64 for debugging
    #[cfg(feature = "verbose-syscalls")]
    {
        let pid = crate::task::current_task().id.0;
        log::info!("[SYSCALL][{}] EXIT nr={} rax={:x}", pid, _nr, frame.rax);
    }
}
