# LevitateOS TODO List

Global tracking of incomplete tasks per Rule 11.

---

## Critical

### VirtIO GPU Display Not Active — TEAM_087
**File:** `kernel/src/gpu.rs`, VirtIO GPU driver  
**Symptom:** QEMU window shows "Display output is not active"  
**Root Cause:** VirtIO GPU scanout not properly configured. The framebuffer exists but is not mapped to the display output.

**What Works:**
- Serial console (type in terminal where QEMU runs)
- Kernel boots fully, shows "[SUCCESS] LevitateOS System Ready"
- VirtIO keyboard input (echoes to serial)

**What Doesn't Work:**
- QEMU graphical window (shows "Display output is not active")
- Mouse cursor in QEMU window
- Any visual output on GPU

**Investigation Needed:**
1. Check `virtio-drivers` GPU setup for `set_scanout()` call
2. Verify `VIRTIO_GPU_CMD_SET_SCANOUT` is sent during init
3. May need to configure display scanout after framebuffer creation

**Workaround:** Use serial console - all kernel functionality works there.

---

## High Priority

*None*

---

## Medium Priority

### Boot Hijack Code Still Present — TEAM_081
**File:** `kernel/src/main.rs:612`  
**Description:** TEAM_073's demo code `run_from_initramfs("hello")` is commented out but not removed. Should be cleaned up once userspace shell is working properly.

---

## Low Priority

### Stale Golden Tests — TEAM_081
**File:** `tests/golden_boot.txt`  
**Description:** Behavior tests reflect old boot sequence. Need update after Phase 8b is complete.

---

## Completed

- [x] TEAM_082: Linker script conflict for userspace builds
- [x] TEAM_083: UART debug spam from stale binary
- [x] TEAM_083: Timer "T" debug output flooding console
- [x] TEAM_083: GPU reference compilation error
- [x] TEAM_086: GPU Display Deadlock API — Refactored Display to accept `&mut GpuState`
- [x] TEAM_087: Enabled dual console callback (but GPU display still not active)
- [x] TEAM_087: Fixed per-println flush causing kernel hang
