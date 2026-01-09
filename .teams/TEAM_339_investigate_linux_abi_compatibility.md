# TEAM_339: Investigate Linux ABI/API Compatibility

**Date:** 2026-01-09  
**Status:** ✅ INVESTIGATION COMPLETE → BUGFIX PLAN CREATED  
**Type:** Investigation (Bug Hunt)

## Objective

Investigate whether LevitateOS is Linux-compatible in terms of:
- Syscall numbers
- Syscall signatures (ABI)
- Error codes (errno)
- Struct layouts (Stat, Termios, etc.)

## Summary

**LevitateOS is NOT Linux-compatible.** It uses Linux syscall *numbers* but has custom syscall *signatures* that differ from Linux ABI. This creates an OS-specific ABI that won't work with standard Linux binaries.

---

# Phase 1: Symptom Analysis

## Expected Behavior (Linux ABI)
- Syscall numbers match Linux per-architecture tables
- Syscall signatures match Linux exactly (argument order, types)
- Struct layouts match Linux exactly (size, alignment, field order)
- Error codes are standard Linux errno values

## Actual Behavior (LevitateOS)
- ✅ Syscall numbers: Mostly Linux-compatible
- ❌ Syscall signatures: **CUSTOM, NOT LINUX**
- ⚠️ Struct layouts: Mixed (some match, some don't)
- ✅ Error codes: Linux-compatible (via `linux_raw_sys`)

---

# Phase 2: Discrepancies Found

## CRITICAL: Syscall Signature Differences

### 1. `openat` - Missing dirfd argument

| Component | Linux Signature | LevitateOS Signature |
|-----------|-----------------|----------------------|
| Kernel | `openat(dirfd, path, flags, mode)` | `sys_openat(path, path_len, flags)` |
| Userspace | `openat(__NR_openat, path_ptr, path_len, flags)` | Same |

**Impact:** Cannot open relative paths with AT_FDCWD, no mode parameter for file creation permissions.

**Location:** 
- `crates/kernel/src/syscall/fs/open.rs:12`
- `crates/userspace/libsyscall/src/fs.rs:36-43`

### 2. `mkdirat` - Path length passed instead of null-terminated

| Component | Linux Signature | LevitateOS Signature |
|-----------|-----------------|----------------------|
| Kernel | `mkdirat(dirfd, pathname, mode)` | `sys_mkdirat(dfd, path, path_len, mode)` |
| Userspace | `mkdirat(__NR_mkdirat, dfd, path_ptr, path_len, mode)` | Same |

**Impact:** Cannot use standard libc, requires length-counted strings.

**Location:**
- `crates/kernel/src/syscall/fs/dir.rs:130`
- `crates/userspace/libsyscall/src/fs.rs:74-82`

### 3. `unlinkat` - Path length passed

| Component | Linux Signature | LevitateOS Signature |
|-----------|-----------------|----------------------|
| Kernel | `unlinkat(dirfd, pathname, flags)` | `sys_unlinkat(dfd, path, path_len, flags)` |

**Location:** `crates/kernel/src/syscall/fs/dir.rs:149`

### 4. `renameat` - Path lengths for both paths

| Component | Linux Signature | LevitateOS Signature |
|-----------|-----------------|----------------------|
| Kernel | `renameat(olddirfd, oldpath, newdirfd, newpath)` | `sys_renameat(old_dfd, old_path, old_path_len, new_dfd, new_path, new_path_len)` |

**Location:** `crates/kernel/src/syscall/fs/dir.rs:177`

### 5. `symlinkat`, `readlinkat`, `linkat` - All use length-counted strings

Same pattern: LevitateOS passes explicit length parameters.

### 6. `getcwd` - Different semantics

| Component | Linux Signature | LevitateOS Signature |
|-----------|-----------------|----------------------|
| Linux | Returns pointer to buf on success | Returns length on success |

---

## HIGH: Architecture-Specific Syscall Number Bug

### `__NR_pause` hardcoded for x86_64 only

```rust
// crates/userspace/libsyscall/src/sysno.rs:58
pub const __NR_pause: u32 = 34; // Verify arch support later
```

**Problem:** 
- x86_64 pause = 34 ✅
- aarch64 **does not have pause** - uses `ppoll(NULL, 0, NULL, NULL)` instead
- The comment "Verify arch support later" was never verified

**Location:** `crates/userspace/libsyscall/src/sysno.rs:58`

---

## MEDIUM: Struct Layout Discrepancies

### `Stat` Structure

| Source | Type |
|--------|------|
| Kernel x86_64 | Custom `Stat` struct (144 bytes with padding) |
| Kernel aarch64 | Custom `Stat` struct (same layout) |
| Userspace | `linux_raw_sys::general::stat` (Linux-compatible) |

**Problem:** The kernel's `Stat` and userspace's `stat` may have different layouts. If kernel writes to userspace buffer with its struct layout, and userspace reads with Linux layout, data corruption occurs.

**Locations:**
- `crates/kernel/src/arch/x86_64/mod.rs:147`
- `crates/kernel/src/arch/aarch64/mod.rs:145`
- `crates/userspace/libsyscall/src/fs.rs:22` - uses `linux_raw_sys::general::stat`

### `Termios` Structure

Both kernel and userspace define Termios, but:
- Kernel: Custom struct in `arch/*/mod.rs`
- Userspace: Uses Linux ioctl codes but passes raw pointers

Need to verify struct sizes match.

---

## LOW: Duplicate Error Code Definitions

### Kernel has two errno modules

```rust
// crates/kernel/src/syscall/mod.rs:14-33
pub mod errno {
    pub const ENOENT: i64 = -2;
    pub const EBADF: i64 = -9;
    // ...
}

pub mod errno_file {
    pub const ENOENT: i64 = -2;
    pub const EMFILE: i64 = -24;
    // ...
}
```

**Problem:** Duplicated constants, easy to get out of sync. Some syscalls use hardcoded values like `-34` (ERANGE) instead of named constants.

**Location:** `crates/kernel/src/syscall/mod.rs:14-33`

---

## INFO: What IS Linux-Compatible

### ✅ Syscall Numbers
Both x86_64 and aarch64 use correct Linux syscall numbers from `linux_raw_sys`.

### ✅ Error Codes (Userspace)
Userspace uses `linux_raw_sys::errno::*` which is Linux-compatible.

### ✅ Syscall Invocation ABI
The inline assembly in `arch/aarch64.rs` and `arch/x86_64.rs` correctly uses:
- aarch64: x8 for syscall number, x0-x5 for args, svc #0
- x86_64: rax for syscall number, rdi/rsi/rdx/r10/r8/r9 for args

### ✅ Open Flags, File Types
Uses `linux_raw_sys::general::O_*`, `DT_*`, etc.

---

# Phase 3: Root Cause Analysis

## Why Is This Happening?

LevitateOS made a **deliberate design decision** to use length-counted strings instead of null-terminated strings for path arguments. This is evident from the pattern across all path-handling syscalls.

**Possible reasons:**
1. Safety: Length-counted strings prevent buffer overruns
2. Simplicity: No need to scan for null terminator
3. Rust-native: Rust strings are length-counted internally

**However**, this breaks Linux ABI compatibility and means:
- Standard Linux ELF binaries won't work
- Cannot use glibc/musl
- Must use custom LevitateOS-specific userspace

---

# Phase 4: Recommendations

## Option A: Full Linux Compatibility (Major Refactor)

Change all syscall signatures to match Linux exactly:
- Accept null-terminated strings
- Add proper dirfd support
- Match struct layouts exactly

**Effort:** ~20-30 UoW
**Risk:** High (touching all syscalls)
**Benefit:** Can run unmodified Linux binaries

## Option B: Document as LevitateOS ABI (Minimal Change)

Accept that LevitateOS has its own ABI:
- Document the differences clearly
- Consider it a "Linux-like" but not "Linux-compatible" OS
- Continue building custom userspace

**Effort:** 1-2 UoW (documentation only)
**Risk:** Low
**Benefit:** No code changes needed

## Option C: Hybrid Approach

Add a compatibility shim layer:
- Keep internal LevitateOS syscalls as-is
- Add wrapper syscalls that translate Linux ABI to LevitateOS ABI
- Use syscall number ranges (Linux: 0-999, LevitateOS: 1000+)

**Effort:** ~10-15 UoW
**Risk:** Medium
**Benefit:** Gradual migration path

---

# Phase 5: Decision Required

**This investigation reveals a fundamental architecture question:**

> **Is LevitateOS intended to be Linux binary-compatible?**

If YES → Major refactor needed (Option A or C)
If NO → Document and proceed (Option B)

**Recommendation:** Create a question file for user decision.

---

## Files With Discrepancies

| File | Issue |
|------|-------|
| `crates/kernel/src/syscall/fs/open.rs` | Non-Linux openat signature |
| `crates/kernel/src/syscall/fs/dir.rs` | Non-Linux mkdirat/unlinkat/renameat |
| `crates/kernel/src/syscall/fs/link.rs` | Non-Linux symlinkat/linkat/readlinkat |
| `crates/userspace/libsyscall/src/sysno.rs` | Hardcoded x86_64-only __NR_pause |
| `crates/kernel/src/arch/*/mod.rs` | Custom Stat struct (may differ from linux_raw_sys) |
| `crates/kernel/src/syscall/mod.rs` | Duplicate errno definitions |

## Verification Commands

```bash
# Check Stat size matches
cargo build -p levitate-kernel --target x86_64-unknown-none
# Then in debugger: print sizeof(Stat)

# Check syscall signature audit
grep -n "pub fn sys_" crates/kernel/src/syscall/**/*.rs | head -50
```

---

## Related Teams

- TEAM_168: Original openat implementation
- TEAM_176: getdents implementation
- TEAM_192: Directory syscalls
- TEAM_210: Syscall number definitions
- TEAM_258: x86_64 syscall numbers
- TEAM_310: linux-raw-sys integration
