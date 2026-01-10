# Phase 1 Step 2 — Create crates/abi Foundation

**TEAM_311**: ABI Stability Refactor
**Parent**: `phase-1.md`
**Status**: Ready for Execution

---

## Objective
Create the `crates/abi` crate skeleton with Cargo.toml and basic module structure.

---

## Tasks

### 2.1 Create Cargo.toml
```toml
# crates/abi/Cargo.toml
[package]
name = "los_abi"
version = "0.1.0"
edition = "2021"
description = "LevitateOS ABI definitions - single source of truth for kernel/userspace"

[features]
default = []
std = []

[dependencies]
# No dependencies - pure definitions
```

### 2.2 Create lib.rs
```rust
// crates/abi/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

//! LevitateOS ABI Definitions
//!
//! This crate is the **single source of truth** for the kernel/userspace ABI.
//! Both kernel and userspace import types from here to ensure consistency.

pub mod errno;
pub mod syscall;
// pub mod stat;      // Phase 2
// pub mod termios;   // Phase 2
// pub mod flags;     // Phase 2
```

### 2.3 Create errno.rs
```rust
// crates/abi/src/errno.rs
//! Linux-compatible error codes.
//!
//! All values are negative as returned by syscalls.

pub const EPERM: i64 = -1;
pub const ENOENT: i64 = -2;
pub const EIO: i64 = -5;
pub const EBADF: i64 = -9;
pub const EAGAIN: i64 = -11;
pub const ENOMEM: i64 = -12;
pub const EACCES: i64 = -13;
pub const EFAULT: i64 = -14;
pub const EEXIST: i64 = -17;
pub const ENOTDIR: i64 = -20;
pub const EINVAL: i64 = -22;
pub const EMFILE: i64 = -24;
pub const ENOTTY: i64 = -25;
pub const ENOSYS: i64 = -38;

// Compile-time verification
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn errno_values_match_linux() {
        assert_eq!(ENOENT, -2);
        assert_eq!(EINVAL, -22);
        assert_eq!(ENOSYS, -38);
    }
}
```

### 2.4 Create syscall/mod.rs (placeholder)
```rust
// crates/abi/src/syscall/mod.rs
//! Syscall number definitions.
//!
//! Architecture-specific syscall numbers following Linux ABI.

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

// Re-export current architecture
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

#[cfg(target_arch = "x86_64")]
pub use x86_64::*;
```

### 2.5 Add to Workspace
Update root `Cargo.toml`:
```toml
members = [
    # ... existing
    "crates/abi",
]
```

---

## Expected Outputs
1. `crates/abi/Cargo.toml` created
2. `crates/abi/src/lib.rs` created
3. `crates/abi/src/errno.rs` created
4. `crates/abi/src/syscall/mod.rs` created
5. Workspace updated

---

## Exit Criteria
- [ ] `cargo build -p los_abi` succeeds
- [ ] `cargo test -p los_abi --features std` passes
- [ ] Ready to proceed to Step 3
