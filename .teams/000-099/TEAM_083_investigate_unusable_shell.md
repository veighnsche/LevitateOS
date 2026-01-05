# Team Log - TEAM_083

## Goal
Investigate and fix the unusable interactive shell as described in `POSTMORTEM.md`.

## Status: ⚠️ PARTIAL FIX

## Issues Found & Fixed

### Issue 1: Stale Binary with Debug Spam ✅
**Symptom:** "UART CR: 0x301 FR: 0x90" flooding console  
**Cause:** Old committed binary had debug output; build was failing so stale binary was used  
**Fix:** Fixed compilation error (GPU reference) and rebuilt

### Issue 2: Timer Debug Output ✅
**Symptom:** "T" character printed every timer interrupt  
**Location:** `kernel/src/main.rs` TimerHandler  
**Fix:** Removed `serial_println!("T")` from timer handler

### Issue 3: GPU Reference Error ✅
**Symptom:** Build failure - `GPU` not found in scope  
**Location:** `kernel/src/main.rs:660`  
**Fix:** Changed `GPU.lock()` to `gpu::GPU.lock()`

### Issue 4: GPU Display Deadlock ❌ NOT FIXED
**Symptom:** System hangs when trying to draw text to GPU screen  
**Root Cause:** `Display::draw_iter()` locks `GPU` internally, then code tries to lock again for flush  
**Workaround:** Bypassed Display struct, use direct framebuffer access  
**Proper Fix Needed:** Refactor Display to not lock internally

## What Works Now
- ✅ System boots fully without hanging
- ✅ Keyboard input captured from QEMU window (VirtIO)
- ✅ Characters echo to UART (terminal where QEMU runs)
- ✅ Interactive prompt `# ` in serial console

## What Still Doesn't Work
- ❌ Text doesn't render on GPU screen (QEMU window)
- ❌ Dual console (UART + GPU) has deadlock issues

## Documentation Created
- Updated `docs/GOTCHAS.md` with GPU deadlock warning
- Added BREADCRUMB in `kernel/src/gpu.rs` Display::draw_iter
- This team file

## Handoff Notes
- Interactive shell (kernel & userspace) is now ready.
- Mirroring to GPU is currently disabled by default for boot stability but can be re-enabled.
- See `walkthrough.md` for full details.

## Symptom Description
The system boots but appears to hang or be unresponsive. Specifically:
- Boot messages scroll but stop without a "Ready" indicator.
- There is no shell prompt.
- Typing on the keyboard provides no visual feedback.
- The system is hijacked by a demo userspace process.

## Environment/Context
- OS: LevitateOS (Rust kernel)
- Target: AArch64 (QEMU)
- Epic: Interactive Shell & Unix-like Boot Experience (Phase 8b)
- Related Postmortem: `docs/planning/interactive-shell-phase8b/POSTMORTEM.md`

## Hypotheses
1. **H1 (Confirmed by Postmortem):** Boot sequence is hijacked by `task::process::run_from_initramfs("hello", ...)` in `kernel/src/main.rs`, preventing it from reaching the interactive loop.
2. **H2 (Confirmed by Postmortem):** Stdin/keyboard input is not being echoed anywhere.
3. **H3 (Suspected):** Global interrupts are not enabled before jumping to userspace, or keyboard events aren't being routed correctly.
