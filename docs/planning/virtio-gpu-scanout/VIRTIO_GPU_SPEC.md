# VirtIO GPU Protocol Specification

**Source:** [OASIS VirtIO 1.1 Specification, Section 5.7](https://docs.oasis-open.org/virtio/virtio/v1.1/cs01/virtio-v1.1-cs01.html#x1-3310007)  
**Document Purpose:** Spec-driven development reference for `levitate-virtio-gpu`  
**Created by:** TEAM_095

---

## Table of Contents

1. [Overview](#1-overview)
2. [Device Identification](#2-device-identification)
3. [Virtqueues](#3-virtqueues)
4. [Feature Bits](#4-feature-bits)
5. [Configuration Space](#5-configuration-space)
6. [Control Header](#6-control-header)
7. [Command Types](#7-command-types)
8. [Response Types](#8-response-types)
9. [Data Structures](#9-data-structures)
10. [Pixel Formats](#10-pixel-formats)
11. [Initialization Sequence](#11-initialization-sequence)
12. [Runtime Operations](#12-runtime-operations)
13. [Driver State Machine](#13-driver-state-machine)
14. [Rust Implementation Notes](#14-rust-implementation-notes)

---

## 1. Overview

The VirtIO GPU device provides a virtual graphics adapter operating in **2D mode** (this spec) or 3D mode (virgl, not covered here).

**Key Concepts:**
- **Resources:** Host-side objects (framebuffers, textures) identified by guest-generated IDs
- **Backing Store:** Guest memory pages attached to resources for DMA transfers
- **Scanouts:** Display outputs (up to 16) that present resources to the host display
- **Control Queue:** Commands for resource management, scanout configuration
- **Cursor Queue:** Fast-path for cursor updates

**Design Philosophy:**
> "The virtio-gpu is based around the concept of resources private to the host, the guest must DMA transfer into these resources."

---

## 2. Device Identification

```
Device ID: 16 (0x10)
PCI Class: DISPLAY_VGA (with VGA compat) or DISPLAY_OTHER
```

---

## 3. Virtqueues

| Index | Name     | Purpose |
|-------|----------|---------|
| 0     | controlq | All commands except cursor |
| 1     | cursorq  | UPDATE_CURSOR, MOVE_CURSOR only |

Both queues use the same request/response format with `virtio_gpu_ctrl_hdr`.

---

## 4. Feature Bits

| Bit | Name | Description |
|-----|------|-------------|
| 0   | VIRTIO_GPU_F_VIRGL | 3D mode support (not used in 2D) |
| 1   | VIRTIO_GPU_F_EDID | EDID data available via GET_EDID |

**For 2D mode:** Only `VIRTIO_GPU_F_EDID` is relevant.

---

## 5. Configuration Space

```c
#define VIRTIO_GPU_EVENT_DISPLAY (1 << 0)

struct virtio_gpu_config {
    le32 events_read;    // Pending events (read-only to driver)
    le32 events_clear;   // Write 1 to clear corresponding bit in events_read
    le32 num_scanouts;   // Max scanouts (1-16)
    le32 reserved;
};
```

### Rust Representation

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuConfig {
    pub events_read: u32,
    pub events_clear: u32,
    pub num_scanouts: u32,
    pub reserved: u32,
}

pub const VIRTIO_GPU_EVENT_DISPLAY: u32 = 1 << 0;
```

### Events

| Event | Value | Meaning |
|-------|-------|---------|
| VIRTIO_GPU_EVENT_DISPLAY | `1 << 0` | Display config changed; driver SHOULD send GET_DISPLAY_INFO |

---

## 6. Control Header

**All** requests and responses begin with this header:

```c
#define VIRTIO_GPU_FLAG_FENCE (1 << 0)

struct virtio_gpu_ctrl_hdr {
    le32 type;      // Command or response type
    le32 flags;     // VIRTIO_GPU_FLAG_FENCE for synchronous completion
    le64 fence_id;  // Copied to response if FLAG_FENCE set
    le32 ctx_id;    // 3D context (0 for 2D mode)
    le32 padding;
};
```

**Size:** 24 bytes

### Rust Representation

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtioGpuCtrlHdr {
    pub type_: u32,
    pub flags: u32,
    pub fence_id: u64,
    pub ctx_id: u32,
    pub padding: u32,
}

pub const VIRTIO_GPU_FLAG_FENCE: u32 = 1 << 0;
```

---

## 7. Command Types

### 2D Commands (0x01XX)

| Command | Value | Request Struct | Response |
|---------|-------|----------------|----------|
| GET_DISPLAY_INFO | 0x0100 | (header only) | virtio_gpu_resp_display_info |
| RESOURCE_CREATE_2D | 0x0101 | virtio_gpu_resource_create_2d | NODATA |
| RESOURCE_UNREF | 0x0102 | virtio_gpu_resource_unref | NODATA |
| SET_SCANOUT | 0x0103 | virtio_gpu_set_scanout | NODATA |
| RESOURCE_FLUSH | 0x0104 | virtio_gpu_resource_flush | NODATA |
| TRANSFER_TO_HOST_2D | 0x0105 | virtio_gpu_transfer_to_host_2d | NODATA |
| RESOURCE_ATTACH_BACKING | 0x0106 | virtio_gpu_resource_attach_backing + mem_entries | NODATA |
| RESOURCE_DETACH_BACKING | 0x0107 | virtio_gpu_resource_detach_backing | NODATA |
| GET_CAPSET_INFO | 0x0108 | virtio_gpu_get_capset_info | virtio_gpu_resp_capset_info |
| GET_CAPSET | 0x0109 | virtio_gpu_get_capset | virtio_gpu_resp_capset |
| GET_EDID | 0x010A | virtio_gpu_get_edid | virtio_gpu_resp_edid |

### Cursor Commands (0x03XX)

| Command | Value | Request Struct | Response |
|---------|-------|----------------|----------|
| UPDATE_CURSOR | 0x0300 | virtio_gpu_update_cursor | NODATA |
| MOVE_CURSOR | 0x0301 | virtio_gpu_update_cursor | NODATA |

### Rust Enum

```rust
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtioGpuCtrlType {
    // 2D Commands
    CmdGetDisplayInfo = 0x0100,
    CmdResourceCreate2d = 0x0101,
    CmdResourceUnref = 0x0102,
    CmdSetScanout = 0x0103,
    CmdResourceFlush = 0x0104,
    CmdTransferToHost2d = 0x0105,
    CmdResourceAttachBacking = 0x0106,
    CmdResourceDetachBacking = 0x0107,
    CmdGetCapsetInfo = 0x0108,
    CmdGetCapset = 0x0109,
    CmdGetEdid = 0x010A,

    // Cursor Commands
    CmdUpdateCursor = 0x0300,
    CmdMoveCursor = 0x0301,

    // Success Responses
    RespOkNodata = 0x1100,
    RespOkDisplayInfo = 0x1101,
    RespOkCapsetInfo = 0x1102,
    RespOkCapset = 0x1103,
    RespOkEdid = 0x1104,

    // Error Responses
    RespErrUnspec = 0x1200,
    RespErrOutOfMemory = 0x1201,
    RespErrInvalidScanoutId = 0x1202,
    RespErrInvalidResourceId = 0x1203,
    RespErrInvalidContextId = 0x1204,
    RespErrInvalidParameter = 0x1205,
}
```

---

## 8. Response Types

### Success Responses (0x11XX)

| Response | Value | Payload |
|----------|-------|---------|
| RESP_OK_NODATA | 0x1100 | (header only) |
| RESP_OK_DISPLAY_INFO | 0x1101 | virtio_gpu_resp_display_info |
| RESP_OK_CAPSET_INFO | 0x1102 | virtio_gpu_resp_capset_info |
| RESP_OK_CAPSET | 0x1103 | virtio_gpu_resp_capset |
| RESP_OK_EDID | 0x1104 | virtio_gpu_resp_edid |

### Error Responses (0x12XX)

| Response | Value | Meaning |
|----------|-------|---------|
| RESP_ERR_UNSPEC | 0x1200 | Unspecified error |
| RESP_ERR_OUT_OF_MEMORY | 0x1201 | Host out of memory |
| RESP_ERR_INVALID_SCANOUT_ID | 0x1202 | Scanout ID >= num_scanouts |
| RESP_ERR_INVALID_RESOURCE_ID | 0x1203 | Resource does not exist |
| RESP_ERR_INVALID_CONTEXT_ID | 0x1204 | 3D context invalid |
| RESP_ERR_INVALID_PARAMETER | 0x1205 | Generic parameter error |

---

## 9. Data Structures

### 9.1 Rectangle

```c
struct virtio_gpu_rect {
    le32 x;
    le32 y;
    le32 width;
    le32 height;
};
```

**Coordinate System:** (0,0) = top-left, +x = right, +y = down

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtioGpuRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
```

---

### 9.2 GET_DISPLAY_INFO Response

```c
#define VIRTIO_GPU_MAX_SCANOUTS 16

struct virtio_gpu_display_one {
    struct virtio_gpu_rect r;
    le32 enabled;
    le32 flags;
};

struct virtio_gpu_resp_display_info {
    struct virtio_gpu_ctrl_hdr hdr;
    struct virtio_gpu_display_one pmodes[VIRTIO_GPU_MAX_SCANOUTS];
};
```

**Size:** 24 + (16 × 24) = 408 bytes

```rust
pub const VIRTIO_GPU_MAX_SCANOUTS: usize = 16;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtioGpuDisplayOne {
    pub r: VirtioGpuRect,
    pub enabled: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuRespDisplayInfo {
    pub hdr: VirtioGpuCtrlHdr,
    pub pmodes: [VirtioGpuDisplayOne; VIRTIO_GPU_MAX_SCANOUTS],
}
```

---

### 9.3 RESOURCE_CREATE_2D

```c
struct virtio_gpu_resource_create_2d {
    struct virtio_gpu_ctrl_hdr hdr;
    le32 resource_id;   // Guest-generated, must be unique
    le32 format;        // virtio_gpu_formats enum
    le32 width;
    le32 height;
};
```

**Size:** 24 + 16 = 40 bytes

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuResourceCreate2d {
    pub hdr: VirtioGpuCtrlHdr,
    pub resource_id: u32,
    pub format: u32,
    pub width: u32,
    pub height: u32,
}
```

---

### 9.4 RESOURCE_UNREF

```c
struct virtio_gpu_resource_unref {
    struct virtio_gpu_ctrl_hdr hdr;
    le32 resource_id;
    le32 padding;
};
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuResourceUnref {
    pub hdr: VirtioGpuCtrlHdr,
    pub resource_id: u32,
    pub padding: u32,
}
```

---

### 9.5 SET_SCANOUT

```c
struct virtio_gpu_set_scanout {
    struct virtio_gpu_ctrl_hdr hdr;
    struct virtio_gpu_rect r;       // Rectangle within resource
    le32 scanout_id;                // 0..num_scanouts-1
    le32 resource_id;               // 0 = disable scanout
};
```

**Size:** 24 + 16 + 8 = 48 bytes

**Critical Requirements:**
- Scanout rectangle MUST be completely covered by the resource
- `resource_id = 0` disables the scanout
- Overlapping scanouts are allowed (mirroring)

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuSetScanout {
    pub hdr: VirtioGpuCtrlHdr,
    pub r: VirtioGpuRect,
    pub scanout_id: u32,
    pub resource_id: u32,
}
```

---

### 9.6 RESOURCE_FLUSH

```c
struct virtio_gpu_resource_flush {
    struct virtio_gpu_ctrl_hdr hdr;
    struct virtio_gpu_rect r;
    le32 resource_id;
    le32 padding;
};
```

**Size:** 24 + 16 + 8 = 48 bytes

**Behavior:** Flushes the specified rectangle of the resource to ALL scanouts that reference it.

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuResourceFlush {
    pub hdr: VirtioGpuCtrlHdr,
    pub r: VirtioGpuRect,
    pub resource_id: u32,
    pub padding: u32,
}
```

---

### 9.7 TRANSFER_TO_HOST_2D

```c
struct virtio_gpu_transfer_to_host_2d {
    struct virtio_gpu_ctrl_hdr hdr;
    struct virtio_gpu_rect r;       // Rectangle to transfer
    le64 offset;                    // Byte offset into guest backing store
    le32 resource_id;
    le32 padding;
};
```

**Size:** 24 + 16 + 8 + 8 = 56 bytes

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuTransferToHost2d {
    pub hdr: VirtioGpuCtrlHdr,
    pub r: VirtioGpuRect,
    pub offset: u64,
    pub resource_id: u32,
    pub padding: u32,
}
```

---

### 9.8 RESOURCE_ATTACH_BACKING

```c
struct virtio_gpu_resource_attach_backing {
    struct virtio_gpu_ctrl_hdr hdr;
    le32 resource_id;
    le32 nr_entries;        // Number of following mem_entry structs
};

struct virtio_gpu_mem_entry {
    le64 addr;              // Guest physical address
    le32 length;            // Length in bytes
    le32 padding;
};
```

**Layout in buffer:**
1. `virtio_gpu_resource_attach_backing` (32 bytes)
2. `virtio_gpu_mem_entry[nr_entries]` (16 bytes each)

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuResourceAttachBacking {
    pub hdr: VirtioGpuCtrlHdr,
    pub resource_id: u32,
    pub nr_entries: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuMemEntry {
    pub addr: u64,
    pub length: u32,
    pub padding: u32,
}
```

---

### 9.9 RESOURCE_DETACH_BACKING

```c
struct virtio_gpu_resource_detach_backing {
    struct virtio_gpu_ctrl_hdr hdr;
    le32 resource_id;
    le32 padding;
};
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuResourceDetachBacking {
    pub hdr: VirtioGpuCtrlHdr,
    pub resource_id: u32,
    pub padding: u32,
}
```

---

### 9.10 GET_EDID

```c
struct virtio_gpu_get_edid {
    struct virtio_gpu_ctrl_hdr hdr;
    le32 scanout;
    le32 padding;
};

struct virtio_gpu_resp_edid {
    struct virtio_gpu_ctrl_hdr hdr;
    le32 size;
    le32 padding;
    u8 edid[1024];
};
```

---

### 9.11 Cursor Structures

```c
struct virtio_gpu_cursor_pos {
    le32 scanout_id;
    le32 x;
    le32 y;
    le32 padding;
};

struct virtio_gpu_update_cursor {
    struct virtio_gpu_ctrl_hdr hdr;
    struct virtio_gpu_cursor_pos pos;
    le32 resource_id;       // 64x64 cursor resource
    le32 hot_x;             // Hotspot X
    le32 hot_y;             // Hotspot Y
    le32 padding;
};
```

**Cursor Requirements:**
- Cursor resource MUST be 64×64 pixels
- Resource must be fully transferred before UPDATE_CURSOR
- Use MOVE_CURSOR for position-only updates (faster)

---

## 10. Pixel Formats

```c
enum virtio_gpu_formats {
    VIRTIO_GPU_FORMAT_B8G8R8A8_UNORM  = 1,   // BGRA (most common)
    VIRTIO_GPU_FORMAT_B8G8R8X8_UNORM  = 2,   // BGRX (no alpha)
    VIRTIO_GPU_FORMAT_A8R8G8B8_UNORM  = 3,   // ARGB
    VIRTIO_GPU_FORMAT_X8R8G8B8_UNORM  = 4,   // XRGB

    VIRTIO_GPU_FORMAT_R8G8B8A8_UNORM  = 67,  // RGBA
    VIRTIO_GPU_FORMAT_X8B8G8R8_UNORM  = 68,  // XBGR

    VIRTIO_GPU_FORMAT_A8B8G8R8_UNORM  = 121, // ABGR
    VIRTIO_GPU_FORMAT_R8G8B8X8_UNORM  = 134, // RGBX
};
```

**Recommended Format:** `B8G8R8A8_UNORM` (1) — matches QEMU default

```rust
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtioGpuFormat {
    B8G8R8A8Unorm = 1,
    B8G8R8X8Unorm = 2,
    A8R8G8B8Unorm = 3,
    X8R8G8B8Unorm = 4,
    R8G8B8A8Unorm = 67,
    X8B8G8R8Unorm = 68,
    A8B8G8R8Unorm = 121,
    R8G8B8X8Unorm = 134,
}

impl VirtioGpuFormat {
    pub fn bytes_per_pixel(&self) -> usize {
        4 // All formats are 32-bit
    }
}
```

---

## 11. Initialization Sequence

### Spec-Mandated Sequence

```
1. Reset device
2. Read feature bits, negotiate
3. Read num_scanouts from config
4. Set DRIVER_OK status
5. GET_DISPLAY_INFO → find enabled scanouts, get preferred resolution
6. RESOURCE_CREATE_2D → create framebuffer resource
7. RESOURCE_ATTACH_BACKING → attach guest memory pages
8. SET_SCANOUT → link resource to scanout
9. [Ready for rendering]
```

### State Machine

```
┌─────────────────┐
│  Uninitialized  │
└────────┬────────┘
         │ VirtIO device init
         ▼
┌─────────────────┐
│  DeviceReady    │
└────────┬────────┘
         │ GET_DISPLAY_INFO
         ▼
┌─────────────────┐
│  DisplayQueried │
└────────┬────────┘
         │ RESOURCE_CREATE_2D
         ▼
┌─────────────────┐
│ ResourceCreated │
└────────┬────────┘
         │ RESOURCE_ATTACH_BACKING
         ▼
┌─────────────────┐
│ BackingAttached │
└────────┬────────┘
         │ SET_SCANOUT
         ▼
┌─────────────────┐
│   ScanoutSet    │◄─────────────────┐
└────────┬────────┘                  │
         │                           │
         ▼                           │
┌─────────────────┐                  │
│     Ready       │──────────────────┘
└────────┬────────┘   (reconfigure)
         │
         ▼
    [Rendering Loop]
```

---

## 12. Runtime Operations

### Render Loop (Per Frame)

```
1. Draw to guest framebuffer memory
2. TRANSFER_TO_HOST_2D (dirty rect)
3. RESOURCE_FLUSH (dirty rect)
```

### Pseudocode

```rust
fn flush_display(&mut self, dirty: VirtioGpuRect) -> Result<(), GpuError> {
    // Step 1: Transfer dirty region to host
    self.send_command(VirtioGpuTransferToHost2d {
        hdr: self.make_header(VirtioGpuCtrlType::CmdTransferToHost2d),
        r: dirty,
        offset: self.rect_to_offset(dirty),
        resource_id: self.framebuffer_resource_id,
        padding: 0,
    }).await?;

    // Step 2: Flush to display
    self.send_command(VirtioGpuResourceFlush {
        hdr: self.make_header(VirtioGpuCtrlType::CmdResourceFlush),
        r: dirty,
        resource_id: self.framebuffer_resource_id,
        padding: 0,
    }).await?;

    Ok(())
}
```

### Pageflip (Double Buffering)

```
1. Create two resources (front, back)
2. Render to back buffer
3. TRANSFER_TO_HOST_2D (back)
4. SET_SCANOUT (back) — switches which resource is displayed
5. RESOURCE_FLUSH (back)
6. Swap front/back pointers
```

---

## 13. Driver State Machine

### Rust Implementation

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtioGpuState {
    Uninitialized,
    DeviceReady,
    DisplayQueried,
    ResourceCreated,
    BackingAttached,
    ScanoutSet,
    Ready,
    Error(VirtioGpuError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtioGpuError {
    DeviceInitFailed,
    NoDisplayEnabled,
    ResourceCreateFailed,
    BackingAttachFailed,
    ScanoutSetFailed,
    CommandTimeout,
}
```

### Transition Table

| Current State | Command | Success → | Failure → |
|---------------|---------|-----------|-----------|
| Uninitialized | (device init) | DeviceReady | Error(DeviceInitFailed) |
| DeviceReady | GET_DISPLAY_INFO | DisplayQueried | Error(NoDisplayEnabled) |
| DisplayQueried | RESOURCE_CREATE_2D | ResourceCreated | Error(ResourceCreateFailed) |
| ResourceCreated | RESOURCE_ATTACH_BACKING | BackingAttached | Error(BackingAttachFailed) |
| BackingAttached | SET_SCANOUT | ScanoutSet | Error(ScanoutSetFailed) |
| ScanoutSet | (auto) | Ready | — |
| Ready | RESOURCE_FLUSH | Ready | Ready (retry) |

---

## 14. Rust Implementation Notes

### Memory Layout Requirements

All structs MUST be:
- `#[repr(C)]` — C-compatible layout
- Little-endian fields (`le32` = `u32` on LE arch, explicit conversion on BE)
- Properly aligned (4-byte alignment minimum)

### Zerocopy Compatibility

```rust
use zerocopy::{AsBytes, FromBytes, FromZeroes};

#[repr(C)]
#[derive(Debug, Clone, Copy, AsBytes, FromBytes, FromZeroes)]
pub struct VirtioGpuCtrlHdr { /* ... */ }
```

### Async Command Interface

```rust
pub trait VirtioGpuCommand: Sized + AsBytes {
    type Response: FromBytes;
    fn command_type(&self) -> VirtioGpuCtrlType;
}

impl VirtioGpuDriver {
    pub async fn send_command<C: VirtioGpuCommand>(
        &mut self,
        cmd: C,
    ) -> Result<C::Response, GpuError> {
        // 1. Allocate request/response buffers
        // 2. Write command to request buffer
        // 3. Submit to virtqueue
        // 4. Await completion (via Waker)
        // 5. Parse response
    }
}
```

### Resource ID Generation

```rust
pub struct ResourceIdAllocator {
    next_id: AtomicU32,
}

impl ResourceIdAllocator {
    pub fn alloc(&self) -> ResourceId {
        ResourceId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);
```

### RAII Resource Handle

```rust
pub struct GpuResource {
    id: ResourceId,
    driver: Arc<Mutex<VirtioGpuDriver>>,
}

impl Drop for GpuResource {
    fn drop(&mut self) {
        if let Ok(mut driver) = self.driver.lock() {
            let _ = driver.send_command_blocking(VirtioGpuResourceUnref {
                hdr: driver.make_header(VirtioGpuCtrlType::CmdResourceUnref),
                resource_id: self.id.0,
                padding: 0,
            });
        }
    }
}
```

---

## Appendix A: Quick Reference

### Struct Sizes

| Struct | Size (bytes) |
|--------|--------------|
| virtio_gpu_ctrl_hdr | 24 |
| virtio_gpu_rect | 16 |
| virtio_gpu_resource_create_2d | 40 |
| virtio_gpu_set_scanout | 48 |
| virtio_gpu_resource_flush | 48 |
| virtio_gpu_transfer_to_host_2d | 56 |
| virtio_gpu_resource_attach_backing | 32 |
| virtio_gpu_mem_entry | 16 |
| virtio_gpu_resp_display_info | 408 |

### Common Pitfalls

1. **Scanout rectangle exceeds resource bounds** → RESP_ERR_INVALID_PARAMETER
2. **Flush before transfer** → Displays stale data
3. **Missing backing attachment** → No DMA possible
4. **Wrong pixel format byte order** → Swapped colors
5. **Blocking on command without response buffer** → Deadlock

---

## Appendix B: QEMU-Specific Notes

- Default format: `B8G8R8A8_UNORM`
- Default resolution: 1280×800 (configurable)
- VGA compatibility: Enabled by default, disabled after first SET_SCANOUT
- 3D mode: Requires `-device virtio-gpu-gl-pci` flag

---

## Revision History

| Date | Team | Changes |
|------|------|---------|
| 2026-01-05 | TEAM_095 | Initial spec extraction from VirtIO 1.1 |
