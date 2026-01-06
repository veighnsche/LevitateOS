# TEAM_170: Implement ulib Phase 10 Steps 7-8

## Objective
Implement time syscalls (nanosleep) and time abstractions (Duration, sleep).

## Context
- **Plan:** `docs/planning/ulib-phase10/phase-3.md`
- **Previous:** TEAM_169 (Steps 5-6)
- **Scope:** Steps 7-8 (time syscalls + ulib time)

## Implementation Progress

### Step 7: Kernel Time Syscalls ✅
- [x] UoW 1: Implement nanosleep syscall
  - Added syscall number 12 (nanosleep)
  - Uses ARM generic timer (CNTVCT_EL0, CNTFRQ_EL0)
  - Yield-loop until target time reached
- [x] UoW 2: Add clock_gettime syscall
  - Added syscall number 13 (clock_gettime)
  - Returns monotonic time from ARM timer

### Step 8: ulib Time Abstractions ✅
- [x] UoW 1: Create time module
  - Created `userspace/ulib/src/time.rs`
- [x] UoW 2: Implement Duration and sleep()
  - `Duration` with secs/nanos, from_secs/millis/micros/nanos
  - `Instant` with now(), elapsed(), duration_since()
  - `sleep(Duration)` and `sleep_ms(millis)`

## Status
- COMPLETE

## Files Modified

### Kernel
- `kernel/src/syscall.rs` — Added nanosleep, clock_gettime syscalls

### Userspace
- `userspace/libsyscall/src/lib.rs` — Added syscall wrappers, Timespec
- `userspace/ulib/src/lib.rs` — Added time module
- `userspace/ulib/src/time.rs` — NEW: Duration, Instant, sleep()

## Usage

```rust
use ulib::time::{Duration, Instant, sleep, sleep_ms};

// Measure elapsed time
let start = Instant::now();
do_something();
println!("Took {:?}", start.elapsed());

// Sleep
sleep(Duration::from_millis(100));
sleep_ms(50);
```

## Handoff Checklist
- [x] Project builds cleanly
- [x] All tests pass
- [x] Team file updated
- [x] Code comments include TEAM_170

## Next Steps (for future teams)
1. Step 9: Integration demo
2. Update shell to use timing functions
