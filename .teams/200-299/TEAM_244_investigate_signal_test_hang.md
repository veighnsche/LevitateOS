# TEAM_244: Investigate signal_test hang at pause()

## Bug Report
- signal_test hangs at `pause()` 
- Ctrl+C doesn't work to interrupt it
- User expects Ctrl+C should send SIGINT and unblock pause()

## ROOT CAUSES FOUND ✅

### Bug 1: signal_test logic flaw
The test sends SIGINT to itself BEFORE calling pause():
```rust
kill(pid, Signal::SIGINT);  // Signal delivered here, handler runs
// ... prints ...
pause();  // Waits for ANOTHER signal that never comes!
```

The test "passed" due to a **scheduler quirk**: when pause() calls `schedule()` and the ready queue is empty, schedule() returns immediately without switching:
```rust
pub fn schedule(&self) {
    if let Some(next) = self.pick_next() {
        crate::task::switch_to(next);  // Only switches if there's a task
    }
    // Returns immediately if no tasks!
}
```

This is a **FALSE POSITIVE** - pause() returns not because a signal woke it, but because there was nothing else to run.

### Bug 2: Ctrl+C goes to wrong process
- `FOREGROUND_PID` is set by shell via `set_foreground()`
- Shell sets `FOREGROUND_PID = child_pid` when running commands
- BUT **test_runner doesn't do this** for its spawned children
- So FOREGROUND_PID still points to shell (or 0), not signal_test
- Ctrl+C → `signal_foreground_process()` → signals wrong process

## Fix Plan

### Fix 1: signal_test should not use pause() after self-signal
- Remove pause() call - test already validates signal handling via kill()
- OR: Have the signal handler set a flag, and check flag instead of pause()

### Fix 2: test_runner should set foreground for child processes
- Call `set_foreground(child_pid)` before waiting
- Call `set_foreground(own_pid)` after child exits

## Confidence
**HIGH** - Both root causes clearly identified with evidence.

## FIXES APPLIED ✅

### Fix 1: signal_test.rs
- Removed `pause()` call - it was waiting for a second signal that never came
- Added `AtomicBool HANDLER_RAN` flag
- Handler sets flag, main checks flag to verify handler executed
- Test now properly validates signal delivery without false positives

### Fix 2: test_runner.rs
- Added `set_foreground(child_pid)` before waitpid()
- Added `set_foreground(own_pid)` after child exits
- Ctrl+C now correctly signals the running test process

## Verification
All 4 tests pass:
```
[TEST_RUNNER] Total: 4/4 tests passed
[TEST_RUNNER] RESULT: PASSED
```

## Files Modified
- `userspace/levbox/src/bin/signal_test.rs` - Fixed test logic
- `userspace/levbox/src/bin/test_runner.rs` - Added foreground handling
