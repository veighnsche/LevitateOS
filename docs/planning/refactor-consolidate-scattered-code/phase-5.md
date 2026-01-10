# Phase 5: Hardening

## Final Verification

### Build Verification
```bash
# Both architectures must build
cargo xtask build kernel --arch aarch64
cargo xtask build kernel --arch x86_64

# Full build including userspace
cargo xtask build all
```

### Test Verification
```bash
# Unit tests
cargo xtask test unit

# Behavior tests (uses golden files)
cargo xtask test behavior

# Regression tests
cargo xtask test regress
```

### ABI Verification

Create a compile-time assertion for struct sizes:
```rust
// In syscall/types.rs
const _: () = {
    assert!(core::mem::size_of::<Stat>() == 128, "Stat must be 128 bytes");
    assert!(core::mem::size_of::<Timespec>() == 16, "Timespec must be 16 bytes");
    assert!(core::mem::size_of::<UserIoVec>() == 16, "UserIoVec must be 16 bytes");
};
```

### Runtime Verification
```bash
# Boot and verify basic functionality
cargo xtask run --headless --test

# Verify syscalls work (these exercise Stat, Termios)
# - File operations (fstat)
# - TTY operations (tcgets/tcsets)
# - Read/write operations (writev)
```

## Documentation Updates

### Update CLAUDE.md
Add to "Key Architectural Patterns" section:
```markdown
**Syscall Types Module**: Architecture-independent ABI types (`Stat`, `Timespec`, `Termios`)
are defined in `crates/kernel/src/syscall/types.rs` and re-exported from arch modules for
backward compatibility. Linux ABI constants are consolidated in `syscall/constants.rs`.
```

### Update Architecture Overview
If `docs/ARCHITECTURE.md` exists, add note about syscall module structure.

### Code Comments
Each new file should have a module-level doc comment:
```rust
//! TEAM_407: Shared ABI types for syscalls.
//!
//! These types are architecture-independent and match the Linux ABI.
//! They are re-exported from arch modules for backward compatibility.
```

## Handoff Notes

### What Changed
1. Created `syscall/types.rs` with Stat, Timespec, Termios, UserIoVec
2. Created `syscall/constants.rs` consolidating all Linux ABI constants
3. Created `syscall/util.rs` with copy_user_buffer and validate_dirfd_path helpers
4. Updated arch modules to re-export from types (no breaking changes)
5. Updated syscall handlers to use shared utilities

### Migration Pattern for Future Code
When adding new syscalls that need path handling:
```rust
use crate::syscall::util::validate_dirfd_path;
use crate::syscall::constants::AT_FDCWD;

pub fn sys_newat(dirfd: i32, pathname: usize, ...) -> i64 {
    let path_str = match read_user_cstring(...) { ... };
    if let Err(e) = validate_dirfd_path(dirfd, path_str, "newat") {
        return e;
    }
    // ...
}
```

When adding new syscalls that need buffer copying:
```rust
use crate::syscall::util::copy_user_buffer;

pub fn sys_newwrite(fd: usize, buf: usize, len: usize) -> i64 {
    let kbuf = match copy_user_buffer(ttbr0, buf, len) {
        Ok(buf) => buf,
        Err(e) => return e,
    };
    // ...
}
```

### Known Limitations
- `validate_dirfd_path` only supports AT_FDCWD (returns EBADF for other dirfd values)
- When full dirfd support is implemented, update `util.rs` once and all callers benefit

### Metrics
- **Lines removed**: ~530 (duplicated code)
- **Lines added**: ~350 (new modules)
- **Net reduction**: ~180 lines
- **Duplicate definitions eliminated**: 5 (Stat, Timespec, Termios, UserIoVec, AT_EMPTY_PATH)
- **Duplicate code patterns eliminated**: 2 (buffer copy, dirfd validation)
