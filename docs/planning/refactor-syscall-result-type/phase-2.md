# Phase 2 — Structural Design

**Refactor:** Syscall Result Type - Proper Error Handling
**Team:** TEAM_421
**Date:** 2026-01-10

---

## Type Alias Location

Add to `syscall/mod.rs`:

```rust
/// TEAM_421: Syscall result type
///
/// - Ok(i64): Success value (fd, count, address, etc.)
/// - Err(u16): Error code from linux_raw_sys::errno (positive, raw)
///
/// The dispatcher converts Err(e) to -(e as i64) for Linux ABI.
pub type SyscallResult = Result<i64, u16>;
```

---

## Dispatcher Change

### Before (WRONG)
```rust
pub fn syscall_dispatch(frame: &mut SyscallFrame) -> i64 {
    match SyscallNumber::from_raw(nr) {
        Some(SyscallNumber::Open) => fs::sys_openat(...),  // Returns i64 directly
        // ...
        None => -(linux_raw_sys::errno::ENOSYS as i64),
    }
}
```

### After (CORRECT)
```rust
pub fn syscall_dispatch(frame: &mut SyscallFrame) -> i64 {
    let result: SyscallResult = match SyscallNumber::from_raw(nr) {
        Some(SyscallNumber::Open) => fs::sys_openat(...),  // Returns SyscallResult
        // ...
        None => Err(linux_raw_sys::errno::ENOSYS),
    };

    // SINGLE conversion point to Linux ABI
    match result {
        Ok(v) => v,
        Err(e) => -(e as i64),
    }
}
```

---

## Syscall Function Pattern

### Before (WRONG)
```rust
pub fn sys_openat(dirfd: i32, pathname: usize, ...) -> i64 {
    if dirfd != AT_FDCWD && !path.starts_with('/') {
        return -(EBADF as i64);  // CAST HERE - BAD
    }

    match vfs_open(path, flags, mode) {
        Ok(file) => fd as i64,
        Err(VfsError::NotFound) => -(ENOENT as i64),  // CAST HERE - BAD
        Err(_) => -(EIO as i64),  // CAST HERE - BAD
    }
}
```

### After (CORRECT)
```rust
pub fn sys_openat(dirfd: i32, pathname: usize, ...) -> SyscallResult {
    use linux_raw_sys::errno::{EBADF, ENOENT, EIO};

    if dirfd != AT_FDCWD && !path.starts_with('/') {
        return Err(EBADF);  // DIRECT - GOOD
    }

    match vfs_open(path, flags, mode) {
        Ok(file) => Ok(fd as i64),
        Err(VfsError::NotFound) => Err(ENOENT),  // DIRECT - GOOD
        Err(_) => Err(EIO),  // DIRECT - GOOD
    }
}
```

---

## VfsError Conversion

The `VfsError::to_errno()` method currently returns `i64`.

### Before
```rust
impl VfsError {
    pub fn to_errno(self) -> i64 {
        match self {
            VfsError::NotFound => -(errno::ENOENT as i64),
            // ...
        }
    }
}
```

### After
```rust
impl VfsError {
    /// Convert to raw errno value (matches linux_raw_sys::errno type)
    pub fn to_errno(self) -> u16 {
        use linux_raw_sys::errno;
        match self {
            VfsError::NotFound => errno::ENOENT,
            VfsError::PermissionDenied => errno::EPERM,
            // ... all direct, no casts
        }
    }
}

// Usage in syscalls:
match vfs_operation() {
    Ok(v) => Ok(v),
    Err(e) => Err(e.to_errno()),  // u16 -> u16, no cast
}
```

---

## Helper Function Updates

### read_user_cstring

Before:
```rust
pub fn read_user_cstring(...) -> Result<&str, i64> {
    // ...
    return Err(-(EFAULT as i64));
}
```

After:
```rust
pub fn read_user_cstring(...) -> Result<&str, u16> {
    use linux_raw_sys::errno::EFAULT;
    // ...
    return Err(EFAULT);
}
```

---

## Pipe Error Returns

Pipe functions return `isize`, not `i64`. Keep separate:

```rust
impl Pipe {
    pub fn read(&self, buf: &mut [u8]) -> isize {
        // Still returns isize for internal use
        // Syscall wrapper converts to SyscallResult
    }
}
```

---

## Exit Criteria for Phase 2

- [ ] SyscallResult type defined
- [ ] Dispatcher pattern documented
- [ ] Syscall function pattern documented
- [ ] VfsError::to_errno() updated to return u16
- [ ] Helper function signatures updated

