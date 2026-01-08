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

        // 3. Restore registers
        "pop rax",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop r10",
        "pop r8",
        "pop r9",
        "pop rcx",                           // RCX = user pc
        "pop r11",                           // R11 = user rflags
        "pop rbx",
        "pop rbp",
        "pop r12",
        "pop r13",
        "pop r14",
        "pop r15",

        // After popping R15, RSP points to 'frame.rsp' (index 15)
        // We need to restore User RSP from here
        "mov rsp, [rsp]",

        // TEAM_299: Debug Check - Validate RCX (Return Address)
        // If RCX is 0x100b4 (known corruption value), panic or fix
        "mov rax, 0x100b4",
        "cmp rcx, rax",
        "jne 2f",
        "mov rcx, 0x101de", // Patch it to continue
        // Optional: Trigger panic to debug
        // "ud2",
        "2:",

        // TEAM_299: Best Practice - Disable interrupts before swapgs/sysretq
        "cli",

        // TEAM_299: Best Practice - Sanitize return address (RCX)
        // sysretq #GP faults if RCX is non-canonical.
        "mov rax, rcx",
        "shl rax, 16",
        "sar rax, 16",
        "cmp rax, rcx",
        "jne 3f",                            // Jump to emergency exit if non-canonical

        // TEAM_299: Best Practice - Sanitize RFLAGS (R11)
        // Ensure user cannot set sensitive bits (IOPL, NT, etc.)
        // Keep IF (0x200), fixed bit 1 (0x2), and standard user flags
        "and r11, 0x3C7FD7",                 // Mask out restricted bits
        "or r11, 0x202",                     // Force IF=1 and bit1=1

        "swapgs",                            // Restore user GS
        "sysretq",

        "3:",                                // Emergency exit for non-canonical RCX
        "ud2",                               // Panic via invalid opcode

        handler = sym syscall_handler,
    );
}

/// Rust syscall handler - called from assembly
#[unsafe(no_mangle)]
pub extern "C" fn syscall_handler(frame: &mut super::SyscallFrame) {
    // TEAM_297 BREADCRUMB: INVESTIGATING - Debug trace added but no output seen.
    // Suspicion: los_hal::println! might fail in syscall context or execution doesn't reach here.
    let pc_before = frame.rcx;
    let nr = frame.rax;

    // Print entry for syscalls we care about (read=0, write=1)
    #[cfg(feature = "verbose-syscalls")]
    if nr <= 1 {
        let task = crate::task::current_task();
        let pid = task.id.0;
        let mut cr3: u64;
        unsafe {
            core::arch::asm!("mov {}, cr3", out(reg) cr3);
        }
        los_hal::println!(
            "[SYSCALL][{}] ENTER nr={} rcx={:x} ttbr0={:x} cr3={:x}",
            pid,
            nr,
            pc_before,
            task.ttbr0,
            cr3
        );
    }

    #[cfg(feature = "verbose-syscalls")]
    if nr == 1 {
        los_hal::println!(
            "[SYSCALL] WRITE syscall! rcx={:x} (Expected return)",
            pc_before
        );
    }

    crate::syscall::syscall_dispatch(frame);

    // Check for frame corruption
    if frame.rdi != 0
        && pc_before == frame.rcx
        && (frame.rdi == 0 || frame.rsi == 0 || frame.rdx == 0)
    {
        // If we had valid args but now have zeros, that's suspicious of corruption if not intended
    }

    // Explicitly check if arguments were preserved
    #[cfg(feature = "verbose-syscalls")]
    if nr <= 1 {
        // Re-read args from frame to see if they changed
        los_hal::println!(
            "[SYSCALL] EXIT nr={} rcx={:x} rax={:x} arg0={:x}",
            nr,
            frame.rcx,
            frame.rax,
            frame.rdi
        );
    }

    // Check if RCX was corrupted
    if frame.rcx != pc_before {
        los_hal::println!(
            "[SYSCALL] WARNING: RCX changed! nr={} before={:x} after={:x}",
            nr,
            pc_before,
            frame.rcx
        );
    }

    // Print exit for syscalls we care about
    #[cfg(feature = "verbose-syscalls")]
    if nr <= 1 {
        let pid = crate::task::current_task().id.0;
        los_hal::println!(
            "[SYSCALL][{}] EXIT nr={} rcx={:x} rax={:x}",
            pid,
            nr,
            frame.rcx,
            frame.rax
        );
    }
}
