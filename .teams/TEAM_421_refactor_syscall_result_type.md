# TEAM_421: SyscallResult Type - Eliminate Scattered Errno Casts

**Status:** COMPLETE
**Date:** 2026-01-11
**Type:** Refactor

## Summary

Introduced `SyscallResult = Result<i64, u32>` type alias to eliminate scattered `-(ERRNO as i64)` casts throughout syscall code. Error codes from `linux_raw_sys::errno` (type `u32`) now flow through directly, with a single conversion to Linux ABI at the dispatcher boundary.

## Problem

Before this refactor, every syscall function had scattered casts:
```rust
// BEFORE: Cast at every error site
pub fn sys_read(...) -> i64 {
    if error { return -(EBADF as i64); }
    if error { return -(EFAULT as i64); }
    // ... dozens more
}
```

This violated the principle: **The library IS the canonical source.**

## Solution

Single conversion point at ABI boundary:
```rust
// syscall/mod.rs
pub type SyscallResult = Result<i64, u32>;

// In dispatcher (THE ONLY CAST):
match result {
    Ok(v) => v,
    Err(e) => -(e as i64),  // Line 413
}

// AFTER: Clean error returns
pub fn sys_read(...) -> SyscallResult {
    if error { return Err(EBADF); }  // Direct use of u32 errno
    Ok(bytes_read)
}
```

## Changes

### Core Infrastructure
- `syscall/mod.rs`: Added `SyscallResult` type alias, single conversion in dispatcher
- `fs/vfs/error.rs`: `VfsError::to_errno()` returns `u32` directly
- `syscall/helpers.rs`: All helpers return `Result<T, u32>`

### Syscall Files Updated
All syscall functions now return `SyscallResult`:
- `fs/`: read.rs, write.rs, open.rs, stat.rs, statx.rs, fd.rs, dir.rs, link.rs, mount.rs
- `process/`: lifecycle.rs, thread.rs, identity.rs, arch_prctl.rs, groups.rs, resources.rs
- `mm.rs`, `sync.rs`, `signal.rs`, `time.rs`, `epoll.rs`, `sys.rs`

### Pipe API Refactored
- `fs/pipe.rs`: `Pipe::read()` and `Pipe::write()` now return `Result<usize, u32>` instead of `isize` with negative errno values

## Verification

```bash
# Only ONE errno cast in syscall code:
grep -rn "as i64)" crates/kernel/src/syscall/ | grep "-(e"
# Output: mod.rs:413:        Err(e) => -(e as i64),

# Both architectures build:
cargo xtask build kernel --arch x86_64   # OK
cargo xtask build kernel --arch aarch64  # OK
```

## Principle Upheld

**The library IS the canonical source. If types don't match, WE change.**

- `linux_raw_sys::errno` constants are `u32`
- `SyscallResult` error type is `u32`
- Single cast at ABI boundary where Linux requires `i64`
