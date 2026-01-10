# TEAM_407: Refactor - Consolidate Scattered Code

## Status: PLANNED

## Summary

Consolidate scattered code that implements cohesive behaviors into unified modules:
1. Move `Stat`, `Timespec`, `Termios` from duplicated arch modules to `syscall/types.rs`
2. Consolidate Linux ABI constants into `syscall/constants.rs`
3. Extract common syscall utilities into `syscall/util.rs`

## Problem Statement

The codebase has significant duplication:
- `Stat` struct (128 bytes) defined identically in both `arch/aarch64/mod.rs` and `arch/x86_64/mod.rs`
- `Timespec` and `Termios` similarly duplicated
- `UserIoVec` defined in both `syscall/fs/read.rs` and `syscall/fs/write.rs`
- Buffer copying pattern repeated 4 times in `write.rs`
- dirfd validation pattern repeated 6+ times across syscall handlers
- Linux ABI constants scattered across 5+ files

## Solution

Create three new modules in `syscall/`:
- `types.rs` - Shared ABI types (Stat, Timespec, Termios, UserIoVec)
- `constants.rs` - All Linux ABI constants (AT_*, SEEK_*, termios flags, ioctl numbers)
- `util.rs` - Helper functions (copy_user_buffer, validate_dirfd_path)

Update arch modules to re-export from `syscall/types`, preserving backward compatibility.

## Files Changed

### New Files
- `crates/kernel/src/syscall/types.rs`
- `crates/kernel/src/syscall/constants.rs`
- `crates/kernel/src/syscall/util.rs`

### Modified Files
- `crates/kernel/src/arch/aarch64/mod.rs` - Remove duplicates, add re-exports
- `crates/kernel/src/arch/x86_64/mod.rs` - Remove duplicates, add re-exports
- `crates/kernel/src/syscall/mod.rs` - Add new module declarations
- `crates/kernel/src/syscall/fs/read.rs` - Use shared UserIoVec
- `crates/kernel/src/syscall/fs/write.rs` - Use shared UserIoVec and copy_user_buffer
- `crates/kernel/src/syscall/fs/open.rs` - Use validate_dirfd_path
- `crates/kernel/src/syscall/fs/link.rs` - Use validate_dirfd_path, shared constants
- `crates/kernel/src/syscall/fs/statx.rs` - Use shared AT_EMPTY_PATH

## Impact

- **Lines removed**: ~530 (duplicated code)
- **Lines added**: ~350 (new modules)
- **Net reduction**: ~180 lines
- **Maintenance benefit**: Changes to Stat/Termios only need to be made once

## Planning Documents

- `docs/planning/refactor-consolidate-scattered-code/phase-1.md` - Discovery & Safeguards
- `docs/planning/refactor-consolidate-scattered-code/phase-2.md` - Structural Extraction
- `docs/planning/refactor-consolidate-scattered-code/phase-3.md` - Migration
- `docs/planning/refactor-consolidate-scattered-code/phase-4.md` - Cleanup
- `docs/planning/refactor-consolidate-scattered-code/phase-5.md` - Hardening

## Testing

```bash
cargo xtask test behavior  # Verify syscall behavior unchanged
cargo xtask test unit      # Verify no regressions
cargo xtask build all      # Both architectures build
```

## ABI Constraints

- `Stat` must remain exactly 128 bytes
- `Timespec` must remain exactly 16 bytes
- All constant values must match Linux ABI

## References

- Linux AArch64 syscall numbers: https://github.com/torvalds/linux/blob/master/include/uapi/asm-generic/unistd.h
- Linux x86_64 syscall numbers: https://github.com/torvalds/linux/blob/master/arch/x86/entry/syscalls/syscall_64.tbl
