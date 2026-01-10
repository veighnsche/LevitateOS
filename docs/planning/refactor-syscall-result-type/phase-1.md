# Phase 1 — Analysis

**Refactor:** Syscall Result Type - Proper Error Handling
**Team:** TEAM_421
**Date:** 2026-01-10

---

## Problem Statement

Current state is WRONG:
```rust
// Scattered casts everywhere - BAD
pub fn sys_open(...) -> i64 {
    return -(linux_raw_sys::errno::ENOENT as i64);  // CAST AT EVERY CALLSITE
}
```

Target state is CORRECT:
```rust
// Return the library type directly - GOOD
pub fn sys_open(...) -> Result<i64, u16> {
    return Err(linux_raw_sys::errno::ENOENT);  // NO CAST
}

// Single conversion point in dispatcher
fn dispatch(...) -> i64 {
    match sys_open(...) {
        Ok(v) => v,
        Err(e) => -(e as i64),  // ONE CAST, ONE PLACE
    }
}
```

---

## Principle

**The library IS the canonical source.**

- linux-raw-sys errno type: `u16`
- Our internal error type: `u16` (MATCH THE LIBRARY)
- Linux ABI return type: `i64` (only at boundary)

Conversion happens at the ABI boundary, NOT scattered throughout code.

---

## Type Definition

```rust
/// Syscall result type - matches linux-raw-sys errno
pub type SyscallResult = Result<i64, u16>;

// Usage in syscall implementations:
use linux_raw_sys::errno::ENOENT;
return Err(ENOENT);  // Direct, no cast
```

---

## Scope

### Files to Change

| Category | Files | Changes |
|----------|-------|---------|
| Dispatcher | `syscall/mod.rs` | Add Result handling, single negation |
| FS syscalls | `syscall/fs/*.rs` (8 files) | Return `SyscallResult` |
| Process syscalls | `syscall/process/*.rs` (6 files) | Return `SyscallResult` |
| Memory syscalls | `syscall/mm.rs` | Return `SyscallResult` |
| Sync syscalls | `syscall/sync.rs` | Return `SyscallResult` |
| Signal syscalls | `syscall/signal.rs` | Return `SyscallResult` |
| Time syscalls | `syscall/time.rs` | Return `SyscallResult` |
| Epoll syscalls | `syscall/epoll.rs` | Return `SyscallResult` |
| Other | `syscall/sys.rs`, `syscall/helpers.rs` | Return `SyscallResult` |

### Estimated Changes

- ~50 function signatures
- ~300 return statements (Err(ERRNO) instead of -(ERRNO as i64))
- 1 dispatcher update

---

## Exit Criteria for Phase 1

- [ ] Problem clearly stated
- [ ] Target type defined (`Result<i64, u16>`)
- [ ] All affected files identified
- [ ] Principle documented: library type is canonical

