# TEAM_096: Review of VIRTIO_GPU_SPEC.md

**Date:** 2026-01-05  
**Role:** Spec Verification + Usability Review  
**Goal:** Verify spec accuracy and assess suitability for showing a terminal on screen

---

## Executive Summary

### Is the Spec Correctly Copied?

**YES** - The `VIRTIO_GPU_SPEC.md` accurately represents VirtIO 1.1 Section 5.7. All structures, commands, and constants match the official specification.

### Is it Right for "Terminal on Screen"?

**YES** - The 2D mode spec is *exactly* what's needed:
- Simple framebuffer → display pipeline
- No 3D complexity
- Guest renders to RAM → DMA to host → Flush to screen

### Why Does QEMU Start with a Tiny Screen?

**Answer:** VGA Compatibility Mode. QEMU's virtio-gpu starts in VGA compat mode until the guest driver calls `SET_SCANOUT`. The tiny screen is the VGA fallback display.

**Fix:** The driver MUST:
1. Call `GET_DISPLAY_INFO` to query the actual resolution QEMU is offering (1280×800 per `run.sh`)
2. Create a resource at that size
3. Call `SET_SCANOUT` to switch to native VirtIO mode

---

## Spec Verification (VirtIO 1.1 vs VIRTIO_GPU_SPEC.md)

### Configuration Space ✅

| Field | Official | VIRTIO_GPU_SPEC.md |
|-------|----------|-------------------|
| events_read | le32 | ✅ u32 |
| events_clear | le32 | ✅ u32 |
| num_scanouts | le32 | ✅ u32 |
| reserved | le32 | ✅ u32 |

### Command Types ✅

| Command | Official Value | VIRTIO_GPU_SPEC.md |
|---------|---------------|-------------------|
| GET_DISPLAY_INFO | 0x0100 | ✅ 0x0100 |
| RESOURCE_CREATE_2D | 0x0101 | ✅ 0x0101 |
| RESOURCE_UNREF | 0x0102 | ✅ 0x0102 |
| SET_SCANOUT | 0x0103 | ✅ 0x0103 |
| RESOURCE_FLUSH | 0x0104 | ✅ 0x0104 |
| TRANSFER_TO_HOST_2D | 0x0105 | ✅ 0x0105 |
| RESOURCE_ATTACH_BACKING | 0x0106 | ✅ 0x0106 |
| RESOURCE_DETACH_BACKING | 0x0107 | ✅ 0x0107 |
| UPDATE_CURSOR | 0x0300 | ✅ 0x0300 |
| MOVE_CURSOR | 0x0301 | ✅ 0x0301 |

### Pixel Formats ✅

| Format | Official Value | VIRTIO_GPU_SPEC.md |
|--------|---------------|-------------------|
| B8G8R8A8_UNORM | 1 | ✅ 1 |
| B8G8R8X8_UNORM | 2 | ✅ 2 |
| A8R8G8B8_UNORM | 3 | ✅ 3 |
| X8R8G8B8_UNORM | 4 | ✅ 4 |
| R8G8B8A8_UNORM | 67 | ✅ 67 |
| X8B8G8R8_UNORM | 68 | ✅ 68 |
| A8B8G8R8_UNORM | 121 | ✅ 121 |
| R8G8B8X8_UNORM | 134 | ✅ 134 |

### Data Structures ✅

All struct layouts verified:
- `virtio_gpu_ctrl_hdr` (24 bytes)
- `virtio_gpu_rect` (16 bytes)
- `virtio_gpu_resource_create_2d` (40 bytes)
- `virtio_gpu_set_scanout` (48 bytes)
- `virtio_gpu_resource_flush` (48 bytes)
- `virtio_gpu_transfer_to_host_2d` (56 bytes)
- `virtio_gpu_resource_attach_backing` (32 bytes)
- `virtio_gpu_mem_entry` (16 bytes)
- `virtio_gpu_resp_display_info` (408 bytes)

---

## Tiny Screen Root Cause Analysis

### Official Spec Quote

From VirtIO 1.1 Section 5.7.7:
> "VGA compatibility: PCI region 0 has the linear framebuffer, standard vga registers are present. **Configuring a scanout (VIRTIO_GPU_CMD_SET_SCANOUT) switches the device from vga compatibility mode into native virtio mode.** A reset switches it back into vga compatibility mode."

### Current QEMU Configuration

```bash
# From run.sh line 29:
-device virtio-gpu-device,xres=1280,yres=800
```

QEMU is offering 1280×800, but the guest isn't asking for it!

### Required Driver Initialization Sequence

```
1. VirtIO device reset/init (standard)
2. GET_DISPLAY_INFO → Returns enabled scanouts with preferred resolution
   - pmodes[0].r.width = 1280 (from QEMU xres=)
   - pmodes[0].r.height = 800 (from QEMU yres=)
   - pmodes[0].enabled = 1
3. RESOURCE_CREATE_2D (1280×800, B8G8R8A8_UNORM)
4. RESOURCE_ATTACH_BACKING (guest framebuffer memory)
5. SET_SCANOUT (scanout_id=0, resource_id=N) ← **THIS EXITS VGA MODE**
6. Display is now active at 1280×800
```

---

## Suitability for Terminal Display

### What You Need for a Terminal

1. **Framebuffer memory** (guest-side) - Already have this
2. **Bitmap font rendering** - `embedded-graphics` handles this
3. **Periodic flush** - Transfer + Flush the dirty region

### The 2D Pipeline

```
[Guest RAM Framebuffer]
        │
        ▼ TRANSFER_TO_HOST_2D (dirty rect)
[Host-side Resource]
        │
        ▼ RESOURCE_FLUSH
[QEMU Display Window]
```

**This is the simplest possible path.** No shaders, no 3D, no complex state.

### Comparison to Alternatives

| Approach | Complexity | Performance | LevitateOS Need |
|----------|------------|-------------|-----------------|
| VirtIO GPU 2D | Low | Good | ✅ Perfect |
| VirtIO GPU 3D (virgl) | High | Best | ❌ Overkill |
| Legacy VGA framebuffer | Lowest | Poor | ❌ Limited |
| PL110/PL111 LCD | Low | Good | ❌ Not portable |

---

## CRITICAL: What's Missing from the Current Driver?

Looking at `run.sh`, QEMU is configured correctly. The problem is in the driver:

### Likely Issues

1. **Not calling GET_DISPLAY_INFO** - Driver hardcodes resolution instead of querying
2. **SET_SCANOUT never called** - Display stays in VGA fallback mode
3. **Wrong resource dimensions** - Creates a resource smaller than QEMU expects

### Minimum Viable Fix

```rust
// 1. Query display info
let display_info = driver.get_display_info().await?;
let width = display_info.pmodes[0].r.width;   // 1280 from QEMU
let height = display_info.pmodes[0].r.height; // 800 from QEMU

// 2. Create matching resource
let resource_id = driver.create_resource_2d(width, height, B8G8R8A8_UNORM).await?;

// 3. Attach guest memory
driver.attach_backing(resource_id, &framebuffer_pages).await?;

// 4. **THIS IS THE CRITICAL STEP** - switches out of VGA mode
driver.set_scanout(0, resource_id, 0, 0, width, height).await?;

// 5. Now rendering works
loop {
    draw_to_framebuffer();
    driver.transfer_to_host_2d(resource_id, dirty_rect).await?;
    driver.resource_flush(resource_id, dirty_rect).await?;
}
```

---

## Recommendations

### Immediate (Fix Tiny Screen)

1. Verify driver calls `GET_DISPLAY_INFO` and uses returned dimensions
2. Verify driver calls `SET_SCANOUT` with correct rectangle
3. Add logging to see what resolution the driver thinks it's using

### Spec Document Updates

The spec document is accurate but could add:

1. **Troubleshooting Section** - "Display shows tiny/blank" checklist
2. **QEMU CLI Reference** - Document `-device virtio-gpu-device,xres=,yres=`
3. **VGA Compat Mode Warning** - Emphasize SET_SCANOUT requirement

---

## Status

- [x] Team file created
- [x] Spec verified against VirtIO 1.1
- [x] Tiny screen cause identified
- [x] Suitability for terminal confirmed

**Verdict:** Spec is accurate. Implementation needs to follow the initialization sequence properly.
