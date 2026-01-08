# TEAM_240: Investigate Pipe EOF and Signal Bugs

**Created**: 2026-01-07  
**Status**: Active  
**Focus**: Bug Investigation

## Bug Report

### Bug 1: Pipe EOF Returns -11 (EAGAIN) Instead of 0

**Symptom:**
- `pipe_test` Test 3 fails
- After closing write end, read returns -11 (EAGAIN) instead of 0 (EOF)

**Expected Behavior:**
- When write end of pipe is closed and buffer is empty, read should return 0

**Actual Behavior:**
- Read returns -11 (EAGAIN)

**Reproduction:**
```
# pipe_test
[pipe_test] Test 3: Pipe EOF
[pipe_test]   EOF read returned -11, expected 0
[pipe_test] Test 3: FAIL
```

---

### Bug 2: Ctrl+C Not Working / signal_test Stuck

**Symptom:**
- `signal_test` gets stuck at `pause()`
- Ctrl+C doesn't interrupt processes

**Expected Behavior:**
- Ctrl+C should send SIGINT to foreground process
- `pause()` should return when signal is delivered

**Actual Behavior:**
- Process remains stuck, no interrupt received

**Reproduction:**
```
# signal_test
Signal test starting...
Registered SIGINT handler. Sending SIGINT to self...
*** HANDLER: Received signal 02 ***
Signal sent. If handled, we should see handler output.
Waiting for signal in pause()...
[SIGNAL] pause() for PID=7
<stuck here, Ctrl+C has no effect>
```

---

## Hypotheses

### Bug 1: Pipe EOF

| # | Hypothesis | Confidence | Evidence Needed |
|---|------------|------------|-----------------|
| 1.1 | Pipe read doesn't check if write end is closed | High | Check pipe read implementation |
| 1.2 | Write end close status not tracked in pipe state | Medium | Check pipe close implementation |
| 1.3 | Read blocking logic returns EAGAIN incorrectly | Medium | Check blocking/non-blocking logic |

### Bug 2: Signal/Ctrl+C

| # | Hypothesis | Confidence | Evidence Needed |
|---|------------|------------|-----------------|
| 2.1 | UART interrupt handler doesn't convert Ctrl+C to SIGINT | High | Check UART input handling |
| 2.2 | Foreground process tracking broken | Medium | Check shell/foreground PID |
| 2.3 | pause() not waking on SIGINT delivery | Low | Check pause implementation |

---

## Investigation Log

### Bug 1 - Pipe EOF (FIXED)

**Root Cause Found:**

`FdTable::close()` in `kernel/src/task/fd_table.rs` just sets the fd slot to `None` without calling `Pipe::close_read()` or `Pipe::close_write()`. 

The pipe's `write_open` flag stays `true` forever, so when userspace reads after "closing" the write end, `Pipe::read()` sees an empty buffer with write_open=true and returns EAGAIN (-11) instead of 0 (EOF).

**Fix Applied:**
- Updated `FdTable::close()` to call appropriate pipe close method
- Updated `FdTable::dup_to()` to properly close replaced fd
- Kernel builds successfully

**Status:** FIXED - awaiting verification

---

### Bug 2 - Ctrl+C / Signal (ROOT CAUSE FOUND)

**Root Cause:**

`input::poll()` is only called from `read_stdin()` (line 153 of `read.rs`). Input polling is **not** interrupt-driven - it's only called when a process reads from stdin.

When `signal_test` calls `pause()`:
1. The process is blocked (not reading stdin)
2. No one calls `input::poll()`
3. VirtIO keyboard events are never processed
4. Ctrl+C is never detected
5. `signal_foreground_process(SIGINT)` is never called

**Confidence:** HIGH

**Fix Options:**

1. **Poll during scheduler idle** - Call `input::poll()` from the scheduler when no task is running (quick fix)
2. **Real interrupt handling** - Implement VirtIO interrupt handler for input device (proper solution, more complex)
3. **Timer-based polling** - Poll input periodically on timer interrupt (middle ground)

**Recommendation:** Option 1 is simplest for MVP. Poll input from the scheduler's idle loop.
