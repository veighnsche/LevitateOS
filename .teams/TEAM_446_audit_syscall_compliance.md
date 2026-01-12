# TEAM_446: Audit Syscall Linux ABI Compliance

## Objective
Verify that implemented syscalls conform to Linux ABI specifications.
Focus on critical shell-related syscalls that are marked as "implemented".

## Summary

**2 BUGS FOUND AND FIXED:**
1. **x86_64 Stat struct has wrong layout** - FIXED: Created architecture-specific Stat structs
2. **waitpid status encoding wrong** - FIXED: Now encodes as `(exit_code & 0xFF) << 8`

## Detailed Findings

### BUG 1: x86_64 Stat struct layout (CRITICAL)

**Location:** `crates/kernel/lib/types/src/lib.rs:66`

**Problem:** LevitateOS uses a single `Stat` struct for both architectures, but Linux x86_64 has a DIFFERENT layout than aarch64:

| Field | aarch64 offset | x86_64 offset | LevitateOS |
|-------|---------------|---------------|------------|
| st_dev | 0 | 0 | 0 ✅ |
| st_ino | 8 | 8 | 8 ✅ |
| st_mode | 16 | **24** | 16 ❌ |
| st_nlink | 20 | **16** | 20 ❌ |
| st_uid | 24 | 28 | 24 ❌ |
| st_gid | 28 | 32 | 28 ❌ |
| **Total size** | 128 | **144** | 128 ❌ |

**Key difference:** On x86_64, `st_nlink` comes BEFORE `st_mode` and is 8 bytes, not 4!

**Impact:** Any program calling `stat()` on x86_64 will get corrupted data. Fields will be misaligned.

**Fix required:**
- Create arch-specific `Stat` structs
- x86_64: `[st_dev, st_ino, st_nlink(u64), st_mode, st_uid, st_gid, __pad0, st_rdev, ...]`
- aarch64: Keep current layout

**References:**
- [Linux x86_64 stat.h](https://github.com/torvalds/linux/blob/master/arch/x86/include/uapi/asm/stat.h)
- [Linux asm-generic stat.h](https://github.com/torvalds/linux/blob/master/include/uapi/asm-generic/stat.h)

---

### BUG 2: waitpid status encoding (MEDIUM)

**Location:** `crates/kernel/syscall/src/process/lifecycle.rs:101`

**Problem:** `write_exit_status()` writes raw `exit_code` directly:
```rust
unsafe { *(ptr as *mut i32) = exit_code; }  // WRONG!
```

Linux encodes the status as `exit_code << 8` for normal exits:
- Bits 0-6: Signal number (if terminated by signal)
- Bit 7: Core dump flag
- Bits 8-15: Exit status

**Impact:**
- `WIFEXITED(status)` returns false for normal exits
- `WEXITSTATUS(status)` returns 0 for any exit code
- Shell `$?` variable broken

**Fix required:**
```rust
// For normal exit:
let encoded_status = (exit_code & 0xFF) << 8;
unsafe { *(ptr as *mut i32) = encoded_status; }

// For signal termination:
let encoded_status = sig_num & 0x7F;  // + 0x80 if core dumped
```

**References:**
- [Linux wait(2) man page](https://man7.org/linux/man-pages/man2/wait.2.html)

---

## Items Verified Correct ✅

### Termios struct (60 bytes)
- Layout matches Linux glibc exactly
- NCCS = 32 ✅
- Field offsets match ✅
- c_ispeed/c_ospeed at correct positions ✅

### Sigaction struct
- Architecture-specific layouts implemented correctly
- x86_64: 32 bytes with sa_restorer ✅
- aarch64: 24 bytes without sa_restorer ✅

### dup/dup2/dup3
- Return new fd on success ✅
- dup2 returns newfd if oldfd == newfd (correct special case) ✅
- dup3 returns EINVAL if oldfd == newfd ✅

### pipe2
- Returns 0 on success ✅
- Writes [read_fd, write_fd] to user array ✅

### errno values
- Using `linux_raw_sys::errno` crate ✅
- Values match Linux kernel exactly ✅

---

## Breadcrumbs Placed

```rust
// TEAM_446 BREADCRUMB: CONFIRMED - x86_64 Stat struct uses wrong layout
// File: crates/kernel/lib/types/src/lib.rs:66
// Issue: Uses 128-byte aarch64 layout, should be 144-byte x86_64 layout

// TEAM_446 BREADCRUMB: CONFIRMED - waitpid status not encoded
// File: crates/kernel/syscall/src/process/lifecycle.rs:101
// Issue: Writes raw exit_code, should be (exit_code << 8)
```

---

## Recommendation

**Priority 1 (High):** Fix x86_64 Stat struct
- Affects ALL programs using stat/fstat on x86_64
- Will cause silent data corruption

**Priority 2 (Medium):** Fix waitpid status encoding
- Affects shell $? variable
- Affects any program checking child exit status

---

## Session Log

### 2026-01-12 (Investigation)
- Compared struct layouts using C sizeof/offsetof
- Found x86_64 Stat struct mismatch (144 vs 128 bytes)
- Found waitpid status encoding bug
- Verified termios, sigaction, errno are correct
- Documented findings and placed breadcrumbs

### 2026-01-12 (Fixes Applied)
**Bug 1 Fix - x86_64 Stat struct:**
- Created architecture-specific `Stat` structs in `lib/types/src/lib.rs`
- aarch64: 128-byte asm-generic layout (unchanged)
- x86_64: 144-byte layout with st_nlink (u64) before st_mode
- Added compile-time size assertions for both architectures
- Updated API: `new_pipe()` and `from_inode_data()` now take i64 for blksize
- Updated all callers: `stat.rs`, `statx.rs`, `inode.rs`

**Bug 2 Fix - waitpid status encoding:**
- Modified `write_exit_status()` in `lifecycle.rs`
- Now encodes status as `(exit_code & 0xFF) << 8`
- `WIFEXITED()` and `WEXITSTATUS()` macros will now work correctly

**Additional fixes found during build:**
- Added `#[cfg(target_arch = "x86_64")]` to `SyscallNumber::Open` dispatch
- Added `#[cfg(target_arch = "x86_64")]` to `SyscallNumber::Socketpair` dispatch
- Made `write_to_user_buf()` public for levitate crate access

**Files modified:**
- `lib/types/src/lib.rs` - Architecture-specific Stat structs
- `syscall/src/process/lifecycle.rs` - waitpid status encoding
- `syscall/src/fs/stat.rs` - Updated new_pipe call
- `syscall/src/fs/statx.rs` - Updated new_pipe call and stat_to_statx casts
- `syscall/src/lib.rs` - Architecture-specific syscall dispatch, pub visibility
- `vfs/src/inode.rs` - Updated blksize cast

**Build verification:**
- x86_64: ✅ Builds successfully
- aarch64: ✅ Builds successfully
