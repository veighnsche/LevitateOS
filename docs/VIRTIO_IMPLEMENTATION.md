# VirtIO Implementation Guide

**TEAM_109** | Created: 2026-01-05

This document captures knowledge for implementing VirtIO drivers in LevitateOS.

---

## Current State

| Crate | Status | Description |
|-------|--------|-------------|
| `levitate-virtio` | Partial | Transport layer, VirtQueue (has issues) |
| `levitate-drivers-gpu` | Broken | Custom GPU driver, times out |
| `levitate-gpu` | **FALSE POSITIVE** | Tests pass but display doesn't work |

**⚠️ CRITICAL: Neither GPU crate actually works!**

The kernel uses `levitate-gpu` but this is a **false positive**:
- Tests pass because they only check driver init, not actual display
- The QEMU window shows nothing / "Display output is not active"
- `levitate-drivers-gpu` was created specifically because `levitate-gpu` never worked

**DO NOT trust passing tests as proof the GPU works.**

---

## VirtIO 1.1 Spec Requirements

### Memory Layout (Split Virtqueue)

Per VirtIO 1.1 Section 2.6:

```
+------------------+  <- Descriptor Table (16-byte aligned)
| Descriptor[0]    |
| Descriptor[1]    |
| ...              |
+------------------+  <- Available Ring (2-byte aligned)
| flags            |  2 bytes
| idx              |  2 bytes
| ring[0..SIZE]    |  2 * SIZE bytes
| used_event       |  2 bytes (even if EVENT_IDX not used)
+------------------+  <- Used Ring (4-BYTE ALIGNED!)
| flags            |  2 bytes
| idx              |  2 bytes
| ring[0..SIZE]    |  8 * SIZE bytes (id: u32, len: u32)
| avail_event      |  2 bytes
+------------------+
```

### Critical Alignment Requirements

| Component | Alignment | Notes |
|-----------|-----------|-------|
| Descriptor Table | 16 bytes | First field, naturally aligned |
| Available Ring | 2 bytes | Follows descriptors |
| **Used Ring** | **4 bytes** | MUST add padding if needed! |

---

## What Works in virtio-drivers

The external `virtio-drivers` crate works. Key architectural choices:

### 1. Separate DMA Regions

```rust
// virtio-drivers allocates SEPARATE regions:
let driver_to_device_dma = Dma::new(pages(desc + avail), ...);
let device_to_driver_dma = Dma::new(pages(used), ...);
```

### 2. Pointer-Based Access

```rust
struct VirtQueue<H, SIZE> {
    desc: NonNull<[Descriptor]>,      // Pointer to DMA memory
    avail: NonNull<AvailRing>,        // Pointer to DMA memory
    used: NonNull<UsedRing>,          // Pointer to DMA memory
    desc_shadow: [Descriptor; SIZE],  // LOCAL shadow copy
}
```

### 3. Shadow Descriptors

virtio-drivers writes to `desc_shadow` first, then copies to actual descriptors:

```rust
fn write_desc(&mut self, index: u16) {
    unsafe {
        (*self.desc.as_ptr())[index] = self.desc_shadow[index].clone();
    }
}
```

### 4. AtomicU16 for Ring Indices

```rust
struct AvailRing<SIZE> {
    flags: AtomicU16,
    idx: AtomicU16,
    ring: [u16; SIZE],
    used_event: AtomicU16,
}
```

---

## What's Different in Our Implementation

### 1. Single Embedded Struct

```rust
// Our VirtQueue embeds everything:
struct VirtQueue<SIZE> {
    descriptors: [Descriptor; SIZE],
    avail_flags: u16,
    avail_idx: u16,
    avail_ring: [u16; SIZE],
    // ...
}
```

### 2. Direct Field Access

We write directly to struct fields instead of through pointers.

### 3. Volatile Instead of Atomics

We use `write_volatile` instead of `AtomicU16`.

---

## Fixes Applied (Still Not Working)

TEAM_109 applied these fixes to `levitate-virtio`:

| Fix | Location | Status |
|-----|----------|--------|
| Event fields (used_event, avail_event) | queue.rs | ✅ Applied |
| Padding for 4-byte used ring alignment | queue.rs | ✅ Applied |
| Volatile writes for descriptors | queue.rs | ✅ Applied |
| Volatile write for avail_ring | queue.rs | ✅ Applied |
| ARM DSB barrier | queue.rs | ✅ Applied |
| DMA buffers for commands | device.rs | ✅ Applied |

**Result:** Device is notified but never responds. All tests fail with timeout.

---

## Debug Findings

When testing the custom driver:

```
[GPU-DBG] Queue setup:
[GPU-DBG]   desc:  v=0xffff8000400c7000 p=0x400c7000
[GPU-DBG]   avail: v=0xffff8000400c7100 p=0x400c7100
[GPU-DBG]   used:  v=0xffff8000400c7128 p=0x400c7128  <- 4-byte aligned ✓
[GPU-DBG] VirtioGpu created, calling init()
[GPU-DBG] cmd: v=0xffff8000400c8000 p=0x400c8000 len=24
[GPU-DBG] resp: v=0xffff8000400c9000 p=0x400c9000 len=408
[GPU-DBG] Buffer added, notifying device...
[GPU-DBG] Wait done, timeout remaining: 0  <- Device never responds!
```

- Addresses look correct (in physical RAM range)
- Alignment is correct
- Device is notified
- But device never writes to used ring

---

## Recommended Next Steps

### Option A: Refactor VirtQueue Architecture (HIGH effort)

Rewrite `levitate-virtio/src/queue.rs` to match virtio-drivers:
- Separate DMA regions for desc+avail and used
- Pointer-based access instead of embedded data
- Shadow descriptors
- AtomicU16 for ring indices

### Option B: Port virtio-drivers' Queue (MEDIUM effort)

Copy and adapt `virtio-drivers/src/queue.rs`:
- Keep our transport layer
- Keep our GPU protocol code
- Replace only the VirtQueue implementation

### Option C: Use virtio-drivers' Queue Directly (LOW effort)

Make `levitate-drivers-gpu` depend on `virtio-drivers` for VirtQueue:
- Use `virtio_drivers::queue::VirtQueue`
- Keep our transport and GPU driver code

---

## Reference Materials

- **VirtIO 1.1 Spec:** https://docs.oasis-open.org/virtio/virtio/v1.1/
- **virtio-drivers crate:** `~/.cargo/registry/src/.../virtio-drivers-0.12.0/`
- **Tock OS VirtIO:** `.external-kernels/tock/chips/virtio/`
- **Team Files:** `.teams/TEAM_108_*.md`, `.teams/TEAM_109_*.md`

---

## Verification Commands

```bash
# Run behavior tests (uses working levitate-gpu)
cargo xtask test

# Build only (to check compilation)
cargo build --release -p levitate-kernel

# Check custom driver compiles
cargo build -p levitate-drivers-gpu
```

---

## Code Locations

| Component | Path |
|-----------|------|
| Working GPU driver | `levitate-gpu/src/gpu.rs` |
| Custom GPU driver | `levitate-drivers-gpu/src/device.rs` |
| VirtQueue (our impl) | `levitate-virtio/src/queue.rs` |
| Transport | `levitate-virtio/src/transport.rs` |
| HAL impl | `levitate-virtio/src/hal_impl.rs` |
| Kernel GPU interface | `kernel/src/gpu.rs` |
