# TEAM_314 — Implement Fork/Exec

**Created**: 2026-01-08
**Purpose**: Implement fork-exec per `docs/planning/fork-exec/`
**Status**: Complete

---

## Pre-Implementation Findings

### Pre-existing Build Errors (Fixed)

Before implementation, the codebase had build errors:

1. **`shutdown_flags::NORMAL` missing** — Added constant in `libsyscall/src/process.rs`
2. **`SIGINT` type mismatch** — Cast to i32 in `shell/src/main.rs`
3. **`Stat` Default not impl** — Used `zeroed()` in `systest/stat_test.rs`
4. **`Timespec` Default not impl** — Used `zeroed()` in `systest/time_test.rs`

### Pre-existing Behavior Test Failure

The x86_64 behavior test shows corrupted serial output:
- Expected: `LH123456789CXLR[BOOT] Stage 1: Early HAL (SEC)`
- Got: `LH123456789CXLRika`

This is unrelated to fork-exec implementation (serial/QEMU issue).

---

## Implementation Summary

### Step 1: Memory Primitives ✅
- `copy_user_address_space()` — Copies parent's page tables for fork
- `clear_user_address_space()` — Clears user pages for exec

### Step 2: Fork Implementation ✅
- Modified `sys_clone()` to call `sys_clone_fork()` when CLONE_VM not set
- Added `create_forked_process()` in `task/thread.rs`
- Child returns 0, parent returns child PID

### Step 3: Exec Implementation ✅
- Full `sys_exec()` that:
  - Clears old address space
  - Loads new ELF
  - Sets up new stack
  - Modifies syscall frame to return to new entry point

### Step 4: Userspace API ✅
- `fork()` — Wraps clone with SIGCHLD flag
- `exec_args()` — Placeholder for execve with args

### Step 5: Integration ✅
- `spawn()` now uses fork+exec internally
- `spawn_args()` now uses fork+exec internally
- Kernel Spawn/SpawnArgs handlers return ENOSYS

---

## Files Modified

### Kernel
- `kernel/src/memory/user.rs` — Added copy/clear address space
- `kernel/src/syscall/process.rs` — sys_clone_fork, sys_exec
- `kernel/src/syscall/mod.rs` — ENOEXEC, deprecated handlers
- `kernel/src/task/thread.rs` — create_forked_process
- `kernel/src/arch/aarch64/mod.rs` — sp/pc methods for SyscallFrame
- `kernel/src/arch/x86_64/mod.rs` — No changes needed

### Userspace
- `userspace/libsyscall/src/process.rs` — fork, exec_args, updated spawn/spawn_args
- `userspace/shell/src/main.rs` — SIGINT cast fix
- `userspace/systest/src/bin/stat_test.rs` — zeroed() fix
- `userspace/systest/src/bin/time_test.rs` — zeroed() fix

---

## Known Limitations

1. **No O_CLOEXEC** — FD close-on-exec not implemented yet
2. **No COW** — Fork uses eager copy (as planned)

## Post-Implementation: argv Support Added

TEAM_314 added full argv support to execve:
- `sys_exec()` now accepts `argv_ptr` and `argc` parameters
- Kernel parses `UserArgvEntry` array from userspace (max 16 args)
- `exec_args()` in libsyscall now passes argv to kernel
- `exec()` updated to use 4-arg syscall with argc=0

---

## Handoff Checklist

- [x] Project builds cleanly
- [x] No deprecation warnings for Spawn/SpawnArgs
- [x] Fork implementation complete
- [x] Exec implementation complete
- [x] Userspace API complete
- [x] spawn/spawn_args migrated to fork+exec
- [ ] Behavior test passes (pre-existing failure unrelated to fork-exec)
