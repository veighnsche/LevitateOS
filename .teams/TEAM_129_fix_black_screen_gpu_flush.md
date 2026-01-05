# TEAM_129: Fix Black Screen - GPU Flush Commented Out

## Problem
User reported black screen when running `bash run.sh` - boot logs show successful initialization but no visual output.

## Root Cause
The GPU flush code in the timer interrupt handler (`kernel/src/main.rs` lines 332-340) was commented out:
```rust
/*
if count % 5 == 0 {
    if let Some(mut guard) = gpu::GPU.try_lock() {
        if let Some(gpu_state) = guard.as_mut() {
            let _ = gpu_state.flush();
        }
    }
}
*/
```

Without periodic flush, the framebuffer contents are never pushed to the display.

## Fix
Uncomment the GPU flush code in TimerHandler.

## Testing Gap
The unit tests don't exercise the actual display pipeline - they pass but don't catch this regression.

## Status
- [x] Identified root cause
- [x] Applied fix (uncommented GPU flush in timer handler)
- [x] Verified with run.sh
- [x] Added GPU regression tests:
  - GPU flush counter (fails if flush count is 0)
  - Framebuffer content check (fails if all pixels are black)
- [x] Updated behavior test to verify GPU display pipeline
- [x] Updated golden log with new boot messages

## Files Modified
- `kernel/src/main.rs` - Uncommented GPU flush, added GPU_TEST verification
- `kernel/src/gpu.rs` - Added flush counter and framebuffer content check
- `kernel/src/syscall.rs` - Added yield syscall (SYS_YIELD = 7)
- `kernel/src/terminal.rs` - Added GPU flush after every write (fixes shell output not visible)
- `userspace/libsyscall/src/lib.rs` - Added yield_cpu() wrapper
- `userspace/init/src/main.rs` - Call yield_cpu() instead of spin loop
- `xtask/src/main.rs` - Added `cargo xtask kill` command
- `xtask/src/clean.rs` - Added kill_qemu() function
- `xtask/src/tests/behavior.rs` - Added GPU regression assertions, shell execution checks
- `tests/golden_boot.txt` - Updated with shell output
- `run.sh` - Switched to SDL display for better window sizing

## Regression Prevention
The behavior test now verifies **5 critical behaviors**:

### GPU Display (Black Screen Prevention)
| Check | What it catches |
|-------|-----------------|
| `[GPU_TEST] Flush count: N > 0` | GPU flush disabled |
| `[GPU_TEST] Flush count: N >= 10` | write_str not flushing (shell output invisible) |
| `[GPU_TEST] Framebuffer: M > 0 non-black` | Terminal not rendering |

### Shell Execution (Scheduling Prevention)
| Check | What it catches |
|-------|-----------------|
| `[INIT] Shell spawned as PID 2` | Spawn syscall broken |
| `[TASK] Entering user task PID=2` | Scheduler not running shell |
| `LevitateOS Shell` | Userspace execution failure |

### Failure Messages
- "GPU flush count is 0 - display would be black!"
- "Framebuffer is entirely black - no content rendered!"
- "Shell was spawned but never scheduled! (scheduling bug)"
- "Shell started but didn't print banner! (userspace execution bug)"
