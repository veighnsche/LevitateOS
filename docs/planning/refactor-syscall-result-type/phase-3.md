# Phase 3 — Migration

**Refactor:** Syscall Result Type - Proper Error Handling
**Team:** TEAM_421
**Date:** 2026-01-10

---

## Migration Strategy

**"Let the compiler scream. Fix every callsite."**

No gradual migration. Change the type, fix all errors.

---

## Migration Order

### Step 1: Add SyscallResult type

In `syscall/mod.rs`:
```rust
/// TEAM_421: Syscall result type - uses linux_raw_sys errno directly
pub type SyscallResult = Result<i64, u16>;
```

### Step 2: Update VfsError::to_errno()

In `fs/vfs/error.rs`:
```rust
pub fn to_errno(self) -> u16 {  // Was i64
    use linux_raw_sys::errno;
    match self {
        VfsError::NotFound => errno::ENOENT,  // Direct, no negation
        // ...
    }
}
```

### Step 3: Update helper functions

In `syscall/mod.rs` and `syscall/helpers.rs`:
```rust
pub fn read_user_cstring(...) -> Result<&str, u16> { ... }
pub fn copy_user_string(...) -> Result<&str, u16> { ... }
```

### Step 4: Update each syscall file

For each file, change:
1. Function signature: `-> i64` to `-> SyscallResult`
2. Success returns: `value` to `Ok(value)`
3. Error returns: `-(ERRNO as i64)` to `Err(ERRNO)`

Order (dependencies first):
1. `syscall/helpers.rs` - used by others
2. `syscall/fs/open.rs` - foundational
3. `syscall/fs/read.rs`
4. `syscall/fs/write.rs`
5. `syscall/fs/fd.rs`
6. `syscall/fs/dir.rs`
7. `syscall/fs/link.rs`
8. `syscall/fs/stat.rs`
9. `syscall/fs/statx.rs`
10. `syscall/fs/mount.rs`
11. `syscall/mm.rs`
12. `syscall/process/lifecycle.rs`
13. `syscall/process/thread.rs`
14. `syscall/process/groups.rs`
15. `syscall/process/resources.rs`
16. `syscall/process/identity.rs`
17. `syscall/process/arch_prctl.rs`
18. `syscall/sync.rs`
19. `syscall/signal.rs`
20. `syscall/time.rs`
21. `syscall/epoll.rs`
22. `syscall/sys.rs`

### Step 5: Update dispatcher

Change every match arm to expect SyscallResult:
```rust
Some(SyscallNumber::Openat) => fs::sys_openat(...),
```

Add final conversion:
```rust
match result {
    Ok(v) => v,
    Err(e) => -(e as i64),
}
```

---

## Example Transformation

### Before (fs/open.rs)
```rust
pub fn sys_openat(dirfd: i32, pathname: usize, flags: u32, _mode: u32) -> i64 {
    let task = crate::task::current_task();
    let mut path_buf = [0u8; PATH_MAX as usize];
    let path_str = match read_user_cstring(task.ttbr0, pathname, &mut path_buf) {
        Ok(s) => s,
        Err(e) => return e,  // e is already i64
    };

    if dirfd != AT_FDCWD && !path_str.starts_with('/') {
        return -(EBADF as i64);
    }

    match vfs_open(path_str, vfs_flags, 0o666) {
        Ok(file) => {
            match fd_table.alloc(FdType::VfsFile(file)) {
                Some(fd) => fd as i64,
                None => -(EMFILE as i64),
            }
        }
        Err(VfsError::NotFound) => -(ENOENT as i64),
        Err(_) => -(EIO as i64),
    }
}
```

### After (fs/open.rs)
```rust
use crate::syscall::SyscallResult;
use linux_raw_sys::errno::{EBADF, EMFILE, ENOENT, EIO};

pub fn sys_openat(dirfd: i32, pathname: usize, flags: u32, _mode: u32) -> SyscallResult {
    let task = crate::task::current_task();
    let mut path_buf = [0u8; PATH_MAX as usize];
    let path_str = match read_user_cstring(task.ttbr0, pathname, &mut path_buf) {
        Ok(s) => s,
        Err(e) => return Err(e),  // e is u16
    };

    if dirfd != AT_FDCWD && !path_str.starts_with('/') {
        return Err(EBADF);
    }

    match vfs_open(path_str, vfs_flags, 0o666) {
        Ok(file) => {
            match fd_table.alloc(FdType::VfsFile(file)) {
                Some(fd) => Ok(fd as i64),
                None => Err(EMFILE),
            }
        }
        Err(VfsError::NotFound) => Err(ENOENT),
        Err(_) => Err(EIO),
    }
}
```

---

## Verification

After migration:
```bash
# Build both architectures
cargo xtask build kernel --arch x86_64
cargo xtask build kernel --arch aarch64

# No casts should appear in syscall returns
grep -rn "as i64)" crates/kernel/src/syscall/ | grep -v "Ok("
# Should only show the ONE cast in the dispatcher
```

---

## Exit Criteria for Phase 3

- [ ] SyscallResult type added
- [ ] VfsError::to_errno() returns u16
- [ ] All helper functions return u16 for errors
- [ ] All syscall functions return SyscallResult
- [ ] Dispatcher handles Result with single conversion
- [ ] Both architectures compile
- [ ] grep verification passes

