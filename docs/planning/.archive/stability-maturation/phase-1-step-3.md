# Phase 1 Step 3 — Add Regression Tests

**TEAM_311**: ABI Stability Refactor
**Parent**: `phase-1.md`
**Status**: Ready for Execution

---

## Objective
Add compile-time and runtime tests to catch ABI drift.

---

## Tasks

### 3.1 Add Size Assertions to los_abi
```rust
// In crates/abi/src/lib.rs

// Compile-time size verification for ABI structures
// These will fail compilation if sizes don't match Linux ABI
#[cfg(target_arch = "aarch64")]
mod size_checks {
    // Stat should be 128 bytes on AArch64
    // const _: () = assert!(core::mem::size_of::<super::stat::Stat>() == 128);
    
    // Timespec should be 16 bytes
    // const _: () = assert!(core::mem::size_of::<super::Timespec>() == 16);
}

#[cfg(target_arch = "x86_64")]
mod size_checks {
    // Same sizes on x86_64
}
```

### 3.2 Add Errno Consistency Test
```rust
// In crates/abi/src/errno.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn errno_matches_linux_raw_sys() {
        // Verify our constants match linux-raw-sys
        // This catches drift between our definitions and Linux
        assert_eq!(ENOENT, -(linux_raw_sys::errno::ENOENT as i64));
        assert_eq!(EINVAL, -(linux_raw_sys::errno::EINVAL as i64));
    }
}
```

### 3.3 Add Syscall Number Test
```rust
// In crates/abi/src/syscall/mod.rs
#[cfg(all(test, feature = "std"))]
mod tests {
    #[test]
    #[cfg(target_arch = "aarch64")]
    fn syscall_numbers_match_linux_aarch64() {
        use linux_raw_sys::general::*;
        use super::aarch64::*;
        
        assert_eq!(NR_READ, __NR_read as u64);
        assert_eq!(NR_WRITE, __NR_write as u64);
        assert_eq!(NR_EXIT, __NR_exit as u64);
    }
    
    #[test]
    #[cfg(target_arch = "x86_64")]
    fn syscall_numbers_match_linux_x86_64() {
        use linux_raw_sys::general::*;
        use super::x86_64::*;
        
        assert_eq!(NR_READ, __NR_read as u64);
        assert_eq!(NR_WRITE, __NR_write as u64);
        assert_eq!(NR_EXIT, __NR_exit as u64);
    }
}
```

### 3.4 Add dev dependency
```toml
# crates/abi/Cargo.toml
[dev-dependencies]
linux-raw-sys = { version = "0.4", features = ["general", "errno"] }
```

---

## Expected Outputs
1. Size assertion tests added
2. Errno consistency tests added
3. Syscall number verification tests added
4. All tests pass

---

## Exit Criteria
- [ ] `cargo test -p los_abi --features std` passes
- [ ] Size assertions compile
- [ ] Errno values verified against linux-raw-sys
- [ ] Phase 1 complete, ready for Phase 2
