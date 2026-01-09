# Phase 4: Implementation and Tests

**TEAM_339** | Linux ABI Compatibility Bugfix

## Overview

Execute the fix in batched changes, with tests after each batch.

## Implementation Batches

### Batch 0: Foundation (2 UoW)

**Prerequisites for all other batches.**

#### UoW 0.1: Add Safe String Helper
- Create `read_user_cstring()` in `crates/kernel/src/syscall/mod.rs`
- Add `ENAMETOOLONG` error code
- Add unit tests for edge cases

#### UoW 0.2: Add Linux Constants
- Add `AT_FDCWD` constant (-100)
- Verify `O_*` flags match Linux
- Add any missing mode constants

**Checkpoint:** Build succeeds

---

### Batch 1: Read-Only Syscalls (4 UoW)

**Lowest risk - cannot corrupt data.**

#### UoW 1.1: sys_openat (read mode)
- Change signature: `(dirfd, pathname, flags, mode)`
- Use `read_user_cstring()` for pathname
- Handle `AT_FDCWD` for dirfd
- Update userspace wrapper

#### UoW 1.2: sys_fstat
- Verify Stat struct matches Linux layout
- Update if needed

#### UoW 1.3: sys_getdents
- Verify Dirent64 struct matches Linux
- No signature change needed

#### UoW 1.4: sys_getcwd
- Verify return value semantics match Linux
- Linux returns pointer, not length

**Checkpoint:** Read operations work with Linux ABI

---

### Batch 2: Path Resolution Syscalls (5 UoW)

#### UoW 2.1: sys_readlinkat
- Change signature to Linux ABI
- Update userspace wrapper

#### UoW 2.2: sys_symlinkat
- Change signature to Linux ABI
- Update userspace wrapper

#### UoW 2.3: sys_linkat
- Change signature to Linux ABI
- Update userspace wrapper

#### UoW 2.4: sys_utimensat
- Change signature to Linux ABI
- Update userspace wrapper

#### UoW 2.5: sys_unlinkat
- Change signature to Linux ABI
- Update userspace wrapper

**Checkpoint:** All link operations work

---

### Batch 3: Directory Syscalls (3 UoW)

#### UoW 3.1: sys_mkdirat
- Change signature to Linux ABI
- Update userspace wrapper

#### UoW 3.2: sys_renameat
- Change signature to Linux ABI (most complex - 4 path args)
- Update userspace wrapper

#### UoW 3.3: sys_mount / sys_umount
- Change signatures to Linux ABI
- Update userspace wrappers

**Checkpoint:** Directory operations work

---

### Batch 4: Quick Fixes (3 UoW)

#### UoW 4.1: Fix __NR_pause for aarch64
- Remove hardcoded value
- Use architecture-conditional logic
- Add ppoll-based implementation for aarch64

#### UoW 4.2: Consolidate errno definitions
- Merge `errno` and `errno_file` modules
- Replace all magic numbers with constants
- Use `linux_raw_sys::errno` in kernel

#### UoW 4.3: Verify Termios struct
- Compare kernel Termios to Linux
- Fix any layout differences

**Checkpoint:** All quick fixes complete

---

### Batch 5: Struct Verification (2 UoW)

#### UoW 5.1: Stat struct alignment
- Write compile-time size assertion
- Compare field-by-field with linux_raw_sys::general::stat
- Fix any differences

#### UoW 5.2: Other struct verification
- Timespec
- iovec
- sigaction (if used)

**Checkpoint:** All structs match Linux

---

## Exit Criteria

- [ ] All batches implemented
- [ ] All checkpoints passed
- [ ] No test regressions
- [ ] Ready for Phase 5 cleanup
