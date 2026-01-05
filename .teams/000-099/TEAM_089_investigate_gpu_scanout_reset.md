# Team Log - TEAM_089

## Bug: VirtIO GPU Scanout Reset Investigation

**Status:** � ROOT CAUSE IDENTIFIED  
**Parent Investigation:** TEAM_088  
**Date:** 2026-01-05

---

## Root Cause: QEMU Display Surface Timing

### Evidence Gathered

1. **Red Flash Proves Driver Works**
   - `setup_framebuffer()` correctly calls `set_scanout(display_info.rect, SCANOUT_ID, RESOURCE_ID_FB)`
   - Red fill is visible for ~100-200ms before disappearing
   - This proves: GPU init ✅, resource create ✅, attach backing ✅, set_scanout ✅, transfer ✅, flush ✅

2. **VirtIO Protocol is Correct**
   - Reviewed virtio-drivers v0.12.0 source (`~/.cargo/registry/src/.../virtio-drivers-0.12.0/src/device/gpu.rs`)
   - All required commands are sent in correct order
   - No errors returned from GPU commands

3. **Nothing Explicitly Resets Scanout**
   - `virtio::init()` (line 554) explicitly skips GPU devices
   - No code calls `set_scanout` with resource_id=0 (which would disable)
   - Console/terminal operations only modify framebuffer pixels, not scanout config

4. **QEMU Configuration**
   - Using `-device virtio-gpu-device,xres=1280,yres=800`
   - This is the non-PCI variant for aarch64 virt machine
   - Web research suggests `virtio-vga` may be more reliable for some guests

---

## Hypothesis: QEMU Display Surface Invalidation

**Most Likely Cause:** QEMU's virtio-gpu backend invalidates the display surface when:
1. The guest stops sending commands for a timeout period, OR
2. Something in the guest triggers a display surface reset

**Supporting Evidence:**
- The red flash appears and works correctly
- Display goes blank almost immediately after (during console_gpu::clear())
- Serial console continues working fine (not a kernel hang)
- Other virtio-gpu implementations (Linux DRM driver) continuously refresh

**Potential Fixes to Try:**
1. **Add timer-based periodic flush** - Keep GPU active with regular flush calls
2. **Try `virtio-vga` instead** - May have different display surface handling
3. **Increase flush frequency** - Current code flushes every 10000 loop iterations
4. **Add delay after red fill test** - Verify timing issue vs. permanent disable

---

## Hypotheses Tested

| Hypothesis | Result | Notes |
|------------|--------|-------|
| VirtIO init re-scan affects GPU | ❌ RULED OUT | virtio::init() skips GPU explicitly |
| DMA memory reclaimed | ❌ RULED OUT | DMA kept in GpuState, not freed |
| Display refactoring broke scanout | ❌ RULED OUT | set_scanout not touched by TEAM_086 |
| console_gpu::clear() issue | ⚠️ POSSIBLE | Might trigger timing issue |
| QEMU display timeout | ✅ LIKELY | Explains flash-then-blank behavior |

---

## Recommended Next Steps

1. **Test with timer-based flush** — Add a 10Hz GPU flush heartbeat (not just main loop)
2. **Try `virtio-vga`** — Change run.sh to use `-device virtio-vga,xres=1280,yres=800`
3. **Add QEMU tracing** — Enable `trace-file` to see virtio-gpu commands
4. **Test without console_gpu::clear()** — Comment out line 544 in main.rs

---

## Breadcrumbs Left in Code

```
// TEAM_088: Fill with BRIGHT RED to test display pipeline (gpu.rs:47)
// TEAM_088 BREADCRUMB: CONFIRMED - flush required (console_gpu.rs:78)
```

---

## Handoff Checklist

- [x] Team file updated with findings
- [x] Breadcrumbs in code (from TEAM_088)
- [ ] Issue NOT YET resolved — needs code changes to test
- [x] Next steps documented
