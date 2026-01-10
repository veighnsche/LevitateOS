# Phase 2: Syscall Path Safety

**Status**: ✅ COMPLETE (TEAM_416)
**TEAM**: TEAM_415
**Dependencies**: Phase 1

---

## Purpose

Replace all `user_va_to_kernel_ptr().unwrap()` calls in syscall handlers with proper error handling that returns `EFAULT`.

---

## Target Design

### Error Handling Pattern

```rust
// Current (unsafe)
let dest = mm_user::user_va_to_kernel_ptr(task.ttbr0, addr).unwrap();

// New (safe)
let dest = mm_user::user_va_to_kernel_ptr(task.ttbr0, addr)
    .ok_or(SyscallError::Fault)?;
```

### Helper Function (Optional)

If the pattern is verbose, consider a helper in `syscall/helpers.rs`:

```rust
/// Convert user VA to kernel pointer, returning EFAULT on failure.
/// 
/// # Safety
/// Caller must have validated the buffer with `validate_user_buffer` first.
pub fn user_ptr<T>(ttbr0: PhysAddr, addr: VirtAddr) -> SyscallResult<*mut T> {
    mm_user::user_va_to_kernel_ptr(ttbr0, addr)
        .map(|p| p as *mut T)
        .ok_or(SyscallError::Fault)
}
```

---

## Call Site Inventory

### Priority Order

Process syscalls first (most critical), then time, then filesystem.

| File | Function | Line | Pattern |
|------|----------|------|---------|
| `process.rs` | `sys_getrusage` | 1036 | `unwrap()` |
| `process.rs` | `sys_getrlimit` | 1122 | `unwrap()` |
| `time.rs` | `sys_clock_gettime` | 91 | `unwrap()` |
| `time.rs` | `sys_gettimeofday` | 146 | `unwrap()` |
| `time.rs` | `sys_clock_getres` | 189 | `unwrap()` |
| `sys.rs` | `sys_getrandom` | 129 | `unwrap()` |
| `fs/stat.rs` | `sys_fstat` | 82 | `unwrap()` |
| `fs/fd.rs` | `sys_pipe2` | 158 | `unwrap()` |
| `fs/fd.rs` | `sys_pread64` | 580 | `unwrap()` |
| `fs/fd.rs` | `sys_pwrite64` | 645 | `unwrap()` |
| `fs/dir.rs` | `sys_getcwd` | 117 | `unwrap()` |
| `fs/statx.rs` | `sys_statx` | 185 | `unwrap()` |
| `fs/read.rs` | `sys_readv` | 31 | `unwrap()` |
| `fs/write.rs` | `sys_writev` | 40,91,115,133,163 | `unwrap()` |

**Total**: 18 call sites

---

## Steps

### Step 1: Process Syscalls

**File**: `phase-2-step-1.md`

Replace `unwrap()` in:
- `src/syscall/process.rs:1036` (getrusage)
- `src/syscall/process.rs:1122` (getrlimit)

**UoW size**: 2 changes - fits in one session.

### Step 2: Time Syscalls

**File**: `phase-2-step-2.md`

Replace `unwrap()` in:
- `src/syscall/time.rs:91` (clock_gettime)
- `src/syscall/time.rs:146` (gettimeofday)
- `src/syscall/time.rs:189` (clock_getres)

**UoW size**: 3 changes - fits in one session.

### Step 3: System Syscalls

**File**: `phase-2-step-3.md`

Replace `unwrap()` in:
- `src/syscall/sys.rs:129` (getrandom)

**UoW size**: 1 change - fits in one session.

### Step 4: Filesystem Syscalls (stat/statx)

**File**: `phase-2-step-4.md`

Replace `unwrap()` in:
- `src/syscall/fs/stat.rs:82` (fstat)
- `src/syscall/fs/statx.rs:185` (statx)

**UoW size**: 2 changes - fits in one session.

### Step 5: Filesystem Syscalls (fd operations)

**File**: `phase-2-step-5.md`

Replace `unwrap()` in:
- `src/syscall/fs/fd.rs:158` (pipe2)
- `src/syscall/fs/fd.rs:580` (pread64)
- `src/syscall/fs/fd.rs:645` (pwrite64)

**UoW size**: 3 changes - fits in one session.

### Step 6: Filesystem Syscalls (dir)

**File**: `phase-2-step-6.md`

Replace `unwrap()` in:
- `src/syscall/fs/dir.rs:117` (getcwd)

**UoW size**: 1 change - fits in one session.

### Step 7: Filesystem Syscalls (read/write)

**File**: `phase-2-step-7.md`

Replace `unwrap()` in:
- `src/syscall/fs/read.rs:31` (readv)
- `src/syscall/fs/write.rs:40,91,115,133,163` (writev family)

**UoW size**: 6 changes - fits in one session.

---

## Rollback Plan

If migration causes issues:
1. Revert to `unwrap()` for specific syscall
2. Add `expect()` with context message instead
3. Document issue for further investigation

---

## Exit Criteria

- [x] All 18 `unwrap()` calls replaced (15 by TEAM_416, 3 already by TEAM_413)
- [x] Build passes
- [ ] Eyra behavior tests pass (not run - requires QEMU)
- [x] No new panics in syscall paths
