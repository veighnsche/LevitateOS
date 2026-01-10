//! Resource usage and limits syscalls.
//!
//! TEAM_409: Resource usage (getrusage) and limits (prlimit64).
//! TEAM_417: Extracted from process.rs.
//! TEAM_418: Use Timeval from SSOT (syscall/types.rs).

use crate::syscall::errno;
// TEAM_418: Import Timeval from SSOT
pub use crate::syscall::types::Timeval;

// ============================================================================
// TEAM_409: Resource usage syscalls
// ============================================================================

/// TEAM_409: rusage structure for getrusage syscall.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Rusage {
    pub ru_utime: Timeval,  // User time used
    pub ru_stime: Timeval,  // System time used
    pub ru_maxrss: i64,     // Maximum resident set size
    pub ru_ixrss: i64,      // Integral shared memory size
    pub ru_idrss: i64,      // Integral unshared data size
    pub ru_isrss: i64,      // Integral unshared stack size
    pub ru_minflt: i64,     // Page reclaims (soft page faults)
    pub ru_majflt: i64,     // Page faults (hard page faults)
    pub ru_nswap: i64,      // Swaps
    pub ru_inblock: i64,    // Block input operations
    pub ru_oublock: i64,    // Block output operations
    pub ru_msgsnd: i64,     // IPC messages sent
    pub ru_msgrcv: i64,     // IPC messages received
    pub ru_nsignals: i64,   // Signals received
    pub ru_nvcsw: i64,      // Voluntary context switches
    pub ru_nivcsw: i64,     // Involuntary context switches
}

/// TEAM_409: sys_getrusage - Get resource usage.
///
/// Returns resource usage statistics for the calling process.
/// Currently returns zeros for most fields (simplified implementation).
///
/// # Arguments
/// * `who` - RUSAGE_SELF (0), RUSAGE_CHILDREN (-1), or RUSAGE_THREAD (1)
/// * `usage` - User pointer to rusage struct
///
/// # Returns
/// 0 on success, negative errno on failure.
pub fn sys_getrusage(who: i32, usage: usize) -> i64 {
    const RUSAGE_SELF: i32 = 0;
    const RUSAGE_CHILDREN: i32 = -1;
    const RUSAGE_THREAD: i32 = 1;

    // Validate who argument
    if who != RUSAGE_SELF && who != RUSAGE_CHILDREN && who != RUSAGE_THREAD {
        return errno::EINVAL;
    }

    if usage == 0 {
        return errno::EFAULT;
    }

    let task = crate::task::current_task();

    // Create a zeroed rusage struct (simplified - we don't track these metrics yet)
    let rusage = Rusage::default();

    // TEAM_416: Use write_struct_to_user helper instead of unwrap() for panic safety
    match crate::syscall::helpers::write_struct_to_user(task.ttbr0, usage, &rusage) {
        Ok(()) => 0,
        Err(e) => e,
    }
}

// ============================================================================
// TEAM_409: Resource limit syscalls
// TEAM_418: Use RLIMIT_* constants from SSOT
// ============================================================================

use crate::syscall::constants::{
    RLIMIT_CPU, RLIMIT_FSIZE, RLIMIT_DATA, RLIMIT_STACK, RLIMIT_CORE,
    RLIMIT_RSS, RLIMIT_NPROC, RLIMIT_NOFILE, RLIMIT_MEMLOCK, RLIMIT_AS,
    RLIM_INFINITY,
};

/// rlimit64 struct: { rlim_cur: u64, rlim_max: u64 }
#[repr(C)]
#[derive(Clone, Copy)]
struct Rlimit64 {
    rlim_cur: u64, // Soft limit
    rlim_max: u64, // Hard limit
}

/// TEAM_409: sys_prlimit64 - Get/set resource limits.
///
/// This is a stub implementation that returns sensible defaults.
/// Full resource limiting is not yet implemented.
///
/// # Arguments
/// * `pid` - Process ID (0 = current process)
/// * `resource` - Resource type (RLIMIT_*)
/// * `new_limit` - New limit to set (NULL to only get)
/// * `old_limit` - Buffer for old limit (NULL to only set)
///
/// # Returns
/// 0 on success, negative errno on failure.
pub fn sys_prlimit64(pid: i32, resource: u32, new_limit: usize, old_limit: usize) -> i64 {
    let task = crate::task::current_task();

    // Only support current process for now
    if pid != 0 && pid != task.id.0 as i32 {
        log::warn!(
            "[SYSCALL] prlimit64: pid {} not supported (only current process)",
            pid
        );
        return errno::ESRCH;
    }

    // Default limits (sensible values for a simple OS)
    let default_limit = match resource {
        RLIMIT_NOFILE => Rlimit64 {
            rlim_cur: 1024,
            rlim_max: 4096,
        },
        RLIMIT_STACK => Rlimit64 {
            rlim_cur: 8 * 1024 * 1024,
            rlim_max: RLIM_INFINITY,
        },
        RLIMIT_AS => Rlimit64 {
            rlim_cur: RLIM_INFINITY,
            rlim_max: RLIM_INFINITY,
        },
        RLIMIT_FSIZE => Rlimit64 {
            rlim_cur: RLIM_INFINITY,
            rlim_max: RLIM_INFINITY,
        },
        RLIMIT_DATA => Rlimit64 {
            rlim_cur: RLIM_INFINITY,
            rlim_max: RLIM_INFINITY,
        },
        RLIMIT_CORE => Rlimit64 {
            rlim_cur: 0,
            rlim_max: RLIM_INFINITY,
        },
        RLIMIT_CPU => Rlimit64 {
            rlim_cur: RLIM_INFINITY,
            rlim_max: RLIM_INFINITY,
        },
        RLIMIT_RSS => Rlimit64 {
            rlim_cur: RLIM_INFINITY,
            rlim_max: RLIM_INFINITY,
        },
        RLIMIT_NPROC => Rlimit64 {
            rlim_cur: 1024,
            rlim_max: 4096,
        },
        RLIMIT_MEMLOCK => Rlimit64 {
            rlim_cur: 64 * 1024,
            rlim_max: 64 * 1024,
        },
        _ => {
            log::warn!("[SYSCALL] prlimit64: unknown resource {}", resource);
            return errno::EINVAL;
        }
    };

    // Return old limit if requested
    if old_limit != 0 {
        // TEAM_416: Use write_struct_to_user helper instead of unwrap() for panic safety
        if let Err(e) =
            crate::syscall::helpers::write_struct_to_user(task.ttbr0, old_limit, &default_limit)
        {
            return e;
        }
    }

    // Setting new limit is a no-op for now (we don't enforce limits)
    if new_limit != 0 {
        log::trace!(
            "[SYSCALL] prlimit64: ignoring new_limit for resource {}",
            resource
        );
    }

    0
}
