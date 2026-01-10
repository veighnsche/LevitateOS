# TEAM_404: Fix Eyra Build-Std Conflict

**Date**: 2026-01-10
**Status**: ✅ COMPLETE
**Type**: Bug Fix

## Symptom

Building Eyra workspace fails with errors like:
```
error[E0599]: no method named `repeat` found for reference `&'static str`
error[E0412]: cannot find type `String` in this scope
error[E0531]: cannot find tuple struct or tuple variant `Ok` in this scope
```

## Root Cause (Found in Team Files)

**TEAM_395** fixed the Eyra dependency conflicts previously. However, the current `build_eyra()` command uses `-Zbuild-std=std,panic_abort` which CONFLICTS with Eyra's design:

1. Eyra uses rename pattern: `std = { package = "eyra" }` 
2. This provides a crate named `std` that Eyra uses to intercept std calls
3. `-Zbuild-std=std` tells cargo to build Rust's REAL std from source
4. **Both provide `std` → conflict → prelude not injected**

**Evidence**: The coreutils build (lines 523-539) does NOT use `-Zbuild-std` and works fine.

## Fix

Remove `-Zbuild-std=std,panic_abort` from the eyra workspace build command.

## Files Changed

1. `xtask/src/build/commands.rs` - Remove conflicting build flag
2. `crates/userspace/Cargo.toml` - Remove libsyscall from workspace (moved to eyra/)
3. `crates/userspace/init/Cargo.toml` - Update libsyscall path to eyra/libsyscall
4. `crates/userspace/shell/Cargo.toml` - Update libsyscall path to eyra/libsyscall

## Why This Bug Kept Recurring

Previous teams fixed related issues but didn't document the critical constraint:

> **NEVER use `-Zbuild-std` with Eyra's rename pattern.**

The rename pattern `std = { package = "eyra" }` IS the std provider. Using `-Zbuild-std` 
tries to build the real Rust std, causing conflicts.

## Additional Work

5. `crates/userspace/init/src/main.rs` - Updated to spawn brush first, fallback to lsh
6. `xtask/src/build/commands.rs` - Added brush to initramfs creation

## Known Limitations

Brush and other Eyra binaries may fail with "Unknown syscall" errors because they use 
Linux syscalls that the kernel doesn't implement yet. Observed: syscall 22 (`pipe`).

## Handoff Checklist

- [x] Project builds cleanly (`cargo xtask build all --arch x86_64`)
- [x] All xtask tests pass (13/13)
- [x] Team file updated
- [x] Documented why bug occurred to prevent future recurrence
- [x] Brush shell added to initramfs
- [x] Init updated to prefer brush over lsh
