# TEAM_156: Investigate Keyboard Input Drops

## Date: 2026-01-06

## Bug Report
**User Report:** Running `bash ./run-term.sh`, after boot, randomly smashing keyboard results in only random amount of key presses getting registered.

### Symptom
- **Expected:** All key presses should be registered
- **Actual:** Only a random subset of key presses are registered
- **Trigger:** Rapid keyboard input ("smashing")

### Environment
- `run-term.sh` mode
- After boot completes

## Prior Team Context
- **TEAM_139:** Fixed GTK focus capture → VNC mode
- **TEAM_149:** Fixed interrupt starvation in sys_read → added interrupt window

Current issue is different: keys ARE being processed, but some are lost under rapid input.

## Phase 1: Understand the Symptom

### Investigation Areas
1. UART receive buffer overflow?
2. Ring buffer in input handling?
3. Interrupt handler dropping characters?
4. Userspace read not consuming fast enough?

## Investigation Log
| Time | Action | Result |
|------|--------|--------|
| 12:40 | Read TEAM_139, TEAM_149 | Prior issues (GTK focus, IRQ starvation) fixed |
| 12:42 | Analyzed input.rs | Found `let _ = push()` discards overflow |
| 12:43 | Analyzed console.rs | Same issue - push() return ignored |
| 12:44 | Checked RingBuffer | push() returns false when full |

## Phase 2: Hypotheses

### Hypothesis 1: Buffer Overflow with Silent Drops (CONFIRMED - HIGH)
- **Evidence:** 
  - `kernel/src/input.rs:82` - `let _ = KEYBOARD_BUFFER.lock().push(c);`
  - `levitate-hal/src/console.rs:34` - `RX_BUFFER.lock().push(byte);`
  - RingBuffer.push() returns `false` when full, but return value is discarded
- **Buffer sizes:**
  - KEYBOARD_BUFFER: 256 chars (VirtIO input)
  - RX_BUFFER: 1024 bytes (UART)
- **Why random:** Depends on timing between input rate vs sys_read consumption rate

### Hypothesis 2: Need separate keyboard crate (RULED OUT)
- Architecture is correct - VirtIO input + UART both work
- Problem is buffer management, not driver structure

## Root Cause (CONFIRMED)

**Silent buffer overflow during rapid input.**

1. User smashes keyboard → characters arrive rapidly
2. VirtIO input ISR calls `input::poll()` → pushes to 256-char buffer
3. If userspace `sys_read` doesn't drain fast enough → buffer fills
4. `push()` returns `false` → **silently discarded** with `let _ =`
5. Characters lost

**Causal Chain:**
```
Keyboard smash → VirtIO events → poll() → KEYBOARD_BUFFER.push() 
                                              ↓
                                    Buffer full? → drop char silently
                                              ↓
                        sys_read (busy-yield loop) drains too slow
```

## Recommended Fix

**Minimal fix (≤5 UoW):**

1. **Increase buffer sizes:**
   - KEYBOARD_BUFFER: 256 → 1024 chars
   - (RX_BUFFER already 1024)

2. **Add overflow warning** (for debugging):
   ```rust
   if !KEYBOARD_BUFFER.lock().push(c) {
       // Log overflow if verbose mode enabled
   }
   ```

3. **Optional: Ring buffer overwrite mode** - Instead of dropping new chars, overwrite oldest (common for input buffers)

**NOT needed:** Separate keyboard crate. Architecture is fine.

## Decision: Fix or Plan?

- Fix is **≤5 UoW**, **≤20 lines**
- Low risk (buffer size change)
- High confidence in root cause

**→ Fix immediately**

## Root Cause (UPDATED - ACTUAL)

**The initial buffer overflow hypothesis was WRONG.**

Real root cause: **Kernel accessing userspace VA directly**.

In `sys_read()`, the kernel was doing:
```rust
let user_buf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, max_read) };
```

The `buf` is a userspace virtual address (mapped via TTBR0). The kernel runs with TTBR1.
**Userspace addresses are NOT accessible from kernel context!** Writing to them wrote to
unmapped/wrong memory, resulting in null bytes.

## Fix Applied (FINAL)

**Changes made:**
1. `kernel/src/task/user_mm.rs`:
   - Added `user_va_to_kernel_ptr()` - translates user VA → physical → kernel VA

2. `kernel/src/syscall.rs`:
   - Added `write_to_user_buf()` helper that uses page table translation
   - Changed `poll_input_devices()` to translate each byte's address before writing
   - Fixed `sys_read()` to pass ttbr0 and buffer address for proper translation

3. `xtask/src/tests/keyboard_input.rs`:
   - **NEW regression test** that verifies keyboard input works with NO false positives
   - Tests single chars, burst input, and repeated chars
   - FAILS if ANY character is dropped

## Test Results
```
✅ TEST 1: Single character input PASSED
✅ TEST 2: Rapid burst input PASSED  
✅ TEST 3: Very rapid repeated characters PASSED
✅ Behavior test PASSED
✅ Serial test PASSED
```

## Handoff Checklist
- [x] Project builds cleanly
- [x] All tests pass (keyboard, behavior, serial)
- [x] Behavioral regression tests pass
- [x] Team file updated
- [x] Root cause confirmed: Kernel accessing user VA directly (page table mismatch)
- [x] Fix implemented: Page table translation for user buffer access
- [x] Regression test added: `cargo xtask test keyboard`
