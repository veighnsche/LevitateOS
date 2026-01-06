# TEAM_183: Investigate Directory Iteration Edge Cases

## Bug Report
- **Component**: `ReadDir`, `DirEntry`, `FileType`, `sys_getdents`
- **Task**: Proactively find and fix edge cases/bugs
- **Source**: User requested investigation
- **Status**: ✅ COMPLETED - Fixed 3 bugs

## Files Modified
- `userspace/ulib/src/io.rs` - ErrorKind enum and from_errno
- `kernel/src/syscall/fs.rs` - sys_getdents reclen calculation

## Bugs Found & Fixed

### Bug 1: CRITICAL - ErrorKind value collision with ENOTDIR - FIXED
**Location**: `userspace/ulib/src/io.rs:29`
**Problem**: TEAM_182 added `UnexpectedEof = -7` which conflicts with kernel's `ENOTDIR = -7`
**Impact**: Calling `getdents` on a non-directory would incorrectly report "unexpected end of file"
**Fix**: 
- Removed `#[repr(i32)]` from ErrorKind (internal errors shouldn't have syscall values)
- Added `NotADirectory` variant to properly map ENOTDIR (-7)
- Added `-7 => Self::NotADirectory` to `from_errno()`

### Bug 2: Missing ENOTDIR mapping in userspace - FIXED
**Location**: `userspace/ulib/src/io.rs:40-48`
**Problem**: `from_errno()` didn't handle -7 (ENOTDIR), so it returned `Unknown`
**Fix**: Added mapping `-7 => Self::NotADirectory`

### Bug 3: Kernel reclen overflow potential - FIXED
**Location**: `kernel/src/syscall/fs.rs:433`
**Problem**: `((19 + name_len + 1 + 7) / 8) * 8` could overflow on very long names, and `reclen as u16` could truncate
**Fix**: Added checked arithmetic with overflow guard, skip entries that would overflow

## Hypotheses Evaluated

| Hypothesis | Status | Notes |
|------------|--------|-------|
| H1: Dirent parsing overflow | NOT A BUG | Bounds checks exist: `self.pos + 19 > self.end` |
| H2: d_reclen validation | NOT A BUG | Check `d_reclen < 19` prevents underflow |
| H3: Empty name handling | NOT A BUG | Creates empty string, valid behavior |
| H4: UTF-8 validation | NOT A BUG | Returns InvalidArgument error on non-UTF8 |
| H5: Large directory handling | NOT A BUG | Iterator pattern handles refilling correctly |
| H6: Kernel reclen overflow | CONFIRMED | Fixed with checked arithmetic |

## Verification
- [x] Full build succeeds (`cargo xtask build all`)
- [x] All crate tests pass (37 tests)
- [x] No regressions introduced

## Handoff Notes
Critical fix: ENOTDIR error code was being masked by UnexpectedEof collision.
The directory iteration is now robust with:
- Proper error mapping for all kernel errno values
- Safe reclen calculation in kernel that won't overflow
- Internal error types (UnexpectedEof, WriteZero) no longer conflict with syscall errors

