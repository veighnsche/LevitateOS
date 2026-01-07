# TEAM_107: levitate-drivers-gpu Additional Issues

**Created:** 2026-01-05
**Status:** Open
**Priority:** Medium

---

## Context

TEAM_106 implemented the VirtQueue DMA fix:
1. ✅ Added `#[repr(C, align(16))]` to Descriptor struct
2. ✅ Added `#[repr(C, align(16))]` to VirtQueue struct  
3. ✅ Changed VirtioGpu to allocate queue via `dma_alloc()` instead of Box

TEAM_107 attempted to migrate the kernel from `levitate-gpu` (virtio-drivers wrapper) to `levitate-drivers-gpu` (our custom implementation with fixed VirtQueue).

---

## Issue

**The migration failed.** Even with the VirtQueue DMA fix, `levitate-drivers-gpu` still times out:

```
[GPU] Init failed: Timeout
```

This means there are **additional issues** in `levitate-drivers-gpu` beyond the VirtQueue DMA alignment problem.

---

## Suspected Areas

1. **MmioTransport implementation** (`levitate-virtio/src/transport.rs`)
   - May have differences from virtio-drivers' MmioTransport
   - Queue setup sequence might differ

2. **VirtQueue command flow** (`levitate-virtio/src/queue.rs`)
   - The command send/receive loop may have issues
   - Descriptor chain handling might differ

3. **GPU protocol handling** (`levitate-drivers-gpu/src/device.rs`)
   - GET_DISPLAY_INFO command timeout
   - May need to check command buffer layout

4. **Physical address translation**
   - queue.addresses() returns virtual addresses
   - Translation to physical addresses may have issues

---

## What Works

`levitate-gpu` (using virtio-drivers crate) works correctly because:
- virtio-drivers has a battle-tested VirtIO implementation
- It handles all the DMA, alignment, and protocol details correctly

---

## Recommended Next Steps

1. **Compare implementations** - Side-by-side diff of:
   - `levitate-virtio/src/queue.rs` vs `virtio-drivers/src/queue.rs`
   - `levitate-virtio/src/transport.rs` vs `virtio-drivers/src/transport/mmio.rs`

2. **Add debugging** - Add serial output in levitate-drivers-gpu to trace:
   - What physical addresses are being set in device registers
   - What values are being written to avail_idx
   - What the used_idx value is after notify

3. **Test with simpler device** - Try levitate-virtio with a simpler device (e.g., entropy) to isolate VirtQueue issues from GPU-specific issues

---

## Current State

- `levitate-gpu` (virtio-drivers): **Working** ✅
- `levitate-drivers-gpu` (custom): **Broken** ❌ - times out even with DMA fix
- VirtQueue DMA fix (TEAM_106): **Implemented** ✅ - but not sufficient

---

## Decision

Keep using `levitate-gpu` until additional issues are resolved.
The VirtQueue DMA fix is in place for when these issues are resolved.
