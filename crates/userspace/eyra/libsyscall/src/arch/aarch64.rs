//! TEAM_275: AArch64 syscall primitives
//!
//! Uses `svc #0` instruction with x8 for syscall number and x0-x5 for arguments.

/// Syscall with 0 arguments
#[inline(always)]
pub fn syscall0(nr: u64) -> i64 {
    let ret: i64;
    // SAFETY: The AArch64 `svc #0` instruction is safe to execute from userspace (EL0).
    // The kernel validates the syscall number in x8 and returns an error code if invalid.
    // This follows the Linux AArch64 syscall ABI where x8 holds the syscall number and
    // x0 contains the return value. The `nostack` option ensures no stack operations occur,
    // which is safe because this function doesn't use stack memory.
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") nr,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 1 argument
#[inline(always)]
pub fn syscall1(nr: u64, a0: u64) -> i64 {
    let ret: i64;
    // SAFETY: See syscall0 for ABI documentation. Arguments are passed in x0.
    // The kernel is responsible for validating argument values.
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") nr,
            in("x0") a0,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 2 arguments
#[inline(always)]
pub fn syscall2(nr: u64, a0: u64, a1: u64) -> i64 {
    let ret: i64;
    // SAFETY: See syscall0 for ABI documentation. Arguments are passed in x0-x1.
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") nr,
            in("x0") a0,
            in("x1") a1,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 3 arguments
#[inline(always)]
pub fn syscall3(nr: u64, a0: u64, a1: u64, a2: u64) -> i64 {
    let ret: i64;
    // SAFETY: See syscall0 for ABI documentation. Arguments are passed in x0-x2.
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") nr,
            in("x0") a0,
            in("x1") a1,
            in("x2") a2,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 4 arguments
#[inline(always)]
pub fn syscall4(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> i64 {
    let ret: i64;
    // SAFETY: See syscall0 for ABI documentation. Arguments are passed in x0-x3.
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") nr,
            in("x0") a0,
            in("x1") a1,
            in("x2") a2,
            in("x3") a3,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 5 arguments
#[inline(always)]
pub fn syscall5(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64) -> i64 {
    let ret: i64;
    // SAFETY: See syscall0 for ABI documentation. Arguments are passed in x0-x4.
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") nr,
            in("x0") a0,
            in("x1") a1,
            in("x2") a2,
            in("x3") a3,
            in("x4") a4,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 6 arguments
#[inline(always)]
pub fn syscall6(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    let ret: i64;
    // SAFETY: See syscall0 for ABI documentation. Arguments are passed in x0-x5.
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") nr,
            in("x0") a0,
            in("x1") a1,
            in("x2") a2,
            in("x3") a3,
            in("x4") a4,
            in("x5") a5,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret
}

/// Syscall with 7 arguments (for linkat which needs 6 args + syscall number)
#[inline(always)]
pub fn syscall7(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64) -> i64 {
    let ret: i64;
    // SAFETY: See syscall0 for ABI documentation. Arguments are passed in x0-x6.
    // AArch64 supports 7 arguments via registers (x0-x6), unlike x86_64 which is limited to 6.
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") nr,
            in("x0") a0,
            in("x1") a1,
            in("x2") a2,
            in("x3") a3,
            in("x4") a4,
            in("x5") a5,
            in("x6") a6,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret
}

/// Exit syscall (noreturn)
#[inline(always)]
pub fn syscall_exit(nr: u64, code: u64) -> ! {
    // SAFETY: This function never returns, which is correct for exit-like syscalls.
    // The kernel terminates the process and never returns to this code path.
    // The `noreturn` option correctly marks this as diverging.
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") nr,
            in("x0") code,
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
            "svc #0",
            in("x8") nr,
            options(noreturn, nostack)
        );
    }
}
