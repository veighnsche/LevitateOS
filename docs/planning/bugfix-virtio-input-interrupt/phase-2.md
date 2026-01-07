# Phase 2: Root Cause Analysis

**Status**: ✅ Complete (from investigation)

## Root Cause

`input::poll()` is only called from `read_stdin()` in `kernel/src/syscall/fs/read.rs:153`.

When a process calls `pause()` or is blocked on something other than stdin:
1. No process is calling `read_stdin()`
2. `input::poll()` is never called
3. VirtIO input device events are never processed
4. Ctrl+C (character `\x03`) is never detected
5. `signal_foreground_process(SIGINT)` is never called

## Evidence

**Code trace:**

```
pause() blocks → no read() call → poll_input_devices() not invoked → 
input::poll() not called → VirtIO events not processed → Ctrl+C missed
```

**Observation:**
- Ctrl+C works fine when shell is waiting for input (shell is reading stdin)
- Ctrl+C fails when running a program that doesn't read stdin

## Hypotheses Tested

| Hypothesis | Status | Evidence |
|------------|--------|----------|
| Input only polled in read_stdin() | ✅ Confirmed | grep shows only call site is read.rs:153 |
| VirtIO input not interrupt-driven | ✅ Confirmed | No IRQ handler for VirtIO input found |
| signal_foreground_process works | ✅ Confirmed | signal_test shows handler works when sent via kill() |
