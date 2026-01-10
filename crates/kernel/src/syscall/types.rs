//! Common syscall type definitions (SSOT).
//!
//! TEAM_418: Consolidated from scattered definitions across the codebase.
//! This module is the Single Source of Truth for time-related types used in syscalls.

// ============================================================================
// Time Types
// ============================================================================

/// Time value with microsecond precision.
///
/// Used by: gettimeofday, rusage (ru_utime, ru_stime)
///
/// Layout: 16 bytes (i64 + i64), matches Linux ABI.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Timeval {
    pub tv_sec: i64,
    pub tv_usec: i64,
}

/// Time value with nanosecond precision.
///
/// Used by: clock_gettime, clock_getres, nanosleep, utimensat
///
/// Layout: 16 bytes (i64 + i64), matches Linux ABI.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}
