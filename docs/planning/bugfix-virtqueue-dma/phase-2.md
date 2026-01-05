# Phase 2: Root Cause Analysis

**Bug**: VirtQueue DMA Memory Allocation
**TEAM_105**: Following /make-a-bugfix-plan workflow

---

## 2.1 Hypotheses List

### H1: Missing Descriptor Alignment ✓ CONFIRMED
- **Theory**: VirtIO spec requires 16-byte alignment for descriptor table
- **Evidence**: 
  - VirtIO 1.1 spec section 2.6 requires 16-byte alignment
  - virtio-drivers uses `#[repr(C, align(16))]`
  - Tock OS uses `#[repr(C, align(16))]` and `DESCRIPTOR_ALIGNMENT = 16`
  - Our Descriptor uses only `#[repr(C)]` (8-byte default on 64-bit)
- **Confidence**: HIGH

### H2: Non-DMA Memory Allocation ✓ CONFIRMED
- **Theory**: Queue memory must be allocated via HAL's dma_alloc for DMA-safe memory
- **Evidence**:
  - virtio-drivers uses `Dma::new()` for queue allocation
  - Our code uses `Box::new()` which uses regular heap
  - Regular heap may not be DMA-coherent or have stable physical addresses
- **Confidence**: HIGH

### H3: Physical Address Translation Issue ✓ RELATED
- **Theory**: Virtual addresses passed to device instead of physical
- **Evidence**:
  - `addresses()` method returns virtual addresses
  - Device code does translate via `H::virt_to_phys()`
  - This is handled correctly, but depends on H1/H2 being fixed first
- **Confidence**: MEDIUM (secondary issue)

---

## 2.2 Key Code Areas

### levitate-virtio/src/queue.rs

**Problem 1**: Descriptor struct (lines 38-55)
```rust
// Current - WRONG
#[repr(C)]  // Only 8-byte alignment
pub struct Descriptor { ... }

// Should be
#[repr(C, align(16))]  // 16-byte alignment per VirtIO spec
pub struct Descriptor { ... }
```

**Problem 2**: VirtQueue struct (lines 65-95)
```rust
// Current - WRONG
pub struct VirtQueue<const SIZE: usize> {
    descriptors: [Descriptor; SIZE],  // Not DMA-safe
    ...
}

// Should be allocated via HAL dma_alloc, not embedded in struct
```

### levitate-drivers-gpu/src/device.rs

**Problem 3**: Queue allocation (lines ~130)
```rust
// Current - WRONG
let mut control_queue = Box::new(VirtQueue::new());

// Should use HAL allocation
let (queue_paddr, queue_ptr) = H::dma_alloc(pages, BufferDirection::Both);
```

---

## 2.3 Investigation Strategy

Already complete - TEAM_104 did thorough investigation.

**Execution Path**:
1. `VirtioGpu::new()` creates VirtQueue via `Box::new()`
2. Gets addresses via `control_queue.addresses()` (returns virtual)
3. Translates to physical via `H::virt_to_phys()`
4. Sets queue addresses in MMIO transport
5. Sends GET_DISPLAY_INFO command
6. **Device writes to used ring at physical address**
7. **BUT: Memory not properly aligned OR not DMA-accessible**
8. Driver polls `has_used()` forever → timeout

---

## 2.4 Root Cause Summary

### Primary Root Cause
**Two-part failure in VirtQueue memory setup:**

1. **Alignment Violation**: `Descriptor` struct lacks `#[repr(C, align(16))]`
   - VirtIO 1.1 spec section 2.6 requires 16-byte alignment
   - Rust's default struct alignment is 8 bytes on 64-bit
   - Device may silently fail or corrupt data on misaligned access

2. **Non-DMA Allocation**: Queue allocated via `Box::new()` not `dma_alloc()`
   - Box uses general heap allocator
   - Memory may not be DMA-coherent
   - Physical address may change (unlikely but possible)

### Causal Chain
```
Box::new(VirtQueue) 
  → Memory at arbitrary heap location (not 16-byte aligned)
  → Physical address passed to device
  → Device attempts DMA write to used ring
  → Either: alignment fault OR cache coherency issue OR address mismatch
  → used_idx never updates from driver's perspective
  → Timeout
```

---

## Phase 2 Complete

Root cause fully documented. Proceed to Phase 3 for fix design.
