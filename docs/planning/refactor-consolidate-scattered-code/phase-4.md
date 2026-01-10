# Phase 4: Cleanup

## Dead Code Removal (Rule 6)

After migration, the following code becomes dead and must be removed:

### From arch/aarch64/mod.rs
- Lines 231-274: `Stat` struct definition
- Lines 276-391: `Stat` impl block
- Lines 406-412: `Timespec` struct
- Lines 414-462: `Termios` + NCCS + constants
- Lines 464-500: Termios constants (ISIG, ICANON, etc.)

**Estimated removal: ~270 lines**

### From arch/x86_64/mod.rs
- Lines 234-258: `Stat` struct definition
- Lines 260-376: `Stat` impl block
- Lines 383-388: `Timespec` struct
- Lines 390-456: `Termios` + NCCS + constants

**Estimated removal: ~220 lines**

### From syscall/fs/read.rs
- Lines 57-63: Local `UserIoVec` definition

**Estimated removal: 7 lines**

### From syscall/fs/write.rs
- Lines 8-14: Local `UserIoVec` definition
- Inline buffer copy patterns (replaced with function calls)

**Estimated removal: ~30 lines**

### From syscall/fs/link.rs
- Lines 7-9: `UTIME_NOW`, `UTIME_OMIT` constants

**Estimated removal: 3 lines**

### From syscall/fs/statx.rs
- Line 9: Duplicate `AT_EMPTY_PATH`

**Estimated removal: 1 line**

## Temporary Adapters to Remove

None. The re-export pattern is permanent and provides a clean API.

## Tighten Encapsulation

### Make Internal Fields Private Where Possible

In `syscall/types.rs`:
- `Stat.__pad1`, `Stat.__pad2`, `Stat.__unused` - keep pub for repr(C) but document as internal
- `UserIoVec` fields - keep pub, needed for direct access in syscall handlers

### Visibility Cleanup

```rust
// types.rs - public API
pub struct Stat { ... }
pub struct Timespec { ... }
pub struct Termios { ... }
pub struct UserIoVec { ... }
pub const NCCS: usize = 32;

// constants.rs - all pub for use across modules
pub const AT_FDCWD: i32 = -100;
// ...

// util.rs - pub(crate) since only used within kernel
pub(crate) fn copy_user_buffer(...) -> Result<Vec<u8>, i64> { ... }
pub(crate) fn validate_dirfd_path(...) -> Result<(), i64> { ... }
```

## File Size Check

### Target Sizes
| File | Before | After | Target |
|------|--------|-------|--------|
| arch/aarch64/mod.rs | ~551 lines | ~280 lines | <500 |
| arch/x86_64/mod.rs | ~585 lines | ~360 lines | <500 |
| syscall/types.rs | NEW | ~200 lines | <500 |
| syscall/constants.rs | NEW | ~100 lines | <500 |
| syscall/util.rs | NEW | ~50 lines | <500 |
| syscall/fs/write.rs | ~213 lines | ~180 lines | <500 |
| syscall/fs/read.rs | ~249 lines | ~240 lines | <500 |
| syscall/fs/link.rs | ~187 lines | ~170 lines | <500 |

All files remain well under 500 lines (ideal) and 1000 lines (max).

## Cleanup Verification

```bash
# Ensure no dead code warnings
cargo build --release 2>&1 | grep -i "dead_code\|unused"

# Verify no duplicate definitions
grep -r "struct Stat" crates/kernel/src/ | wc -l  # Should be 1

# Verify constants are consolidated
grep -r "AT_EMPTY_PATH" crates/kernel/src/ | wc -l  # Should be 1

# Run full test suite
cargo xtask test
```
