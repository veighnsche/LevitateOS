# TEAM_090: Make recstrap Production Ready

## Status: COMPLETE

## Summary

Made recstrap fully production ready with comprehensive edge case handling, bug fixes, and thorough testing.

## Features Added

1. **`--check` flag** - Pre-flight validation only, don't extract. Runs all 14 validation checks and reports ready/not-ready status.

## Bugs Fixed

1. **stderr capture bug** - Using `Stdio::inherit()` then `output()` caused empty stderr on failure. Fixed by using `status()` instead.
2. **CString path handling** - `to_string_lossy()` corrupted non-UTF8 paths. Fixed with `OsStrExt::as_bytes()`.
3. **Protected only /** - Only blocked `/`, but `/usr`, `/etc`, etc. equally dangerous. Now blocks 18 critical paths.
4. **lost+found false positive** - Fresh ext4 mount points contain `lost+found`, causing "not empty" error on valid targets. Fixed by ignoring `lost+found` in empty check.
5. **Write test file cleanup** - If previous run was interrupted, `.recstrap_write_test` might remain and trigger "not empty". Fixed by ignoring it in empty check.
6. **CI smoke test** - Fixed E001 → E008 (root check happens first in non-root CI environment).

## New Safety Checks

- **E010: Protected paths** - Blocks /, /usr, /etc, /home, etc. (18 paths total, CANNOT override)
- **E014: Squashfs not readable** - Checks if we can actually read the file
- **E015: Squashfs inside target** - Prevents recursive extraction disaster

## All Safety Checks (14 total, in order)

1. Root privileges (E008)
2. unsquashfs available (E007)
3. Target exists (E001)
4. Target is directory (E002)
5. Path canonicalization (resolve symlinks, ..)
6. **Target not protected path (E010)** - CANNOT be overridden
7. Target writable (E003)
8. Is mount point (E011) - skipped with --force
9. Target empty (E009) - skipped with --force
10. Sufficient disk space (E012)
11. Squashfs exists (E004)
12. Squashfs is a file (E013)
13. **Squashfs readable (E014)**
14. **Squashfs not inside target (E015)**

## Protected Paths (blocked even with --force)

`/`, `/bin`, `/boot`, `/dev`, `/etc`, `/home`, `/lib`, `/lib64`, `/opt`, `/proc`, `/root`, `/run`, `/sbin`, `/srv`, `/sys`, `/tmp`, `/usr`, `/var`

## Error Codes (15 total)

| Code | Exit | Description |
|------|------|-------------|
| E001 | 1 | Target directory does not exist |
| E002 | 2 | Target is not a directory |
| E003 | 3 | Target directory not writable |
| E004 | 4 | Squashfs image not found |
| E005 | 5 | unsquashfs command failed |
| E006 | 6 | Extraction verification failed |
| E007 | 7 | unsquashfs not installed |
| E008 | 8 | Must run as root |
| E009 | 9 | Target directory not empty |
| E010 | 10 | Target is protected system path |
| E011 | 11 | Target is not a mount point |
| E012 | 12 | Insufficient disk space |
| E013 | 13 | Squashfs is not a regular file |
| E014 | 14 | Squashfs is not readable |
| E015 | 15 | Squashfs is inside target |

## Test Coverage

- **35 unit tests** - error codes, helper functions, edge cases, lost+found handling
- **20 integration tests** - CLI, error paths, protected paths
- **55 total tests** - all pass

## Cheat-Aware Validation

Added `guarded_ensure!` macro based on Anthropic's [emergent misalignment research](https://www.anthropic.com/research/emergent-misalignment-reward-hacking). Uses same philosophy as cheat-guard but with recstrap's distinct error codes.

Every validation check now documents:

- **protects** - What user scenario this check protects
- **severity** - CRITICAL, HIGH, MEDIUM, or LOW
- **cheats** - Ways the check could be weakened to falsely pass
- **consequence** - What users experience if cheated

When a check fails, full cheat documentation is printed:

```
======================================================================
=== CHEAT-GUARDED VALIDATION FAILED ===
======================================================================

PROTECTS: Critical system directories are never overwritten
SEVERITY: CRITICAL

CHEAT VECTORS (ways this check could be weakened):
  1. Remove paths from protected list
  2. Add --force override for protected paths
  3. Skip check when running as root
  4. Check before canonicalization (symlink bypass)

USER CONSEQUENCE IF CHEATED:
  Complete system destruction - / or /usr overwritten, unbootable system

======================================================================
```

## Verification

```bash
cargo test        # 55 tests pass
cargo clippy      # No warnings
cargo build --release  # Success
```
