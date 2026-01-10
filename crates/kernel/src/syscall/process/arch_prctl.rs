//! Architecture-specific process control syscalls.
//!
//! TEAM_350: arch_prctl for x86_64 TLS/GS handling.
//! TEAM_417: Extracted from process.rs.

use crate::memory::user as mm_user;
use crate::syscall::errno;

// ============================================================================
// TEAM_350: arch_prctl (x86_64 only) - Set architecture-specific thread state
// ============================================================================

/// TEAM_350: arch_prctl codes for x86_64
#[cfg(target_arch = "x86_64")]
pub mod arch_prctl_codes {
    pub const ARCH_SET_GS: i32 = 0x1001;
    pub const ARCH_SET_FS: i32 = 0x1002;
    pub const ARCH_GET_FS: i32 = 0x1003;
    pub const ARCH_GET_GS: i32 = 0x1004;
}

/// TEAM_350: sys_arch_prctl - Set architecture-specific thread state (x86_64).
///
/// Used primarily for setting the FS base register for TLS (Thread Local Storage).
///
/// # Arguments
/// * `code` - ARCH_SET_FS, ARCH_GET_FS, ARCH_SET_GS, ARCH_GET_GS
/// * `addr` - Address to set/get
///
/// # Returns
/// 0 on success, negative errno on failure.
#[cfg(target_arch = "x86_64")]
pub fn sys_arch_prctl(code: i32, addr: usize) -> i64 {
    use arch_prctl_codes::*;

    log::trace!("[SYSCALL] arch_prctl(code=0x{:x}, addr=0x{:x})", code, addr);

    match code {
        ARCH_SET_FS => {
            // TEAM_350: Set FS base register for TLS
            // SAFETY: Writing to FS_BASE MSR is safe if addr is valid
            unsafe {
                // IA32_FS_BASE MSR = 0xC0000100
                core::arch::asm!(
                    "wrmsr",
                    in("ecx") 0xC000_0100u32,
                    in("eax") (addr as u32),
                    in("edx") ((addr >> 32) as u32),
                    options(nostack, preserves_flags)
                );
            }
            // TEAM_409: Store in BOTH task.tls AND context.fs_base for context switch restore
            // The context switch assembly restores from context.fs_base, not task.tls
            let task = crate::task::current_task();
            task.tls
                .store(addr, core::sync::atomic::Ordering::Release);
            // SAFETY: We're modifying our own context which won't be used until we context switch out
            unsafe {
                let ctx_ptr = &task.context as *const _ as *mut crate::arch::Context;
                (*ctx_ptr).set_tls(addr as u64);
            }
            0
        }
        ARCH_GET_FS => {
            // TEAM_350: Get FS base register
            let task = crate::task::current_task();
            if addr != 0 {
                if mm_user::validate_user_buffer(task.ttbr0, addr, 8, true).is_ok() {
                    if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, addr) {
                        let fs_base = task.tls.load(core::sync::atomic::Ordering::Acquire);
                        // SAFETY: validate_user_buffer confirmed address is writable
                        unsafe {
                            *(ptr as *mut u64) = fs_base as u64;
                        }
                        return 0;
                    }
                }
                return errno::EFAULT;
            }
            0
        }
        ARCH_SET_GS => {
            // TEAM_350: Set GS base register (less commonly used)
            unsafe {
                // IA32_GS_BASE MSR = 0xC0000101
                core::arch::asm!(
                    "wrmsr",
                    in("ecx") 0xC000_0101u32,
                    in("eax") (addr as u32),
                    in("edx") ((addr >> 32) as u32),
                    options(nostack, preserves_flags)
                );
            }
            0
        }
        ARCH_GET_GS => {
            // TEAM_350: Get GS base - read from MSR
            let task = crate::task::current_task();
            if addr != 0 {
                if mm_user::validate_user_buffer(task.ttbr0, addr, 8, true).is_ok() {
                    if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, addr) {
                        let gs_base: u64;
                        unsafe {
                            let lo: u32;
                            let hi: u32;
                            core::arch::asm!(
                                "rdmsr",
                                in("ecx") 0xC000_0101u32,
                                out("eax") lo,
                                out("edx") hi,
                                options(nostack, preserves_flags)
                            );
                            gs_base = ((hi as u64) << 32) | (lo as u64);
                            *(ptr as *mut u64) = gs_base;
                        }
                        return 0;
                    }
                }
                return errno::EFAULT;
            }
            0
        }
        _ => errno::EINVAL,
    }
}

/// TEAM_350: sys_arch_prctl stub for non-x86_64 architectures.
#[cfg(not(target_arch = "x86_64"))]
pub fn sys_arch_prctl(_code: i32, _addr: usize) -> i64 {
    // arch_prctl is x86_64-specific
    errno::ENOSYS
}
