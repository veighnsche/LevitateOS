# Phase 2: Structural Extraction

## Target Design

### New Module Layout

```
crates/kernel/src/syscall/
├── mod.rs              # Existing dispatcher + re-exports
├── types.rs            # NEW: Stat, Timespec, Termios, UserIoVec
├── constants.rs        # NEW: All fcntl/AT_*, ioctl, termios constants
├── util.rs             # NEW: copy_user_buffer(), validate_dirfd_path()
├── errno.rs            # MOVE: errno constants from mod.rs::errno
├── fs/
│   ├── read.rs         # MODIFY: Use types::UserIoVec
│   ├── write.rs        # MODIFY: Use types::UserIoVec, util::copy_user_buffer
│   ├── open.rs         # MODIFY: Use util::validate_dirfd_path
│   ├── link.rs         # MODIFY: Use util::validate_dirfd_path
│   └── statx.rs        # MODIFY: Remove duplicate AT_EMPTY_PATH
└── ...

crates/kernel/src/arch/
├── aarch64/mod.rs      # MODIFY: Re-export from syscall::types
└── x86_64/mod.rs       # MODIFY: Re-export from syscall::types
```

### New Files Content

#### `syscall/types.rs`
```rust
//! TEAM_407: Shared ABI types for syscalls.
//!
//! These types are architecture-independent for aarch64/x86_64 Linux ABI.
//! They are re-exported from arch modules for backward compatibility.

use crate::fs::mode::S_IFIFO;

/// Linux-compatible Stat structure (128 bytes).
/// Matches both AArch64 and x86_64 asm-generic layout.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Stat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    pub __pad1: u64,
    pub st_size: i64,
    pub st_blksize: i32,
    pub __pad2: i32,
    pub st_blocks: i64,
    pub st_atime: i64,
    pub st_atime_nsec: u64,
    pub st_mtime: i64,
    pub st_mtime_nsec: u64,
    pub st_ctime: i64,
    pub st_ctime_nsec: u64,
    pub __unused: [u32; 2],
}

impl Stat {
    pub fn new_device(mode: u32, rdev: u64) -> Self { ... }
    pub fn new_pipe(blksize: i32) -> Self { ... }
    pub fn new_file(ino: u64, mode: u32, size: i64, blocks: i64, blksize: i32) -> Self { ... }
    pub fn from_inode_data(...) -> Self { ... }
}

/// Linux-compatible Timespec.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

/// Number of control characters in termios.
pub const NCCS: usize = 32;

/// Termios structure (matches Linux layout).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Termios { ... }

/// struct iovec for writev/readv syscalls.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UserIoVec {
    pub base: usize,
    pub len: usize,
}
```

#### `syscall/constants.rs`
```rust
//! TEAM_407: Consolidated Linux ABI constants.

// === fcntl constants ===
pub const AT_FDCWD: i32 = -100;
pub const AT_SYMLINK_NOFOLLOW: u32 = 0x100;
pub const AT_REMOVEDIR: u32 = 0x200;
pub const AT_SYMLINK_FOLLOW: u32 = 0x400;
pub const AT_NO_AUTOMOUNT: u32 = 0x800;
pub const AT_EMPTY_PATH: u32 = 0x1000;

// === File control ===
pub const F_DUPFD: i32 = 0;
pub const F_GETFD: i32 = 1;
pub const F_SETFD: i32 = 2;
pub const F_GETFL: i32 = 3;
pub const F_SETFL: i32 = 4;
pub const F_SETPIPE_SZ: i32 = 1031;
pub const F_GETPIPE_SZ: i32 = 1032;

// === Seek constants ===
pub const SEEK_SET: i32 = 0;
pub const SEEK_CUR: i32 = 1;
pub const SEEK_END: i32 = 2;

// === Access mode ===
pub const F_OK: i32 = 0;
pub const X_OK: i32 = 1;
pub const W_OK: i32 = 2;
pub const R_OK: i32 = 4;

// === Time constants ===
pub const UTIME_NOW: u64 = 0x3FFFFFFF;
pub const UTIME_OMIT: u64 = 0x3FFFFFFE;

// === Termios local mode flags (c_lflag) ===
pub const ISIG: u32 = 0x01;
pub const ICANON: u32 = 0x02;
pub const ECHO: u32 = 0x08;
// ... (all termios constants)

// === ioctl requests ===
pub const TCGETS: u64 = 0x5401;
pub const TCSETS: u64 = 0x5402;
// ... (all ioctl constants)
```

#### `syscall/util.rs`
```rust
//! TEAM_407: Syscall utility functions.

use crate::memory::user as mm_user;
use crate::syscall::errno;
use alloc::vec::Vec;

/// Copy buffer from user space to kernel.
/// Returns kernel buffer on success, errno on failure.
pub fn copy_user_buffer(ttbr0: usize, user_buf: usize, len: usize) -> Result<Vec<u8>, i64> {
    if mm_user::validate_user_buffer(ttbr0, user_buf, len, false).is_err() {
        return Err(errno::EFAULT);
    }
    let mut kbuf = alloc::vec![0u8; len];
    for i in 0..len {
        if let Some(ptr) = mm_user::user_va_to_kernel_ptr(ttbr0, user_buf + i) {
            kbuf[i] = unsafe { *ptr };
        } else {
            return Err(errno::EFAULT);
        }
    }
    Ok(kbuf)
}

/// Validate dirfd and path combination.
/// Returns Ok(()) if valid, Err(errno) if not yet supported.
pub fn validate_dirfd_path(dirfd: i32, path: &str, syscall_name: &str) -> Result<(), i64> {
    use crate::syscall::constants::AT_FDCWD;
    if dirfd != AT_FDCWD && !path.starts_with('/') {
        log::warn!("[SYSCALL] {}: dirfd {} not yet supported for relative paths", syscall_name, dirfd);
        return Err(errno::EBADF);
    }
    Ok(())
}
```

## Extraction Strategy

### Order of Extraction

1. **Create `syscall/types.rs`** with Stat, Timespec, Termios, UserIoVec
   - Copy from aarch64/mod.rs (canonical source)
   - Include all impl blocks and constructors

2. **Create `syscall/constants.rs`** with all Linux ABI constants
   - Merge from: arch modules, fcntl module, fd.rs, open.rs, link.rs, statx.rs

3. **Create `syscall/util.rs`** with helper functions
   - Extract copy_user_buffer pattern from write.rs
   - Extract validate_dirfd_path pattern from open.rs/link.rs

4. **Update arch modules** to re-export from types
   - Keep SyscallNumber, SyscallFrame (architecture-specific)
   - Re-export: `pub use crate::syscall::types::{Stat, Timespec, Termios, NCCS};`
   - Re-export constants: `pub use crate::syscall::constants::*;`

5. **Update syscall handlers** to use new modules
   - read.rs, write.rs: use types::UserIoVec
   - write.rs: use util::copy_user_buffer
   - open.rs, link.rs: use util::validate_dirfd_path
   - statx.rs: remove duplicate constant

### Coexistence During Migration

The re-export pattern allows gradual migration:
- Old imports like `use crate::arch::Stat` continue to work
- New code can use `use crate::syscall::types::Stat` directly
- No breaking changes to external callers

## Module Size Guidelines (Rule 7)

| Module | Target Lines | Justification |
|--------|--------------|---------------|
| types.rs | ~200 | 4 structs + impls, cohesive |
| constants.rs | ~100 | Pure constants, no logic |
| util.rs | ~50 | 2-3 helper functions |
| arch/*/mod.rs | Reduced by ~250 each | Structs moved out |
