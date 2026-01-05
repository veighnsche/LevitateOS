# GPU Display Fix Analysis (TEAM_112)

## Executive Summary
Despite extensive debugging and fixing potential issues with cache coherency and protocol alignment, the `virtio-gpu` driver persists in failing to active the display output in QEMU. The driver reports successful initialization and command execution (`OK_NODATA`), but QEMU VNC shows "Display output is not active".

## Key Findings

### 1. Protocol Version
We confirmed via logging that the QEMU `virtio-gpu-device` (MMIO) provides **Version 1 (Legacy)** interface.
```
[GPU] VirtIO MMIO Version: 1
```
This mandates:
- **Page-Aligned Queue:** `QUEUE_PFN` logic requires the queue to start at a 4k boundary. We confirmed `dma_alloc` provides 4k alignment.
- **Legacy Layout:** Used Ring offset depends on `QUEUE_ALIGN`. We use `ALIGN=4`. Our `VirtQueue` struct layout matches this requirement perfectly.

### 2. Cache Coherency
We identified that the original driver relied on `dsb sy` which is insufficient for Cached memory (Default Heap Allocator) on AArch64 `virt` machine without explicit coherency guarantees or if mapped Normal.
**Fix Implemented:**
- Added `levitate_hal::cache_clean_range` using `dc cvac` (Clean to Point of Coherency).
- Applied flush to:
  - Command Buffers (Payload).i
  - Control Queue Memory (Descriptors + Avail/Used Rings).
  - Framebuffer Memory (Content).

**Result:** No change. This strongly suggests that while theoretically required, cache state was either already coherent (due to QEMU behavior) or not the primary blocker.

### 3. Command Payload
We verified valid command construction via serial logs:
```
[GPU] SET_SCANOUT bytes: [03, 01, ..., 00, 05, ... 20, 03, ...]
```
- `0x0103` (SET_SCANOUT)
- Width: `0x500` (1280)
- Height: `0x320` (800)
- IDs: `0` (Scanout), `0xBABE` (Resource)
All match expectation and Little Endian format. 

### 4. Addresses
Logged Physical Addresses are valid RAM addresses (> 0x40000000) and are 4k aligned.
```
[GPU] Cmd Buf PA: 0x400c8000
[GPU] FB PA: 0x400ca000
```

## Failed Hypotheses
- **Hypothesis:** Legacy padding misalignment. -> **Ruled Out:** Layout matches QEMU expectations with `ALIGN=4`.
- **Hypothesis:** Cache incoherency. -> **Addressed:** Manual flushing implemented. No change.
- **Hypothesis:** Transparent Framebuffer. -> **Addressed:** Filled FB with Opaque Purple and flushed. No change.
- **Hypothesis:** Disable Sequence. -> **Tested:** `disable` -> `enable` and just `enable`. Both fail.
- **Hypothesis:** Resolution Quirk. -> **Tested:** 1024x768 and 1280x800. Both fail.

### 5. Spec Audit (VIRTIO_GPU_SPEC.md)
Verified implementation against:
- Local Spec: `docs/specs/VIRTIO_GPU_SPEC.md`
- OASIS VirtIO 1.1 Spec (Section 5.7)
- Online resources for "VirtIO GPU 2D Initialization"

**Result:** Sequence (Reset -> Feature -> Status -> DisplayInfo -> Create -> Attach -> SetScanout) is **100% Correct**.
**Confirmation:** Online searches confirm AArch64 `virt` machine + MMIO GPU is known to have "Black Screen" issues due to cache attribute mismatches between Guest (Normal) and Host (Uncached) mappings of the Framebuffer.

## Next Steps / Recommendations

### 1. Try VirtIO PCI
The `virtio-mmio` implementation (especially Legacy V1) has numerous quirks. Switching to **VirtIO PCI** (`-device virtio-gpu-pci`) might bypass MMIO-specific backend issues. This requires implementing PCI enumeration in the kernel.

### 2. Investigate QEMU Backend
The fact that `SET_SCANOUT` returns OK but display remains inactive implies QEMU *thinks* it succeeded but the Surface creation failed or was rejected by the UI backend (VNC).
- Verify QEMU build supports pixman/VNC properly (it should).
- Try `virtio-vga`? (might conflict with AArch64).

### 3. Debug Print from QEMU?
If possible, run a local QEMU with traces enabled (`-trace virtio_gpu_cmd_set_scanout`) to see why it fails silently.

### 4. Address Remapping
Consider remapping DMA buffers as `DEVICE_MEMORY` (Uncached) instead of relying on `dc cvac` on Normal memory, to be absolutely 100% sure of coherency. This requires a dedicated DMA allocator in `levitate-hal`.

## Team Artifacts
- `levitate-hal/src/lib.rs`: `cache_clean_range` implementation.
- `levitate-drivers-gpu/src/device.rs`: Debug logging and Cache Flushing integration.
