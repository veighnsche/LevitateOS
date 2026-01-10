# Phase 3: Migration

## Migration Order

### Step 1: Create New Modules (No Breaking Changes)

1. Create `syscall/types.rs` with all shared types
2. Create `syscall/constants.rs` with all constants
3. Create `syscall/util.rs` with helper functions
4. Update `syscall/mod.rs` to declare new modules

### Step 2: Update Arch Modules (Re-export Pattern)

**aarch64/mod.rs changes:**
```rust
// REMOVE: Lines 231-462 (Stat, Timespec, Termios, constants)
// ADD: Re-exports
pub use crate::syscall::types::{Stat, Timespec, Termios, NCCS};
pub use crate::syscall::constants::{
    ISIG, ICANON, ECHO, ECHOE, ECHOK, ECHONL, NOFLSH, TOSTOP, IEXTEN,
    OPOST, ONLCR,
    VINTR, VQUIT, VERASE, VKILL, VEOF, VTIME, VMIN, VSTART, VSTOP, VSUSP,
    TCGETS, TCSETS, TCSETSW, TCSETSF,
    TIOCGPTN, TIOCSPTLCK, TIOCGWINSZ, TIOCSWINSZ,
};
```

**x86_64/mod.rs changes:**
```rust
// REMOVE: Lines 233-456 (Stat, Timespec, Termios, constants)
// ADD: Re-exports (same as aarch64)
pub use crate::syscall::types::{Stat, Timespec, Termios, NCCS};
pub use crate::syscall::constants::{...};
```

### Step 3: Update Syscall Handlers

**fs/read.rs:**
```rust
// REMOVE: Lines 57-63 (UserIoVec definition)
// ADD: Import
use crate::syscall::types::UserIoVec;
```

**fs/write.rs:**
```rust
// REMOVE: Lines 8-14 (UserIoVec definition)
// ADD: Imports
use crate::syscall::types::UserIoVec;
use crate::syscall::util::copy_user_buffer;

// REPLACE: 4 occurrences of buffer copy pattern with:
let kbuf = match copy_user_buffer(ttbr0, buf, len) {
    Ok(buf) => buf,
    Err(e) => return e,
};
```

**fs/open.rs:**
```rust
// REPLACE: Lines 25-29 with:
use crate::syscall::util::validate_dirfd_path;

if let Err(e) = validate_dirfd_path(dirfd, path_str, "openat") {
    return e;
}
```

**fs/link.rs:**
```rust
// REMOVE: Lines 7-9 (UTIME_NOW, UTIME_OMIT)
// ADD: Import
use crate::syscall::constants::{UTIME_NOW, UTIME_OMIT};

// REPLACE: 4 dirfd validation blocks with validate_dirfd_path calls
```

**fs/statx.rs:**
```rust
// REMOVE: Line 9 (duplicate AT_EMPTY_PATH)
// ADD: Import
use crate::syscall::constants::AT_EMPTY_PATH;
```

### Step 4: Update syscall/mod.rs

```rust
// ADD: New modules
pub mod constants;
pub mod types;
pub mod util;

// MODIFY: Re-export line
pub use crate::arch::{SyscallFrame, SyscallNumber, is_svc_exception};
pub use types::{Stat, Timespec};  // Now from types, arch re-exports

// MODIFY: fcntl module - delegate to constants
pub mod fcntl {
    pub use super::constants::{
        AT_FDCWD, AT_SYMLINK_NOFOLLOW, AT_REMOVEDIR,
        AT_SYMLINK_FOLLOW, AT_NO_AUTOMOUNT, AT_EMPTY_PATH
    };
}
```

## Call Site Inventory

### Stat Users (will get via re-export, no changes needed)
- `syscall/fs/stat.rs` - `use crate::arch::Stat`
- `fs/vfs/inode.rs` - `use crate::arch::Stat`
- `loader/elf.rs` - `use crate::arch::Stat`

### Timespec Users
- `syscall/time.rs` - `use crate::arch::Timespec`
- `syscall/fs/link.rs` - uses inline, will import from constants

### Termios Users
- `fs/tty.rs` - `use crate::arch::{Termios, ...}`
- `syscall/fs/fd.rs` - `use crate::arch::{Termios, ...}`

### UserIoVec Users
- `syscall/fs/read.rs:sys_readv` - local definition
- `syscall/fs/write.rs:sys_writev` - local definition

### Buffer Copy Pattern Locations
- `write.rs:89-96` (PtyMaster)
- `write.rs:115-122` (VfsFile)
- `write.rs:135-142` (PipeWrite)
- `write.rs:167-174` (write_to_tty)

### dirfd Validation Locations
- `open.rs:25-29` (sys_openat)
- `open.rs:135-141` (sys_faccessat)
- `link.rs:26-29` (sys_utimensat)
- `link.rs:102-106` (sys_linkat)
- `link.rs:138-141` (sys_symlinkat)
- `link.rs:165-168` (sys_readlinkat)

## Rollback Plan

If issues arise, rollback is straightforward:

1. **Revert arch module changes** - restore original Stat/Timespec/Termios definitions
2. **Revert syscall handler imports** - restore local definitions
3. **Delete new modules** - remove types.rs, constants.rs, util.rs

Since we're using re-exports, the old import paths continue to work. The only risk is if we miss a constant value or struct field during the extraction.

**Verification before each step:**
```bash
cargo xtask build kernel
cargo xtask test unit
```
