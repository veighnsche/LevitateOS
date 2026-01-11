//! TEAM_276: x86_64 syscall primitives
//!
//! Uses `syscall` instruction with:
//! - rax: syscall number (input), return value (output)
//! - rdi, rsi, rdx, r10, r8, r9: arguments 1-6
//! - rcx, r11: clobbered by syscall instruction

/// Syscall with 0 arguments
#[inline(always)]
pub fn syscall0(nr: u64) -> i64 {
    let ret: i64;
    // SAFETY: The x86_64 `syscall` instruction is safe to execute from userspace (ring 3).
    // The kernel validates the syscall number in rax and returns an error code if invalid.
    // This follows the Linux x86_64 syscall ABI where:
    // - rax holds the syscall number (input) and return value (output)
    // - rcx and r11 are clobbered by the syscall instruction (saved/restored by kernel)
    // The `nostack` option ensures no stack operations occur, which is safe because
    // this function doesn't use stack memory.
    // TEAM_297: RCX/R11 clobbers verified correct per x86_64 syscall convention.
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr => ret,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 1 argument
#[inline(always)]
pub fn syscall1(nr: u64, a0: u64) -> i64 {
    let ret: i64;
    // SAFETY: See syscall0 for ABI documentation. Arguments are passed in rdi.
    // The kernel is responsible for validating argument values.
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a0,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 2 arguments
#[inline(always)]
pub fn syscall2(nr: u64, a0: u64, a1: u64) -> i64 {
    let ret: i64;
    // SAFETY: See syscall0 for ABI documentation. Arguments are passed in rdi, rsi.
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a0,
            in("rsi") a1,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 3 arguments
#[inline(always)]
pub fn syscall3(nr: u64, a0: u64, a1: u64, a2: u64) -> i64 {
    let ret: i64;
    // SAFETY: See syscall0 for ABI documentation. Arguments are passed in rdi, rsi, rdx.
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a0,
            in("rsi") a1,
            in("rdx") a2,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 4 arguments
#[inline(always)]
pub fn syscall4(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> i64 {
    let ret: i64;
    // SAFETY: See syscall0 for ABI documentation. Arguments: rdi, rsi, rdx, r10.
    // Note: r10 is used instead of rcx (which is clobbered by syscall instruction).
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a0,
            in("rsi") a1,
            in("rdx") a2,
            in("r10") a3,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 5 arguments
#[inline(always)]
pub fn syscall5(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64) -> i64 {
    let ret: i64;
    // SAFETY: See syscall0 for ABI documentation. Arguments: rdi, rsi, rdx, r10, r8.
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a0,
            in("rsi") a1,
            in("rdx") a2,
            in("r10") a3,
            in("r8") a4,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 6 arguments
#[inline(always)]
pub fn syscall6(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    let ret: i64;
    // SAFETY: See syscall0 for ABI documentation. Arguments: rdi, rsi, rdx, r10, r8, r9.
    // This is the maximum number of arguments supported by x86_64 syscall registers.
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") nr => ret,
            in("rdi") a0,
            in("rsi") a1,
            in("rdx") a2,
            in("r10") a3,
            in("r8") a4,
            in("r9") a5,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 7 arguments (for linkat which needs 6 args + flags)
///
/// **IMPORTANT**: x86_64 only supports 6 syscall arguments via registers.
/// The 7th argument (`a6`) is **silently ignored** on this architecture.
/// This is acceptable because LevitateOS currently has no syscalls that require
/// 7 arguments on x86_64. AArch64 supports 7 arguments via x0-x6 registers.
///
/// If you need a true 7-argument syscall on x86_64, the implementation would
/// need to be extended to pass arguments via the stack, which is not currently supported.
#[inline(always)]
pub fn syscall7(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, _a6: u64) -> i64 {
    // SAFETY: This delegates to syscall6, see its SAFETY comment.
    // The 7th argument is intentionally discarded per architecture limitation.
    syscall6(nr, a0, a1, a2, a3, a4, a5)
}

/// Exit syscall (noreturn)
#[inline(always)]
pub fn syscall_exit(nr: u64, code: u64) -> ! {
    // SAFETY: This function never returns, which is correct for exit-like syscalls.
    // The kernel terminates the process and never returns to this code path.
    // The `noreturn` option correctly marks this as diverging.
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") nr,
            in("rdi") code,
            options(noreturn, nostack)
        );
    }
}

/// Sigreturn syscall (noreturn)
#[inline(always)]
pub fn syscall_noreturn(nr: u64) -> ! {
    // SAFETY: This is used for signal return (rt_sigreturn) which restores the full
    // process context from the signal frame and never returns to this code path.
    // The kernel either successfully restores context (jumping elsewhere) or terminates
    // the process if the signal frame is invalid. The `noreturn` option is correct.
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") nr,
            options(noreturn, nostack)
        );
    }
}
