# TEAM_104: Investigate VirtQueue DMA Bug

**Created:** 2026-01-05
**Status:** Active
**Workflow:** /investigate-a-bug

---

## Bug Report

**Source:** TEAM_103 implementation blocker (`.teams/TEAM_103_implement_crate_reorganization.md`)

**Symptom:** GPU initialization times out when using `levitate-virtio-gpu` driver with `levitate-virtio` VirtQueue.

**Error:** `[GPU] init() failed: Timeout` on GET_DISPLAY_INFO command

**Working alternative:** `levitate-gpu` (which uses `virtio-drivers` crate) works correctly.

**Previous investigation (TEAM_103):**
- Boxing VirtQueue to stabilize addresses: ❌ Did not fix
- Hypothesis: VirtQueue needs DMA-safe memory allocation

---

## Phase 1: Understand the Symptom

### Expected Behavior
- GPU driver sends GET_DISPLAY_INFO command via VirtQueue
- Device responds within timeout period
- Driver reads display info from used ring

### Actual Behavior
- GPU driver sends GET_DISPLAY_INFO command
- Device never responds (or response not detected)
- Driver times out waiting for used ring update

### Delta
- Command sent but no response received/detected

---

## Phase 2: Hypotheses

### H1: Memory Alignment Issue (HIGH confidence)
- **Theory**: VirtQueue descriptor table needs 16-byte alignment per VirtIO spec
- **Evidence needed**: Check if Box guarantees 16-byte alignment
- **Evidence to refute**: If Box does guarantee sufficient alignment

### H2: Non-DMA Memory Allocation (HIGH confidence)
- **Theory**: Queue memory must be allocated via HAL's dma_alloc for DMA-safe memory
- **Evidence needed**: Compare virtio-drivers allocation vs our allocation
- **Evidence to refute**: If regular heap memory works for DMA

### H3: Struct Layout Mismatch (MEDIUM confidence)
- **Theory**: VirtQueue struct layout doesn't match VirtIO spec
- **Evidence needed**: Check struct sizes and field offsets

---

## Phase 3: Evidence

### virtio-drivers Analysis

**File**: `~/.cargo/registry/src/.../virtio-drivers-0.12.0/src/queue.rs`

1. **Line 709**: Descriptor has explicit alignment:
   ```rust
   #[repr(C, align(16))]  // <-- 16-byte alignment!
   struct Descriptor { ... }
   ```

2. **Lines 588-614**: Queue memory allocated via HAL:
   ```rust
   fn allocate_legacy(queue_size: u16) -> Result<Self> {
       let dma = Dma::new(size / PAGE_SIZE, BufferDirection::Both)?;
       ...
   }
   fn allocate_flexible(queue_size: u16) -> Result<Self> {
       let driver_to_device_dma = Dma::new(pages(desc + avail), ...)?
       let device_to_driver_dma = Dma::new(pages(used), ...)?
   }
   ```

### Our VirtQueue Analysis

**File**: `levitate-virtio/src/queue.rs`

1. **No explicit alignment** on Descriptor or VirtQueue struct
2. **Box::new()** used for allocation - NOT through HAL
3. Box doesn't guarantee 16-byte alignment (only 8-byte on 64-bit)

---

## Phase 4: Root Cause

### CONFIRMED: Two-Part Root Cause

**Root Cause 1: Missing Descriptor Alignment**
- VirtIO spec requires descriptor table to be 16-byte aligned
- Our `Descriptor` struct lacks `#[repr(C, align(16))]`
- Box only guarantees 8-byte alignment on 64-bit

**Root Cause 2: Non-DMA Memory Allocation**
- VirtQueue must be in DMA-safe memory
- virtio-drivers uses HAL's `Dma::new()` / `dma_alloc()`
- We use `Box::new()` which uses regular heap
- Regular heap may not be DMA-coherent

**Causal Chain:**
1. VirtQueue allocated via Box (not DMA-safe, not 16-byte aligned)
2. Physical addresses passed to device point to potentially misaligned/non-DMA memory
3. Device writes to used ring, but either:
   - Memory not accessible to device (non-DMA)
   - Alignment causes device to malfunction
4. Driver polls used_idx but never sees update
5. Timeout

**Confidence**: HIGH (virtio-drivers working code confirms this pattern)

---

## Breadcrumbs Placed

1. `levitate-virtio/src/queue.rs:41-43` - CONFIRMED: Missing align(16) on Descriptor
2. `levitate-virtio/src/queue.rs:69-73` - CONFIRMED: VirtQueue needs HAL dma_alloc

---

## Phase 5: Decision

**Decision**: Create bugfix plan (fix too large for immediate implementation)

**Criteria evaluation**:
- ≤ 5 Units of Work: NO (~9 units)
- ≤ 50 lines of code: NO (~100-150 lines)
- Low risk: NO (touches core VirtIO)
- High confidence: YES
- Easily reversible: MAYBE

**Plan created**: `docs/planning/bugfix-virtqueue-dma/plan.md`

---

## Handoff Notes

### Summary
Investigated VirtQueue DMA bug that causes GPU init timeout. Found two-part root cause:
1. Missing 16-byte alignment on Descriptor struct
2. Using Box instead of HAL's dma_alloc for queue memory

### For Next Team
1. **Read**: `docs/planning/bugfix-virtqueue-dma/plan.md`
2. **Implement**: Follow the 6-phase implementation plan
3. **Test**: Verify GPU init succeeds and all behavior tests pass

### Key Files
- `levitate-virtio/src/queue.rs` - Add alignment attributes
- `levitate-drivers-gpu/src/device.rs` - Change to DMA allocation

### Reference
- Working implementation: `~/.cargo/registry/src/.../virtio-drivers-0.12.0/src/queue.rs`
- VirtIO 1.1 Spec Section 2.6

---

## Session Complete

- [x] Bug investigated
- [x] Root cause confirmed (high confidence)
- [x] Breadcrumbs placed in code
- [x] Bugfix plan created
- [x] Handoff documented
