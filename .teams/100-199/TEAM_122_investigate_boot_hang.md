# Investigation: Shell Runs but Display Freezes

**Status:** INVESTIGATING  
**Team:** TEAM_122

## 1. Symptom
- **Expected:** Shell prompt appears on VNC (GPU) display.
- **Actual:** Shell prompt appears on UART, but VNC display freezes at `[BOOT] Filesystem initialized successfully.`.
- **Delta:** System is alive (shell runs), but display updates stop.

## 2. Hypotheses
1.  **Deadlock on GPU Lock (High Confidence):**
    - `input::poll` calls `GPU.lock()` (via `dimensions()`).
    - Timer interrupt fires -> calls `print!` -> calls `terminal::write` -> calls `GPU.lock()`.
    - If interrupt hits while `input::poll` holds the lock: Deadlock.
2.  **GPU Driver Crash:**
    - Driver enters invalid state after specific command.
    - Unlikely given UART logs continue showing `[TERM] ...` if terminal writes succeed? No, terminal writes might be buffering or going to UART first.

## 4. Root Cause Analysis
1.  **Interrupts Masked in User Mode:** `kmain` disabled interrupts (or failed to enable them) before transitioning to `init`. Since `Context` switching does not restore PSTATE (interrupts), user tasks ran with interrupts disabled.
    - **Effect:** Timer interrupt never fired.
    - **Symptom:** GPU flush (driven by timer) never happened -> Screen frozen.
2.  **Deadlock in Timer Handler:** Even if interrupts were enabled, `TimerHandler` called `verbose!`, which calls `println!`, taking the UART lock. If the main thread (shell) held the UART lock when interrupted -> Deadlock.

## 5. Decision & Fix
**Fixed Immediately:**
1.  **Enable Interrupts:** Added `unsafe { levitate_hal::interrupts::enable() };` in `kmain` before `task_exit()`.
2.  **Fix Deadlock:** Removed `verbose!` logging from `TimerHandler`.
3.  **Prevent Future Deadlock:** Converted `GPU` static to `IrqSafeLock`.

## 6. Verification
- **UART Logs:** Show `LevitateOS Shell (lsh) v0.1` and `[INIT] Shell spawned`.
- **Logic:** Timer now fires, driving `gpu_state.flush()`. Locking is safe.
- **Status:** **FIXED**
