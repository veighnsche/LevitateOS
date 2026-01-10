# Phase 1 Step 1: Callsite Inventory

**TEAM_311**: ABI Stability Refactor
**Generated**: 2026-01-08
**Status**: Complete

---

## 1. Spawn/SpawnArgs Callsites (TO MIGRATE)

### Userspace Callers

| File | Line | Usage | Migration |
|------|------|-------|-----------|
| `userspace/init/src/main.rs` | 24 | `spawn("shell")` | fork()+exec() |
| `userspace/shell/src/main.rs` | 158 | `spawn_args(path, &argv_strs)` | fork()+exec() |
| `userspace/levbox/src/bin/test/test_runner.rs` | 58 | `spawn(test)` | fork()+exec() |
| `userspace/levbox/src/bin/test/suite_test_core.rs` | 72 | `spawn_args(bin, full_args_slice)` | fork()+exec() |

### Userspace Library (TO DELETE)

| File | Line | Function |
|------|------|----------|
| `userspace/libsyscall/src/process.rs` | 80 | `pub fn spawn(path: &str)` |
| `userspace/libsyscall/src/process.rs` | 86 | `pub fn spawn_args(path: &str, argv: &[&str])` |

### Kernel Handlers (TO DELETE)

| File | Line | Handler |
|------|------|---------|
| `kernel/src/syscall/process.rs` | 47 | `sys_spawn()` |
| `kernel/src/syscall/process.rs` | 170 | `sys_spawn_args()` |
| `kernel/src/syscall/mod.rs` | 51-52 | Dispatch for Spawn |
| `kernel/src/syscall/mod.rs` | 77-81 | Dispatch for SpawnArgs |

### Kernel Infrastructure (TO KEEP, REFACTOR)

| File | Line | Code | Note |
|------|------|------|------|
| `kernel/src/task/process.rs` | 45 | `spawn_from_elf()` | Keep - used by exec() |
| `kernel/src/task/process.rs` | 67 | `spawn_from_elf_with_args()` | Keep - used by exec() |
| `kernel/src/init.rs` | 413 | `spawn_init()` | Internal kernel init |

---

## 2. Custom Syscall Numbers (1000+)

### Current Definitions (3 places - TO CONSOLIDATE)

| Syscall | userspace/sysno.rs | kernel/aarch64 | kernel/x86_64 | Action |
|---------|-------------------|----------------|---------------|--------|
| Spawn | SYS_SPAWN=1000 | Spawn=1000 | Spawn=1000 | DELETE |
| SpawnArgs | SYS_SPAWN_ARGS=1001 | SpawnArgs=1001 | SpawnArgs=1001 | DELETE |
| SetForeground | SYS_SET_FOREGROUND=1002 | SetForeground=1002 | SetForeground=1002 | KEEP |
| GetForeground | SYS_GET_FOREGROUND=1003 | GetForeground=1003 | GetForeground=1003 | KEEP |
| Isatty | SYS_ISATTY=1010 | Isatty=1010 | Isatty=1010 | KEEP |

---

## 3. Errno Definitions (TO CONSOLIDATE)

### kernel/src/syscall/mod.rs - errno module (lines 14-24)
```rust
pub const ENOENT: i64 = -2;
pub const EBADF: i64 = -9;
pub const ENOMEM: i64 = -12;
pub const EFAULT: i64 = -14;
pub const EEXIST: i64 = -17;
pub const EINVAL: i64 = -22;
pub const ENOSYS: i64 = -38;
pub const EIO: i64 = -5;
pub const ENOTTY: i64 = -25;
```

### kernel/src/syscall/mod.rs - errno_file module (lines 26-33) - DUPLICATE!
```rust
pub const ENOENT: i64 = -2;
pub const EMFILE: i64 = -24;
pub const ENOTDIR: i64 = -20;
pub const EACCES: i64 = -13;
pub const EEXIST: i64 = -17;
pub const EIO: i64 = -5;
```

### userspace/libsyscall/src/errno.rs (TO DELETE, use los_abi)
Separate file with similar definitions.

---

## 4. Summary

### Files to Modify (Migration)
- `userspace/init/src/main.rs` - 1 callsite
- `userspace/shell/src/main.rs` - 1 callsite  
- `userspace/levbox/src/bin/test/test_runner.rs` - 1 callsite
- `userspace/levbox/src/bin/test/suite_test_core.rs` - 1 callsite

### Files to Delete
- `userspace/libsyscall/src/sysno.rs` (replaced by los_abi)
- `userspace/libsyscall/src/errno.rs` (replaced by los_abi)

### Code to Delete
- `kernel/src/syscall/process.rs`: sys_spawn(), sys_spawn_args()
- `kernel/src/syscall/mod.rs`: errno module, errno_file module, Spawn/SpawnArgs dispatch
- `kernel/src/arch/*/mod.rs`: Spawn, SpawnArgs enum variants

### Code to Add
- `crates/abi/` - New crate
- `userspace/libsyscall/src/process.rs`: fork(), update exec()
