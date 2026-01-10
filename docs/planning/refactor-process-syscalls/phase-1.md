# Phase 1: Discovery & Safeguards

## Refactor Summary

**Target File**: `crates/kernel/src/syscall/process.rs`
**Current Size**: 1090 lines (exceeds 500-line ideal, over 1000-line threshold)

### What
Refactor the process syscall module from a monolithic 1090-line file into focused, single-responsibility modules.

### Why
- File exceeds the 1000-line hard limit and 500-line ideal (Rule 7)
- Mixes unrelated concerns: process lifecycle, threading, user/group identity, resource limits, architecture-specific code
- x86_64-specific arch_prctl implementation is deeply embedded
- Multiple struct definitions scattered throughout

### Pain Points
1. **Size**: 1090 lines makes navigation difficult
2. **Mixed concerns**: 8+ distinct functional areas in one file
3. **Architecture coupling**: x86_64 arch_prctl code mixed with generic syscalls
4. **Scattered constants**: Clone flags, resource limits, error codes spread around
5. **Limited testability**: Monolithic structure prevents unit testing

## Success Criteria

### Before
- Single 1090-line file with 8+ distinct areas
- x86_64 code mixed with portable code
- No clear module boundaries
- Structs (Utsname, Rusage, Timeval, Rlimit64) inline

### After
- Multiple focused modules under 500 lines each
- Architecture-specific code isolated
- Clear module boundaries with explicit exports
- Reusable struct definitions in dedicated location

## Behavioral Contracts (APIs that MUST NOT change)

All public syscall functions must maintain identical signatures:

```rust
// Process lifecycle
pub fn sys_exit(code: i32) -> i64;
pub fn sys_getpid() -> i64;
pub fn sys_getppid() -> i64;
pub fn sys_yield() -> i64;
pub fn sys_spawn(path_ptr: usize, path_len: usize) -> i64;
pub fn sys_exec(path_ptr: usize, path_len: usize) -> i64;
pub fn sys_spawn_args(path_ptr: usize, path_len: usize, argv_ptr: usize, argc: usize) -> i64;
pub fn sys_waitpid(pid: i32, status_ptr: usize) -> i64;
pub fn sys_set_foreground(pid: usize) -> i64;
pub fn sys_get_foreground() -> i64;

// Threading
pub fn sys_clone(flags: u64, stack: usize, parent_tid: usize, tls: usize, child_tid: usize, tf: &SyscallFrame) -> i64;
pub fn sys_set_tid_address(tidptr: usize) -> i64;
pub fn sys_gettid() -> i64;
pub fn sys_exit_group(status: i32) -> i64;

// Identity
pub fn sys_getuid() -> i64;
pub fn sys_geteuid() -> i64;
pub fn sys_getgid() -> i64;
pub fn sys_getegid() -> i64;

// Process groups/sessions
pub fn sys_setpgid(pid: i32, pgid: i32) -> i64;
pub fn sys_getpgid(pid: i32) -> i64;
pub fn sys_getpgrp() -> i64;
pub fn sys_setsid() -> i64;

// System info
pub fn sys_uname(buf: usize) -> i64;
pub fn sys_umask(mask: u32) -> i64;

// Resource usage/limits
pub fn sys_getrusage(who: i32, usage: usize) -> i64;
pub fn sys_prlimit64(pid: i32, resource: u32, new_limit: usize, old_limit: usize) -> i64;

// Architecture-specific
#[cfg(target_arch = "x86_64")]
pub fn sys_arch_prctl(code: i32, addr: usize) -> i64;
#[cfg(not(target_arch = "x86_64"))]
pub fn sys_arch_prctl(_code: i32, _addr: usize) -> i64;
```

### Constants that must remain exported:
- `CLONE_*` flags (CLONE_VM, CLONE_FS, CLONE_FILES, etc.)
- `Utsname`, `Rusage`, `Timeval` structs (if used externally)

## Golden/Regression Tests to Lock In

### Behavioral Tests
- Boot sequence with `--features verbose` captures process syscall behavior
- Any boot test that spawns processes validates spawn/spawn_args

### Build Verification
- Both architectures must compile: `cargo xtask build kernel --arch aarch64` and `--arch x86_64`
- The `#[cfg(target_arch)]` attributes must be preserved correctly

### Runtime Contracts
- Eyra coreutils must continue to work (uses most of these syscalls)
- Shell must spawn processes correctly
- Process hierarchy (getppid) must remain consistent

## Current Architecture Notes

### Dependencies (Inbound)
- `crate::syscall::mod.rs` - Dispatches to these functions
- `crate::arch::SyscallFrame` - Used by sys_clone

### Dependencies (Outbound)
- `crate::memory::user` (mm_user) - Buffer validation, VA translation
- `crate::task::*` - Process/thread management
- `crate::fs::INITRAMFS` - Executable loading
- `crate::syscall::errno` - Error codes
- `crate::syscall::helpers` - write_struct_to_user
- `los_hal::IrqSafeLock`, `los_hal::interrupts`
- `los_utils::cpio::CpioEntryType`

### Functional Areas (8 identified)
1. **Helpers** (lines 9-134): resolve_initramfs_executable, clone_fd_table_for_child, etc.
2. **Lifecycle** (lines 136-250): exit, getpid, getppid, yield, spawn, exec
3. **Spawn with args** (lines 252-364): sys_spawn_args
4. **Waitpid** (lines 366-420): sys_waitpid, foreground process
5. **Threading** (lines 422-557): clone, set_tid_address
6. **Identity** (lines 559-610): getuid, geteuid, getgid, getegid, gettid, exit_group
7. **arch_prctl** (lines 612-734): x86_64-specific TLS/GS handling
8. **Process groups** (lines 736-828): setpgid, getpgid, getpgrp, setsid
9. **System info** (lines 830-933): Utsname struct, uname, umask
10. **Resource usage** (lines 935-1090): Rusage, Timeval, getrusage, prlimit64

## Constraints

1. **No runtime behavior changes**: All syscalls must behave identically
2. **No compatibility shims**: Use compiler errors to find call sites
3. **Maintain arch separation**: x86_64 code stays cfg-gated
4. **Keep file sizes under 500 lines**: Each new module should be focused
5. **Preserve TEAM comments**: Historical context must be maintained
