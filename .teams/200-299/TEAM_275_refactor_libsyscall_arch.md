# TEAM_275: Refactor libsyscall Architecture Abstraction

## Goal
Refactor `libsyscall` to use a clean architecture abstraction layer instead of scattered `#[cfg(target_arch)]` flags.

## Status: COMPLETE ✅

## Changes Made

### New Files
- `arch/mod.rs` - Single cfg selector for architecture
- `arch/aarch64.rs` - AArch64 syscall primitives (syscall0-7, noreturn variants)
- `arch/x86_64.rs` - x86_64 stubs (for future implementation)

### Modified Files
All syscall files migrated to use `arch::syscallN()`:
- `lib.rs` - Added `mod arch;`
- `sched.rs`, `io.rs`, `time.rs`, `signal.rs`, `sync.rs`, `tty.rs`, `mm.rs`, `process.rs`, `fs.rs`

## Results
- **cfg flags reduced**: 100+ → 4 (all in `arch/mod.rs`)
- **Tests**: 13/13 passed (AArch64)
- **Builds**: x86_64 compiles successfully

## Handoff
- [x] All tests pass
- [x] No cfg flags in business logic
- [x] Team file updated
