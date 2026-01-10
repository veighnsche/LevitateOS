# Phase 1: Discovery & Safeguards

## Refactor Summary

**What**: Consolidate scattered code that implements cohesive behaviors into unified modules.

**Why**: The codebase has significant code duplication across three areas:
1. Architecture-specific structs (`Stat`, `Timespec`, `Termios`) are 100% identical between aarch64/x86_64 but duplicated
2. Syscall utility code (`UserIoVec`, buffer copying, dirfd validation) is copy-pasted across files
3. Linux ABI constants are scattered across 5+ files instead of a single source of truth

**Pain Points**:
- Maintenance burden: Changes must be made in 2+ places
- Risk of divergence: Easy to update one copy and forget others
- Code bloat: ~500+ lines of unnecessary duplication
- Harder to understand: Related code is scattered instead of grouped

## Success Criteria

### Before
- `Stat` struct defined identically in `arch/aarch64/mod.rs:233-274` AND `arch/x86_64/mod.rs:236-258`
- `Timespec` struct defined in both arch modules
- `Termios` + 25 constants duplicated in both arch modules
- `UserIoVec` defined in both `syscall/fs/read.rs:57-63` AND `syscall/fs/write.rs:8-14`
- Buffer copying pattern repeated 4 times in `syscall/fs/write.rs`
- dirfd validation pattern repeated 6+ times across `open.rs`, `link.rs`
- AT_EMPTY_PATH defined in both `syscall/mod.rs:27` AND `syscall/fs/statx.rs:9`

### After
- `Stat`, `Timespec`, `Termios` defined once in `syscall/types.rs`
- Arch modules re-export: `pub use crate::syscall::types::{Stat, Timespec, Termios};`
- `UserIoVec` defined once in `syscall/types.rs`
- `copy_user_buffer()` helper eliminates duplicate buffer copying
- `validate_dirfd_path()` helper eliminates duplicate validation
- All fcntl/AT_* constants consolidated in `syscall/fcntl.rs`

## Behavioral Contracts (APIs that must NOT change)

### Struct Layouts (ABI Critical)
- `Stat` must remain exactly 128 bytes with same field offsets
- `Timespec` must remain 16 bytes (i64, i64)
- `Termios` must remain ABI-compatible with Linux

### Public APIs Preserved
- `crate::arch::Stat` - re-exported from types
- `crate::arch::Timespec` - re-exported from types
- `crate::arch::Termios` - re-exported from types
- `crate::syscall::fcntl::AT_*` constants - same values
- `crate::syscall::errno::*` constants - unchanged

## Golden/Regression Tests to Lock In

```bash
# Run before refactoring to establish baseline
cargo xtask test behavior    # Boot sequence and syscall behavior
cargo xtask test unit        # Host-side unit tests

# Key behavioral tests that exercise Stat/Termios:
# - tests/golden/boot-verbose.txt (TTY initialization uses Termios)
# - Any test that uses fstat() exercises Stat struct
```

## Current Architecture Notes

### File Locations
```
crates/kernel/src/
├── arch/
│   ├── aarch64/mod.rs     # Stat:233-274, Timespec:407-412, Termios:417-456 + constants
│   └── x86_64/mod.rs      # Stat:236-258, Timespec:383-388, Termios:393-418 + constants
└── syscall/
    ├── mod.rs             # Re-exports arch types, defines fcntl module
    ├── fs/
    │   ├── read.rs        # UserIoVec:57-63
    │   ├── write.rs       # UserIoVec:8-14, buffer copy pattern x4
    │   ├── open.rs        # dirfd validation:25
    │   ├── link.rs        # dirfd validation x4 (lines 26, 102, 138, 165)
    │   └── statx.rs       # Duplicate AT_EMPTY_PATH:9
    └── ...
```

### Dependencies
- `Stat` used by: `syscall/fs/stat.rs`, VFS layer, ELF loader
- `Timespec` used by: `syscall/time.rs`, `syscall/fs/link.rs` (utimensat)
- `Termios` used by: `fs/tty.rs`, `syscall/fs/fd.rs` (ioctl)
- `UserIoVec` used by: `sys_readv`, `sys_writev`

### Couplings
- `crate::syscall::mod.rs` line 12: `pub use crate::arch::{Stat, SyscallFrame, SyscallNumber, Timespec, is_svc_exception};`
- Both arch modules must export same symbols

## Constraints

1. **ABI Preservation**: Struct sizes and layouts cannot change
2. **No Feature Regressions**: All existing tests must pass
3. **Minimal Churn**: Re-export pattern to preserve import paths
4. **Architecture-Specific Remains Separate**: `SyscallNumber`, `SyscallFrame` stay in arch modules (they have different layouts)
