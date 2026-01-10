//! Process identity syscalls.
//!
//! TEAM_350: User/group identity syscalls.
//! TEAM_406: System identification (uname, umask).
//! TEAM_417: Extracted from process.rs.

use crate::memory::user as mm_user;
use crate::syscall::errno;
use core::sync::atomic::Ordering;

// ============================================================================
// TEAM_350: Eyra Prerequisites - Simple Identity Syscalls
// ============================================================================

/// TEAM_350: sys_gettid - Get thread ID.
///
/// Returns the caller's thread ID (TID). In a single-threaded process,
/// this is the same as the PID.
pub fn sys_gettid() -> i64 {
    crate::task::current_task().id.0 as i64
}

/// TEAM_350: sys_getuid - Get real user ID.
///
/// LevitateOS is single-user, always returns 0 (root).
pub fn sys_getuid() -> i64 {
    0
}

/// TEAM_350: sys_geteuid - Get effective user ID.
///
/// LevitateOS is single-user, always returns 0 (root).
pub fn sys_geteuid() -> i64 {
    0
}

/// TEAM_350: sys_getgid - Get real group ID.
///
/// LevitateOS is single-user, always returns 0 (root group).
pub fn sys_getgid() -> i64 {
    0
}

/// TEAM_350: sys_getegid - Get effective group ID.
///
/// LevitateOS is single-user, always returns 0 (root group).
pub fn sys_getegid() -> i64 {
    0
}

// ============================================================================
// TEAM_406: System identification and file creation mask
// ============================================================================

/// TEAM_406: Linux utsname structure for sys_uname.
#[repr(C)]
pub struct Utsname {
    pub sysname: [u8; 65],
    pub nodename: [u8; 65],
    pub release: [u8; 65],
    pub version: [u8; 65],
    pub machine: [u8; 65],
    pub domainname: [u8; 65],
}

impl Default for Utsname {
    fn default() -> Self {
        Self {
            sysname: [0; 65],
            nodename: [0; 65],
            release: [0; 65],
            version: [0; 65],
            machine: [0; 65],
            domainname: [0; 65],
        }
    }
}

/// TEAM_406: Copy a string into a fixed-size array, null-terminated.
fn str_to_array<const N: usize>(s: &str) -> [u8; N] {
    let mut arr = [0u8; N];
    let bytes = s.as_bytes();
    let len = bytes.len().min(N - 1); // Leave room for null terminator
    arr[..len].copy_from_slice(&bytes[..len]);
    arr
}

/// TEAM_406: sys_uname - Get system identification.
///
/// Fills the utsname structure with system information.
///
/// # Arguments
/// * `buf` - User pointer to utsname structure
///
/// # Returns
/// 0 on success, negative errno on failure.
pub fn sys_uname(buf: usize) -> i64 {
    let task = crate::task::current_task();

    // Validate user buffer
    let size = core::mem::size_of::<Utsname>();
    if mm_user::validate_user_buffer(task.ttbr0, buf, size, true).is_err() {
        return errno::EFAULT;
    }

    #[cfg(target_arch = "x86_64")]
    const MACHINE: &str = "x86_64";
    #[cfg(target_arch = "aarch64")]
    const MACHINE: &str = "aarch64";

    let utsname = Utsname {
        sysname: str_to_array("LevitateOS"),
        nodename: str_to_array("levitate"),
        release: str_to_array("0.1.0"),
        version: str_to_array("0.1.0"),
        machine: str_to_array(MACHINE),
        domainname: str_to_array("(none)"),
    };

    // Copy to user space byte by byte
    let bytes = unsafe {
        core::slice::from_raw_parts(&utsname as *const Utsname as *const u8, size)
    };

    for (i, &byte) in bytes.iter().enumerate() {
        if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, buf + i) {
            // SAFETY: We validated the buffer above
            unsafe {
                *ptr = byte;
            }
        } else {
            return errno::EFAULT;
        }
    }

    log::trace!("[SYSCALL] uname() -> 0");
    0
}

/// TEAM_406: sys_umask - Set file creation mask.
///
/// Sets the file mode creation mask and returns the old mask.
///
/// # Arguments
/// * `mask` - New file mode creation mask
///
/// # Returns
/// Previous umask value.
pub fn sys_umask(mask: u32) -> i64 {
    let task = crate::task::current_task();
    let old = task.umask.swap(mask & 0o777, Ordering::SeqCst);
    log::trace!("[SYSCALL] umask(0o{:o}) -> 0o{:o}", mask, old);
    old as i64
}
