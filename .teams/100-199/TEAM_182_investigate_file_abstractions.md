# TEAM_182: Investigate File Abstractions Edge Cases

## Bug Report
- **Component**: `File`, `Metadata`, `Read::read()` in ulib
- **Task**: Proactively find and fix edge cases/bugs
- **Source**: User requested investigation (similar to TEAM_181 allocator work)
- **Status**: ✅ COMPLETED - Fixed 5 edge cases

## Files Modified
- `userspace/ulib/src/fs.rs` - Metadata struct
- `userspace/ulib/src/io.rs` - ErrorKind, Read/Write traits, BufReader/BufWriter

## Bugs Found & Fixed

### Bug 1: `read_exact` returns wrong ErrorKind on EOF - FIXED
**Location**: `userspace/ulib/src/io.rs:106`
**Problem**: Returned `ErrorKind::Unknown` when EOF reached before buffer filled
**Fix**: Added `ErrorKind::UnexpectedEof` and use it in `read_exact`

### Bug 2: `write_all` returns wrong ErrorKind on short write - FIXED
**Location**: `userspace/ulib/src/io.rs:130`
**Problem**: Returned `ErrorKind::Unknown` when write returns 0
**Fix**: Added `ErrorKind::WriteZero` and use it in `write_all`

### Bug 3: Missing `is_dir()` method on Metadata - FIXED
**Location**: `userspace/ulib/src/fs.rs:82-106`
**Problem**: `Metadata` only had `is_file()`, no way to check for directories
**Fix**: Changed `is_file: bool` to `mode: u32`, added `is_dir()` method

### Bug 4: `BufReader::with_capacity(0, _)` edge case - FIXED
**Location**: `userspace/ulib/src/io.rs:173-182`
**Problem**: Zero capacity creates unusable buffer, causes issues in `fill_buf`
**Fix**: Treat capacity of 0 as `DEFAULT_BUF_CAPACITY`

### Bug 5: `BufWriter::with_capacity(0, _)` edge case - FIXED
**Location**: `userspace/ulib/src/io.rs:316-320`
**Problem**: Zero capacity causes every write to bypass buffering incorrectly
**Fix**: Treat capacity of 0 as `DEFAULT_BUF_CAPACITY`

## Hypotheses Evaluated

| Hypothesis | Status | Notes |
|------------|--------|-------|
| H1: Empty buffer read | NOT A BUG | Kernel returns 0, valid behavior |
| H2: Large read overflow | NOT A BUG | Return is i64, cast to usize is safe on 64-bit |
| H3: FD exhaustion | NOT A BUG | Kernel returns EMFILE (-6), properly handled |
| H4: read_exact EOF handling | CONFIRMED | Fixed with UnexpectedEof error |
| H5: Metadata st_mode | CONFIRMED | Added is_dir() for completeness |
| H6: Path handling | NOT A BUG | Kernel validates path length, returns EINVAL |

## Verification
- [x] Full build succeeds (`cargo xtask build all`)
- [x] All crate tests pass (37 tests)
- [x] No regressions introduced

## Handoff Notes
The file abstraction layer is now hardened with:
- Proper error types for EOF and write failures (matches std::io semantics)
- Complete Metadata API with both `is_file()` and `is_dir()`
- Safe BufReader/BufWriter that handle zero-capacity edge case

Future considerations:
- Kernel errno values are well-mapped to userspace ErrorKind
- File syscalls have proper validation in kernel

