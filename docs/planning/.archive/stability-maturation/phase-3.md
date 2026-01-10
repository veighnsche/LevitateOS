# Phase 3 — Migration

**TEAM_311**: ABI Stability Refactor
**Parent**: `docs/planning/stability-maturation/`
**Depends On**: Phase 2 complete
**Status**: BLOCKED (Kernel work required)
**Last Updated**: 2026-01-08

---

## ⚠️ BLOCKER: Kernel Fork/Exec Not Implemented

> **IMPORTANT**: The kernel's current `sys_clone` only supports **thread-style** clones
> (CLONE_VM | CLONE_THREAD). Fork-style clones (new address space) return ENOSYS.
> Additionally, `sys_exec` is a **stub** that returns ENOSYS.
>
> **Until the kernel implements fork-style clone and execve, Spawn/SpawnArgs cannot
> be removed.** They remain deprecated in los_abi as a reminder.
>
> See `kernel/src/syscall/process.rs` lines 424-428 and 139-140.

### Required Kernel Work (Future Team)
1. Implement fork-style `sys_clone` (copy address space, not share)
2. Implement proper `sys_exec` (replace current process with new ELF)
3. Then migrate callsites from spawn → fork+exec
4. Then remove Spawn/SpawnArgs from los_abi

---

## 1. Migration Strategy

### 1.1 Approach
**Breaking changes allowed** - Per user directive. No shims, no backward compatibility.

1. Update imports to use `los_abi`
2. Fix all compilation errors
3. Remove old definitions
4. Run tests

### 1.2 Migration Order (By Risk)
1. **Low Risk**: errno imports (constants only)
2. **Medium Risk**: syscall number imports (enums)
3. **High Risk**: spawn → clone+exec migration (logic change)

---

## 2. Call Site Inventory

### 2.1 Kernel Call Sites

| File | Uses | Migration |
|------|------|-----------|
| `kernel/src/syscall/mod.rs` | `SyscallNumber`, `errno::*` | Import from los_abi |
| `kernel/src/syscall/fs.rs` | `errno::*` | Import from los_abi |
| `kernel/src/syscall/process.rs` | `errno::*`, Spawn/SpawnArgs handlers | Remove handlers, import errno |
| `kernel/src/syscall/mm.rs` | `errno::*` | Import from los_abi |
| `kernel/src/syscall/signal.rs` | `errno::*` | Import from los_abi |
| `kernel/src/arch/aarch64/mod.rs` | `SyscallNumber` definition | Delete, re-export from los_abi |
| `kernel/src/arch/x86_64/mod.rs` | `SyscallNumber` definition | Delete, re-export from los_abi |

### 2.2 Userspace Call Sites

| File | Uses | Migration |
|------|------|-----------|
| `userspace/libsyscall/src/process.rs` | `spawn()`, `spawn_args()` | Replace with `clone()` + `exec()` |
| `userspace/libsyscall/src/sysno.rs` | Syscall numbers | Delete file, use los_abi |
| `userspace/libsyscall/src/errno.rs` | Error codes | Delete file, use los_abi |
| `userspace/libsyscall/src/lib.rs` | Re-exports | Update re-exports |
| `userspace/init/src/main.rs` | `spawn()` calls | Use `fork()` + `exec()` pattern |
| `userspace/levbox/src/*.rs` | `spawn()` calls | Use `fork()` + `exec()` pattern |

---

## 3. Spawn → Clone+Exec Migration

### 3.1 Current Pattern (To Remove)
```rust
// userspace/libsyscall/src/process.rs
pub fn spawn(path: &str) -> i64 {
    syscall2(SYS_SPAWN, path.as_ptr() as u64, path.len() as u64)
}
```

### 3.2 New Pattern (Linux Standard)
```rust
pub fn fork() -> i64 {
    // clone with SIGCHLD only
    syscall5(__NR_clone as u64, SIGCHLD as u64, 0, 0, 0, 0)
}

pub fn exec(path: &str, argv: &[*const u8], envp: &[*const u8]) -> i64 {
    syscall3(__NR_execve as u64, 
             path.as_ptr() as u64,
             argv.as_ptr() as u64,
             envp.as_ptr() as u64)
}
```

### 3.3 Callsite Migration
```rust
// Before (custom syscall)
let pid = spawn("/bin/ls");

// After (Linux standard)
let pid = fork();
if pid == 0 {
    exec("/bin/ls", &[b"/bin/ls\0".as_ptr(), null()], &[null()]);
    exit(1); // exec failed
}
```

---

## 4. Steps

### Step 1 — Migrate Kernel errno Imports
- [ ] Update `kernel/src/syscall/mod.rs` to import from los_abi
- [ ] Update all syscall/*.rs files
- [ ] Delete errno/errno_file modules

### Step 2 — Migrate Kernel SyscallNumber
- [ ] Update `kernel/src/arch/aarch64/mod.rs` to re-export from los_abi
- [ ] Update `kernel/src/arch/x86_64/mod.rs` to re-export from los_abi
- [ ] Update dispatch table

### Step 3 — Migrate Userspace libsyscall
- [ ] Delete `sysno.rs`, import from los_abi
- [ ] Delete `errno.rs`, import from los_abi
- [ ] Implement `fork()` and update `exec()`
- [ ] Remove `spawn()` and `spawn_args()`

### Step 4 — Migrate init and levbox
- [ ] Update all spawn() calls in init
- [ ] Update all spawn() calls in levbox
- [ ] Test process creation

### Step 5 — Remove Custom Syscall Handlers
- [ ] Remove Spawn handler from kernel
- [ ] Remove SpawnArgs handler from kernel
- [ ] Remove from SyscallNumber enum

See `phase-3-step-*.md` files for details.
