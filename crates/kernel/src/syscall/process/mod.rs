//! Process management syscalls.
//!
//! TEAM_417: Refactored from monolithic process.rs for maintainability.
//! TEAM_420: Uses linux_raw_sys directly, no shims
//! See `docs/planning/refactor-process-syscalls/` for refactor details.

mod arch_prctl;
mod groups;
mod identity;
mod lifecycle;
mod resources;
mod thread;

// Re-export all syscall functions
pub use arch_prctl::sys_arch_prctl;
pub use groups::{sys_getpgid, sys_getpgrp, sys_setpgid, sys_setsid};
pub use identity::{
    sys_getegid, sys_geteuid, sys_getgid, sys_gettid, sys_getuid, sys_uname, sys_umask,
};
pub use lifecycle::{
    sys_exec, sys_exit, sys_exit_group, sys_get_foreground, sys_getpid, sys_getppid,
    sys_set_foreground, sys_spawn, sys_spawn_args, sys_waitpid, sys_yield,
};
pub use resources::{sys_getrusage, sys_prlimit64};
pub use thread::{sys_clone, sys_set_tid_address};

// Re-export types that may be used externally
pub use identity::Utsname;
pub use resources::{Rusage, Timeval};

// TEAM_420: No shims - use linux_raw_sys::general::CLONE_* directly at callsites
