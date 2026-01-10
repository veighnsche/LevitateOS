# Phase 1 — Discovery and Safeguards

**TEAM_311**: ABI Stability Refactor
**Parent**: `docs/planning/stability-maturation/`
**Status**: In Progress
**Date**: 2026-01-08

---

## 1. Refactor Summary

**What**: Create `crates/abi` as single source of truth for syscall ABI, migrate custom syscalls (Spawn/SpawnArgs) to Linux clone/execve, break code and fix callsites.

**Pain Points**:
1. Syscall numbers defined in 3 places (kernel aarch64, kernel x86_64, userspace) - drift causes crashes
2. Custom syscalls (1000+) prevent Linux binary compatibility
3. Errno codes duplicated in 3 places
4. No compile-time verification of ABI agreement
5. Hand-rolled code where battle-tested crates exist (ELF, GDT/IDT, Multiboot2)

**Motivation**: One change crashes everything. Need proper isolation and stable ABI.

---

## 2. Success Criteria

### Before
- Syscall numbers: 3 places, manual sync required
- Custom syscalls: Spawn (1000), SpawnArgs (1001), SetForeground (1002), GetForeground (1003), Isatty (1010)
- Errno: kernel `errno` + `errno_file` modules + userspace `errno.rs`
- No compile-time ABI verification

### After
- Syscall numbers: 1 place (`crates/abi`), kernel + userspace import same types
- Custom syscalls: REMOVED, use Linux clone(220) + execve(221)
- Errno: 1 place (use `linux-raw-sys` in userspace)
- Compile-time size assertions on all ABI structures
- Hand-rolled code replaced with battle-tested crates
- All tests pass

---

## 3. Behavioral Contracts

### 3.1 Syscall ABI (Must Preserve)
| Syscall | AArch64 NR | x86_64 NR | Signature |
|---------|------------|-----------|-----------|
| read | 63 | 0 | `(fd, buf, len) -> isize` |
| write | 64 | 1 | `(fd, buf, len) -> isize` |
| exit | 93 | 60 | `(code) -> !` |
| openat | 56 | 257 | `(dirfd, path, flags, mode) -> i32` |
| close | 57 | 3 | `(fd) -> i32` |
| clone | 220 | 56 | `(flags, stack, ptid, tls, ctid) -> i32` |
| execve | 221 | 59 | `(path, argv, envp) -> i32` |

### 3.2 Error Codes (Must Match Linux)
```
ENOENT = -2, EBADF = -9, ENOMEM = -12, EFAULT = -14, 
EEXIST = -17, EINVAL = -22, ENOSYS = -38, EIO = -5
```

### 3.3 Data Structures (Must Match Linux)
- `Stat` (128 bytes, arch-specific layout)
- `Timespec` (16 bytes)
- `Termios` (60 bytes)
- `Dirent64` (variable, d_reclen aligned)

---

## 4. Golden/Regression Tests

### 4.1 Existing Tests
- `tests/golden_boot.txt` - Boot sequence output
- `tests/golden_boot_x86_64.txt` - x86_64 boot output
- `tests/golden_shutdown.txt` - Shutdown sequence
- `cargo test -p los_utils -p los_error --features std` - Library tests

### 4.2 Tests to Add
- [ ] ABI structure size assertions
- [ ] Syscall number consistency test
- [ ] Errno value consistency test

---

## 5. Current Architecture

### 5.1 Dependency Graph (Syscall Path)
```
userspace/init
    └── userspace/libsyscall
            └── libsyscall/src/sysno.rs (syscall numbers)
            └── libsyscall/src/arch/*.rs (syscall invocation)

kernel/src/arch/*/mod.rs (SyscallNumber enum)
    └── kernel/src/syscall/mod.rs (dispatch)
        └── kernel/src/syscall/*.rs (implementations)
```

### 5.2 Files to Modify

| File | Change |
|------|--------|
| `kernel/src/arch/aarch64/mod.rs` | Remove SyscallNumber, import from los_abi |
| `kernel/src/arch/x86_64/mod.rs` | Remove SyscallNumber, import from los_abi |
| `kernel/src/syscall/mod.rs` | Remove errno modules, update dispatch |
| `userspace/libsyscall/src/sysno.rs` | Remove, use los_abi |
| `userspace/libsyscall/src/errno.rs` | Remove, use los_abi |
| `userspace/libsyscall/src/process.rs` | Replace spawn() with clone()+exec() |
| `userspace/init/src/main.rs` | Update spawn calls |
| `userspace/levbox/src/*.rs` | Update spawn calls |

### 5.3 Custom Syscalls to Remove
| Syscall | NR | Replacement |
|---------|-----|-------------|
| Spawn | 1000 | `clone()` + `execve()` |
| SpawnArgs | 1001 | `clone()` + `execve()` |
| SetForeground | 1002 | Keep (LevitateOS-specific) |
| GetForeground | 1003 | Keep (LevitateOS-specific) |
| Isatty | 1010 | Use `ioctl(TCGETS)` or keep |

---

## 6. Constraints

1. **Linux ABI Compatibility**: All standard syscalls must use Linux numbers
2. **Breaking Changes**: Allowed - fix all callsites
3. **Architecture Support**: Must work for both aarch64 and x86_64
4. **No Shims**: Clean migration, no backward-compat wrappers

---

## 7. Steps

### Step 1 — Inventory Current State
- [ ] List all syscall usages in userspace
- [ ] List all spawn/spawn_args callsites
- [ ] Verify golden tests pass

### Step 2 — Create crates/abi Foundation
- [ ] Create `crates/abi/Cargo.toml`
- [ ] Create `crates/abi/src/lib.rs`
- [ ] Create `crates/abi/src/errno.rs`
- [ ] Add size assertions

### Step 3 — Add Regression Tests
- [ ] Add syscall number consistency test
- [ ] Add errno consistency test
- [ ] Add structure size assertions

See `phase-1-step-1.md`, `phase-1-step-2.md`, `phase-1-step-3.md` for details.
