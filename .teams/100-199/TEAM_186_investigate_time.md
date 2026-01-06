# TEAM_186: Investigate Time Edge Cases

## Bug Report
- **Component**: `Duration`, `Instant`, `sleep()`, `sleep_ms()` via syscalls
- **Task**: Proactively find and fix edge cases/bugs
- **Source**: User requested investigation
- **Status**: ✅ COMPLETED - Fixed 4 bugs

## Files Modified
- `userspace/ulib/src/time.rs` - Instant::now()
- `kernel/src/syscall/time.rs` - sys_nanosleep

## Bugs Found & Fixed

### Bug 1: Instant::now() ignores syscall error - FIXED
**Location**: `userspace/ulib/src/time.rs:147-148`
**Problem**: Return value of `clock_gettime()` was ignored with `let _ = ...`
**Fix**: Check return value, return zeros on error

### Bug 2: tv_nsec cast truncation - FIXED
**Location**: `userspace/ulib/src/time.rs:153-154`
**Problem**: `ts.tv_nsec as u32` could truncate if value corrupted
**Fix**: Clamp to valid range with `.min(999_999_999)`

### Bug 3: Kernel nanosleep overflow - FIXED
**Location**: `kernel/src/syscall/time.rs:36-37`
**Problem**: `total_ns * freq` could overflow on long sleep durations
**Fix**: Split calculation: `secs * freq + (nanos * freq) / 1e9` using u128 intermediate

### Bug 4: Kernel nanoseconds not normalized - FIXED
**Location**: `kernel/src/syscall/time.rs:27-30`
**Problem**: If `nanoseconds > 1e9`, calculation would be incorrect
**Fix**: Normalize: `extra_secs = nanos / 1e9`, `norm_nanos = nanos % 1e9`

## Hypotheses Evaluated

| Hypothesis | Status | Notes |
|------------|--------|-------|
| H1: Duration arithmetic overflow | NOT A BUG | Already uses saturating_add/sub |
| H2: Instant elapsed overflow | NOT A BUG | Uses saturating arithmetic |
| H3: sleep_ms overflow | NOT A BUG | from_millis handles correctly |
| H4: Nanoseconds out of range | CONFIRMED | Fixed with normalization |
| H5: Zero duration sleep | NOT A BUG | Handled correctly (returns immediately) |
| H6: Kernel syscall error handling | CONFIRMED | Fixed Instant::now() |

## Verification
- [x] Full build succeeds (`cargo xtask build all`)
- [x] All crate tests pass (37 tests)
- [x] No regressions introduced

## Handoff Notes
The time subsystem is now hardened with:
- Proper syscall error handling in Instant::now()
- Overflow-safe tick calculation in kernel nanosleep
- Normalized nanoseconds handling (accepts any u64 value)
- Clamped tv_nsec to valid range for safety

