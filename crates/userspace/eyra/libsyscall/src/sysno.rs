// TEAM_310: Deep integration of linux-raw-sys definitions
// TEAM_210: Syscall constants

// Direct export of standard syscall numbers
pub use linux_raw_sys::general::{
    __NR_brk,
    __NR_chdir,
    __NR_clock_gettime,
    __NR_clone,
    __NR_close,
    __NR_dup,
    __NR_dup3,
    __NR_execve,
    __NR_exit,
    __NR_fstat,
    __NR_futex,
    __NR_getcwd,
    __NR_getdents64 as __NR_getdents, // Modern Linux uses getdents64
    __NR_getpid,
    __NR_getppid,
    __NR_ioctl,
    __NR_kill,
    __NR_linkat,
    __NR_mkdirat,
    __NR_mmap,
    __NR_mprotect,
    __NR_munmap,
    __NR_nanosleep,
    __NR_openat,
    __NR_pipe2,
    __NR_read,
    __NR_readlinkat,
    __NR_readv,
    __NR_reboot,
    __NR_renameat,
    __NR_rt_sigaction,
    __NR_rt_sigprocmask,
    __NR_rt_sigreturn,
    __NR_sched_yield,
    __NR_set_tid_address,
    __NR_symlinkat,
    __NR_unlinkat,
    __NR_utimensat,
    __NR_wait4,
    __NR_write,
    __NR_writev,
};

// Custom LevitateOS syscalls (defined as u32 to match __NR_* types)
pub const SYS_SPAWN: u32 = 1000;
pub const SYS_SPAWN_ARGS: u32 = 1001;
pub const SYS_SET_FOREGROUND: u32 = 1002;
pub const SYS_GET_FOREGROUND: u32 = 1003;
pub const SYS_ISATTY: u32 = 1010;

// TEAM_345: Architecture-specific pause syscall handling
// x86_64 has pause (34), aarch64 uses internal kernel implementation via SyscallNumber::Pause
#[cfg(target_arch = "x86_64")]
pub use linux_raw_sys::general::__NR_pause;

#[cfg(target_arch = "aarch64")]
#[allow(non_upper_case_globals)]
pub const __NR_pause: u32 = 236; // Maps to kernel's SyscallNumber::Pause
