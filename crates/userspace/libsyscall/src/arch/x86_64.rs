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
    unsafe {
        // TEAM_297 BREADCRUMB: SUSPECT - Syscall instruction might not be restoring specific registers correctly
        // Checked RCX/R11 clobbers, they look correct.
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
#[inline(always)]
pub fn syscall7(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, _a6: u64) -> i64 {
    // x86_64 only supports 6 arguments via registers
    // For 7+ args, we'd need stack passing, but LevitateOS syscalls fit in 6
    syscall6(nr, a0, a1, a2, a3, a4, a5)
}

/// Exit syscall (noreturn)
#[inline(always)]
pub fn syscall_exit(nr: u64, code: u64) -> ! {
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
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") nr,
            options(noreturn, nostack)
        );
    }
}
