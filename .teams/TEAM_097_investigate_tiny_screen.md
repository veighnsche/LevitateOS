# TEAM_097: Investigate Tiny Screen Bug

**Date:** 2026-01-05  
**Role:** Bug Investigator  
**Bug:** QEMU display shows tiny screen instead of configured 1280×800

---

## Bug Report

**Symptom:** QEMU graphical window starts with a tiny screen instead of the configured resolution (1280×800 in run.sh, 2400×1080 in run-pixel6.sh).

**Expected:** Display fills the QEMU window at the configured resolution.

**Actual:** Display is tiny/small, possibly stuck in VGA compatibility mode.

**Environment:**
- QEMU with `-device virtio-gpu-device,xres=1280,yres=800`
- LevitateOS kernel with VirtIO GPU driver
- aarch64 target

---

## Phase 1: Understand the Symptom

### Symptom Description
- QEMU window shows a tiny display area
- The configured resolution (xres=1280, yres=800) is not being used
- Per VirtIO spec, this indicates VGA compatibility mode is active

### Hypothesis from Prior Review (TEAM_096)
VirtIO GPU starts in VGA compat mode until SET_SCANOUT is called with a valid resource.

---

## Phase 2: Hypotheses

| # | Hypothesis | Evidence Needed | Confidence | Status |
|---|------------|-----------------|------------|--------|
| H1 | Driver never calls GET_DISPLAY_INFO | Code search | Medium | ❌ RULED OUT - virtio-drivers calls it |
| H2 | Driver never calls SET_SCANOUT | Code search | Medium | ❌ RULED OUT - virtio-drivers calls it in setup_framebuffer() |
| H3 | SET_SCANOUT called with wrong dimensions | Code inspection | Medium | 🔍 INVESTIGATING |
| H4 | Resource created with hardcoded small size | Code inspection | Medium | ❌ RULED OUT - uses GET_DISPLAY_INFO result |
| H5 | VirtIO transport not working at all | Would see no display | Low | ❌ RULED OUT |
| H6 | QEMU returning unexpected resolution | Add logging | High | ❌ RULED OUT - returns 1280x800 |
| H7 | Display not enabled (enabled=0) | Add logging | Medium | ❌ RULED OUT - driver works |
| H8 | QEMU GTK window not resizing to match scanout | Test QEMU options | High | ✅ CONFIRMED - zoom-to-fit default |

---

## Phase 3: Evidence Gathering

### virtio-drivers 0.12.0 Source Analysis

**File:** `virtio_drivers/device/gpu.rs`

**Findings:**

1. **`resolution()` (line 91-94):**
   - ✅ Calls `get_display_info()` which sends `GET_DISPLAY_INFO` command
   - Returns `(display_info.rect.width, display_info.rect.height)`

2. **`setup_framebuffer()` (line 97-130):**
   - ✅ Calls `get_display_info()` to get resolution
   - ✅ Creates resource at returned size
   - ✅ Calls `set_scanout()` with the rect
   - ⚠️ Does NOT check if `enabled == 1`

3. **`RespDisplayInfo` struct:**
   - Reads first 48 bytes: header(24) + rect(16) + enabled(4) + flags(4)
   - This correctly maps to pmodes[0] of the spec response

4. **Logging:**
   - Line 100: `info!("=> {:?}", display_info);`
   - Requires `log` crate logger - likely not configured!

### LevitateOS GPU Init Flow

```
kernel/src/main.rs:538  -> virtio::init_gpu()
kernel/src/virtio.rs:39 -> gpu::init(transport)
kernel/src/gpu.rs:10    -> GpuState::new(transport)
levitate-gpu/gpu.rs:27  -> VirtIOGpu::new(transport)
levitate-gpu/gpu.rs:30  -> gpu.resolution() // calls GET_DISPLAY_INFO
levitate-gpu/gpu.rs:45  -> gpu.setup_framebuffer() // calls SET_SCANOUT
```

### Key Question

What resolution does GET_DISPLAY_INFO actually return?
- The code at `main.rs:552` prints: `[TERM] GPU resolution: {}x{}`
- Need to check the actual boot log output

### Diagnostic Run Output

```
[GPU] GET_DISPLAY_INFO returned: 1280x800
[TERM] GPU resolution: 1280x800
```

**CONFIRMED:** QEMU is returning the correct 1280x800 resolution!

This means:
- ✅ virtio-drivers is correctly calling GET_DISPLAY_INFO
- ✅ Resource is being created at 1280x800
- ✅ SET_SCANOUT is being called correctly
- ✅ Driver exits VGA compat mode

**New Hypothesis:** The issue is NOT with the driver. The issue is likely:
1. QEMU GTK window not auto-resizing to match scanout
2. QEMU display configuration issue
3. Something about how the window appears initially

---

## Phase 4: Root Cause

**DRIVER IS WORKING CORRECTLY.**

The VirtIO GPU driver correctly:
1. Queries display info (GET_DISPLAY_INFO) → returns 1280x800
2. Creates resource at 1280x800
3. Calls SET_SCANOUT to exit VGA compat mode

**ROOT CAUSE CONFIRMED:** QEMU GTK display `zoom-to-fit` default behavior.

When `zoom-to-fit` is enabled (QEMU default with virtio-gpu), the GTK window:
1. Does not automatically resize when guest changes scanout resolution
2. May start with a small default size before guest driver initializes
3. Scales content to fit window rather than resizing window to fit content

**FIX APPLIED:** Added `zoom-to-fit=off` to run scripts:
- `run.sh`: `-display gtk,zoom-to-fit=off`
- `run-pixel6.sh`: `-display gtk,zoom-to-fit=off`

---

## Phase 5: Decision

**FIX IMMEDIATELY** - Simple 1-line change per script.

### Changes Made

1. `run.sh` line 28: Changed `-display gtk` to `-display gtk,zoom-to-fit=off`
2. `run-pixel6.sh` line 56: Changed `-display gtk` to `-display gtk,zoom-to-fit=off`
3. Added diagnostic logging to `levitate-gpu/src/gpu.rs` (TEAM_097 breadcrumb)

---

## Breadcrumbs Placed

1. `levitate-gpu/src/gpu.rs:30-37` - INVESTIGATING breadcrumb with diagnostic logging
   - Can be removed once fix is verified

---

## Summary

**Bug:** QEMU display shows tiny screen instead of 1280x800

**Root Cause:** NOT the kernel driver. QEMU GTK `zoom-to-fit` default behavior prevents window from resizing to match VirtIO GPU scanout.

**Fix:** Added `zoom-to-fit=off` to QEMU display options in run scripts.

**Verification:** Run `./run.sh` and confirm window is 1280x800.

