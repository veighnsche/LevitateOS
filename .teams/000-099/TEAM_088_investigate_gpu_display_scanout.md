# Team Log - TEAM_088

## Bug: VirtIO GPU "Display output is not active"

### Status: 🔴 UNRESOLVED - Documented for Future Teams

---

## Key Finding: Display Pipeline WORKS

**Proof:** Red screen flash appears during boot, then immediately goes inactive.

This confirms:
- ✅ `VirtIOGpu::new()` succeeds
- ✅ `setup_framebuffer()` succeeds (includes `set_scanout()`)
- ✅ Framebuffer fill works (red pixels visible)
- ✅ `gpu.flush()` works (content transfers to host)

**The VirtIO GPU driver is correctly implemented.**

---

## Root Cause: Unknown - Something Resets Scanout

Something in the boot sequence AFTER `gpu::init()` is disabling/resetting the scanout.

### Ruled Out
- ❌ `console_gpu::clear()` - disabled, still flashes
- ❌ `set_secondary_output()` - disabled, still flashes
- ❌ Missing flush - flush is called and succeeds
- ❌ Pixel format - red appears correctly

### Remaining Suspects
1. **Another VirtIO device init** - Block/Net/Input devices might affect GPU
2. **Timer or interrupt initialization** - GIC/timer setup might reset GPU
3. **QEMU internal behavior** - Possible QEMU bug with virtio-gpu-device on virt machine
4. **Memory mapping conflict** - DMA memory might be reclaimed

---

## Reproduction

```bash
./run.sh
# Observe: Brief red flash, then "Display output is not active"
```

### Test Code Location
`kernel/src/gpu.rs` - init() fills screen red and flushes at line ~46-62

---

## BREADCRUMBS Left in Code

```
// TEAM_088: Fill with BRIGHT RED to test display pipeline
```

---

## Recommended Next Steps

1. **Add delay after red fill** - Test if timing issue
2. **Disable other VirtIO devices** - Test if Block/Net/Input affect GPU
3. **Try `virtio-vga` instead of `virtio-gpu-device`**
4. **Check QEMU source** for when "Display output is not active" is set
5. **Add more tracing** around interrupt/timer init

---

## Files Modified

| File | Change |
|------|--------|
| `kernel/src/gpu.rs` | Added red fill test, fixed BGRA pixel order |
| `kernel/src/main.rs` | Temporarily disabled clear() and secondary callback |

---

## Handoff Checklist

- [x] Team file updated with findings
- [x] Breadcrumbs in code
- [ ] Issue NOT resolved
- [x] Next steps documented
