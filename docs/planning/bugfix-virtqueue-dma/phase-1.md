# Phase 1: Understanding and Scoping

**Bug**: VirtQueue DMA Memory Allocation
**TEAM_105**: Following /make-a-bugfix-plan workflow

---

## 1.1 Bug Summary

**Short description**: The `levitate-virtio` VirtQueue implementation times out on GPU commands because the queue memory lacks proper alignment and is not allocated through DMA-safe HAL functions.

**Severity**: HIGH - Blocks GPU initialization, no display output
**Impact**: User-facing - kernel boots but cannot display anything

---

## 1.2 Reproduction Status

**Reproducible**: YES

**Steps**:
1. Build kernel with `levitate-virtio-gpu` driver enabled
2. Run `cargo xtask test` or boot in QEMU
3. Observe timeout on GPU GET_DISPLAY_INFO command

**Expected**: GPU initializes successfully, terminal displays
**Actual**: GPU init hangs, timeout after ~1000ms polling used ring

---

## 1.3 Context

### Affected Code Areas
- `levitate-virtio/src/queue.rs` - VirtQueue struct and Descriptor
- `levitate-drivers-gpu/src/device.rs` - VirtioGpu device initialization

### Related Investigation
- TEAM_104 investigation: `.teams/TEAM_104_investigate_virtqueue_dma_bug.md`
- Breadcrumbs placed at `levitate-virtio/src/queue.rs:41-43, 69-73`

### Root Cause (from TEAM_104)
1. **Missing alignment**: Descriptor struct lacks 16-byte alignment
2. **Non-DMA allocation**: Queue uses `Box::new()` instead of HAL's `dma_alloc()`

---

## 1.4 Constraints

- **Backwards compatibility**: Must maintain VirtQueue API
- **No external dependencies**: Must work with existing HAL
- **Testing**: Must pass all 22 regression tests

---

## 1.5 Open Questions

None - root cause confirmed by TEAM_104 with high confidence.

---

## 1.6 Standards and Specifications

### VirtIO 1.1 Specification (OASIS)

**Source**: https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html

**Section 2.6 - Split Virtqueues**:
> "The driver MUST ensure that the physical address of the first byte of each virtqueue part is a multiple of the specified alignment value"

**Alignment Requirements Table** (Section 2.6):

| Virtqueue Part     | Alignment | Size |
|--------------------|-----------|------|
| Descriptor Table   | 16 bytes  | 16 × Queue Size |
| Available Ring     | 2 bytes   | 6 + 2 × Queue Size |
| Used Ring          | 4 bytes   | 6 + 8 × Queue Size |

### Reference Implementations

#### 1. virtio-drivers crate (Rust)
**File**: `~/.cargo/registry/src/.../virtio-drivers-0.12.0/src/queue.rs`
```rust
#[repr(C, align(16))]
struct Descriptor { ... }  // Line 709

fn allocate_legacy(queue_size: u16) -> Result<Self> {
    let dma = Dma::new(size / PAGE_SIZE, BufferDirection::Both)?;  // Lines 588-592
}
```

#### 2. Tock OS (Rust)
**File**: `.external-kernels/tock/chips/virtio/src/queues/split_queue.rs`
```rust
pub const DESCRIPTOR_ALIGNMENT: usize = 16;      // Line 30
pub const AVAILABLE_RING_ALIGNMENT: usize = 2;   // Line 31
pub const USED_RING_ALIGNMENT: usize = 4;        // Line 32

#[repr(C, align(16))]
pub struct VirtqueueDescriptors<...> { ... }     // Line 140
```

---

## Phase 1 Complete

All context gathered. Proceed to Phase 2 for root cause analysis documentation.
