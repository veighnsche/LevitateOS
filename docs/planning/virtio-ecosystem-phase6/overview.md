# Phase 6: VirtIO Ecosystem Expansion — Overview

## Feature Summary

**Phase 6** expands LevitateOS hardware support through three VirtIO subsystems:

1. **VirtIO Net** — Network packet transmission/reception
2. **GPU Refinement** — Text rendering / terminal emulation on framebuffer
3. **9P Filesystem** — Mount host directories via `virtio-9p`

## Current State Analysis

### Existing Infrastructure

| Component | Status | Location |
|-----------|--------|----------|
| VirtIO HAL | ✅ Complete | `kernel/src/virtio.rs` |
| MMIO Transport | ✅ Complete | Uses `virtio-drivers` v0.12 |
| VirtIO Block | ✅ Complete | `kernel/src/block.rs` |
| VirtIO GPU | ✅ Basic | `kernel/src/gpu.rs` (framebuffer only) |
| VirtIO Input | ✅ Complete | `kernel/src/input.rs` |
| Device Discovery | ✅ Complete | Scans `0x0a000000` - `0x0a100000` |

### QEMU Configuration

Current `run.sh` already includes:
```bash
-device virtio-net-device,netdev=net0
-netdev user,id=net0
```

VirtIO-Net device is **already being passed** but not handled by the kernel.

### Dependencies

```toml
# kernel/Cargo.toml
virtio-drivers = "0.12"        # Has VirtIONet, NO 9P driver
embedded-graphics = "0.8.1"    # Has text rendering support
```

---

## Task Breakdown

### Task 6.1: VirtIO Net (Priority: High)

**Objective**: Basic network packet TX/RX capability.

**Available Support**:
- `virtio_drivers::device::net::VirtIONet<H, T, QUEUE_SIZE>` — Full driver
- `virtio_drivers::device::net::VirtIONetRaw<H, T, QUEUE_SIZE>` — Raw API

**Implementation Pattern** (follows `block.rs`):
```rust
// kernel/src/net.rs (NEW)
use virtio_drivers::device::net::VirtIONet;
use crate::virtio::{StaticMmioTransport, VirtioHal};

static NET_DEVICE: Spinlock<Option<VirtIONet<VirtioHal, StaticMmioTransport, 16>>> =
    Spinlock::new(None);

pub fn init(transport: StaticMmioTransport) { ... }
pub fn mac_address() -> [u8; 6] { ... }
pub fn send(packet: &[u8]) -> Result<(), Error> { ... }
pub fn receive() -> Option<Vec<u8>> { ... }
```

**Files to Create/Modify**:
- `kernel/src/net.rs` — **NEW**: Network driver wrapper
- `kernel/src/virtio.rs` — Add `DeviceType::Network` match arm
- `kernel/src/main.rs` — Add `mod net;` declaration

**Success Criteria**:
- [ ] Detect `virtio-net` device during MMIO scan
- [ ] Read MAC address from device
- [ ] Send a raw Ethernet frame (even if it goes nowhere)
- [ ] Receive packets (polling mode initially)

---

### Task 6.2: GPU Text Rendering (Priority: Medium)

**Objective**: Render text to the GPU framebuffer for terminal emulation.

**Available Support**:
- `embedded-graphics` already in dependencies
- `embedded-graphics::mono_font` — Built-in monospace bitmap fonts
- `embedded-graphics::text::Text` — Text rendering primitive

**Current GPU State** (`kernel/src/gpu.rs`):
- `GpuState` holds framebuffer pointer, dimensions
- `Display` implements `DrawTarget` trait
- Can draw rectangles via `embedded-graphics`

**Implementation Pattern**:
```rust
// kernel/src/terminal.rs (NEW)
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    text::Text,
};

pub struct Terminal {
    cursor_x: u32,
    cursor_y: u32,
    fg_color: Rgb888,
    bg_color: Rgb888,
}

impl Terminal {
    pub fn write_char(&mut self, display: &mut Display, c: char) { ... }
    pub fn write_str(&mut self, display: &mut Display, s: &str) { ... }
    pub fn newline(&mut self) { ... }
    pub fn scroll(&mut self, display: &mut Display) { ... }
}
```

**Font Options** (built into `embedded-graphics`):
- `FONT_6X10` — Compact, good for terminals
- `FONT_8X13` — More readable
- `FONT_10X20` — Large, high-DPI friendly

**Files to Create/Modify**:
- `kernel/src/terminal.rs` — **NEW**: Terminal emulator
- `kernel/src/gpu.rs` — Add text rendering helpers (optional)
- `kernel/src/main.rs` — Integrate terminal output

**Success Criteria**:
- [ ] Render "Hello, LevitateOS!" on GPU framebuffer
- [ ] Implement basic character output (`write_char`)
- [ ] Handle newlines and cursor positioning
- [ ] (Optional) Basic scrolling when buffer full

---

### Task 6.3: VirtIO 9P Filesystem (Priority: Low — Complex)

**Objective**: Mount host directories via Plan 9 protocol.

**Available Support**:
- `virtio-drivers` recognizes `DeviceType::_9P` (device type 9)
- **NO built-in 9P driver** in `virtio-drivers` crate
- Must implement 9P2000.L protocol manually or find/create a crate

**9P Protocol Overview**:
- Plan 9 filesystem protocol, extended as 9P2000.L for Linux
- Message-based: Tversion, Rversion, Tattach, Rattach, Twalk, Rwalk, etc.
- Each message has a 4-byte size, 1-byte type, 2-byte tag

**QEMU Configuration** (not yet in `run.sh`):
```bash
-fsdev local,security_model=mapped,id=fsdev0,path=/tmp/share \
-device virtio-9p-device,fsdev=fsdev0,mount_tag=hostshare
```

**Implementation Strategy**:

**Option A: Minimal Read-Only 9P Client**
- Implement subset: `Tversion`, `Tattach`, `Twalk`, `Topen`, `Tread`, `Tclunk`
- ~500-1000 lines of protocol handling
- Sufficient for loading files from host

**Option B: Port Existing Rust 9P Crate**
- `rs9p` — Tokio-based, requires adaptation for `no_std`
- `rust-9p` — Also async/Tokio-based
- Would need significant modification

**Files to Create**:
- `kernel/src/p9/mod.rs` — **NEW**: 9P protocol module
- `kernel/src/p9/protocol.rs` — Message types and parsing
- `kernel/src/p9/client.rs` — 9P client state machine
- `kernel/src/virtio.rs` — Add `DeviceType::_9P` match arm

**Success Criteria**:
- [ ] Detect `virtio-9p` device during MMIO scan
- [ ] Complete 9P version handshake
- [ ] Attach to filesystem root
- [ ] Walk to a file path
- [ ] Read file contents from host

---

## Recommended Implementation Order

```
1. VirtIO Net (Task 6.1)
   └── Follows existing driver pattern
   └── Crate has full support
   └── ~1-2 hours implementation

2. GPU Text Rendering (Task 6.2)
   └── Uses existing embedded-graphics
   └── Foundation for future terminal
   └── ~2-3 hours implementation

3. 9P Filesystem (Task 6.3)
   └── Most complex (no crate support)
   └── Requires protocol implementation
   └── ~1-2 days for minimal read-only
```

---

## External References

### VirtIO Specifications
- VirtIO Net: https://docs.oasis-open.org/virtio/virtio/v1.1/cs01/virtio-v1.1-cs01.html#x1-2000003
- VirtIO 9P: https://docs.oasis-open.org/virtio/virtio/v1.1/cs01/virtio-v1.1-cs01.html#x1-3900009

### 9P Protocol
- 9P2000.L Specification: http://ericvh.github.io/9p-rfc/
- Linux kernel 9P: https://www.kernel.org/doc/Documentation/filesystems/9p.txt

### Rust Resources
- virtio-drivers source: https://github.com/rcore-os/virtio-drivers
- embedded-graphics fonts: https://docs.rs/embedded-graphics/latest/embedded_graphics/mono_font/index.html

---

## Risk Assessment

| Task | Risk | Mitigation |
|------|------|------------|
| VirtIO Net | Low | Full crate support |
| GPU Text | Low | embedded-graphics well-documented |
| 9P | High | No existing no_std implementation; manual protocol work |

---

## Questions for USER

1. **VirtIO Net**: Should we implement a basic IP stack or just raw Ethernet frames initially?
2. **GPU Text**: Preferred font size? (6x10 compact vs 10x20 readable)
3. **9P**: Should we prioritize this or defer to a later phase given complexity?
