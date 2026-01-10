# Phase 1: Understanding and Scoping

**Team:** TEAM_337  
**Purpose:** Make the bug understandable and bounded before touching code.

## Bug Summary

- **Description:** AArch64 QEMU shows "Display output is not active" - VirtIO GPU not rendering
- **Severity:** High - No visual output on AArch64 platform
- **Impact:** Complete loss of display functionality on AArch64

## Reproduction Status

- **Reproducible:** Yes
- **Steps:**
  1. Build kernel: `cargo xtask build kernel --arch aarch64`
  2. Run: `cargo xtask run --arch aarch64`
  3. Observe QEMU window shows "Display output is not active"
- **Expected:** Terminal with shell prompt visible
- **Actual:** Black screen with QEMU message

## Context

### User Hint
> "for aarch I remember that it went from gpu to pci to machine I guess"

This suggests there may be a different path for GPU on AArch64:
- GPU device might be on a **PCI bus** even on AArch64 QEMU virt machine
- Or there's machine-level configuration needed

### Code Areas Suspected

1. `kernel/src/virtio.rs` - `detect_gpu_transport()` function for AArch64
2. `kernel/src/gpu.rs` - GPU initialization
3. `crates/gpu/src/lib.rs` - los_gpu driver
4. QEMU configuration - How VirtIO GPU is attached

### Recent Changes (TEAM_336)

- Made `los_gpu::Gpu` generic over transport type
- Added arch-specific `GpuTransport` type aliases
- Added `detect_gpu_transport()` for MMIO scanning on AArch64

## Open Questions

1. **Is VirtIO GPU on MMIO or PCI on AArch64 QEMU virt?**
   - Need to check QEMU machine configuration
   - Need to check device tree for GPU location

2. **Is the MMIO GPU being detected at all?**
   - Need to add debug logging to see if GPU is found

3. **Is the GPU initialization failing silently?**
   - Need to check what error is returned from `Gpu::new()`

4. **What does the device tree show for GPU?**
   - DTB might specify GPU location differently

## Phase 1 Steps

### Step 1: Check QEMU VirtIO GPU Configuration
- How is `-device virtio-gpu-pci` vs `-device virtio-gpu-device` different?
- What does the QEMU command line use for AArch64?

### Step 2: Add Debug Logging
- Log when GPU transport is detected
- Log the device type found at each MMIO slot
- Log any errors from GPU initialization

### Step 3: Check Historical Context
- Look at previous team files for GPU-related work
- Check if there was a working AArch64 GPU configuration before
