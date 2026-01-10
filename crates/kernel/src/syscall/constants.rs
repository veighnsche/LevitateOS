//! Common syscall constants (SSOT).
//!
//! TEAM_418: Consolidated from scattered definitions across the codebase.
//! This module is the Single Source of Truth for syscall-related constants.

// ============================================================================
// Path Constants
// ============================================================================

/// Maximum path length (Linux standard).
pub const PATH_MAX: usize = 4096;

// ============================================================================
// Clone Flags (Linux ABI)
// ============================================================================

/// Share virtual memory with parent.
pub const CLONE_VM: u64 = 0x00000100;
/// Share filesystem information with parent.
pub const CLONE_FS: u64 = 0x00000200;
/// Share file descriptor table with parent.
pub const CLONE_FILES: u64 = 0x00000400;
/// Share signal handlers with parent.
pub const CLONE_SIGHAND: u64 = 0x00000800;
/// Create as a thread (share thread group).
pub const CLONE_THREAD: u64 = 0x00010000;
/// Set TLS for the child.
pub const CLONE_SETTLS: u64 = 0x00080000;
/// Write parent TID to parent's memory.
pub const CLONE_PARENT_SETTID: u64 = 0x00100000;
/// Clear child TID in child's memory on exit.
pub const CLONE_CHILD_CLEARTID: u64 = 0x00200000;
/// Write child TID to child's memory.
pub const CLONE_CHILD_SETTID: u64 = 0x01000000;

// ============================================================================
// Resource Limit Constants (Linux ABI)
// ============================================================================

/// CPU time limit in seconds.
pub const RLIMIT_CPU: u32 = 0;
/// Maximum file size.
pub const RLIMIT_FSIZE: u32 = 1;
/// Maximum data segment size.
pub const RLIMIT_DATA: u32 = 2;
/// Maximum stack size.
pub const RLIMIT_STACK: u32 = 3;
/// Maximum core file size.
pub const RLIMIT_CORE: u32 = 4;
/// Maximum resident set size.
pub const RLIMIT_RSS: u32 = 5;
/// Maximum number of processes.
pub const RLIMIT_NPROC: u32 = 6;
/// Maximum number of open files.
pub const RLIMIT_NOFILE: u32 = 7;
/// Maximum locked memory.
pub const RLIMIT_MEMLOCK: u32 = 8;
/// Address space limit.
pub const RLIMIT_AS: u32 = 9;

/// Infinity value for resource limits.
pub const RLIM_INFINITY: u64 = u64::MAX;
