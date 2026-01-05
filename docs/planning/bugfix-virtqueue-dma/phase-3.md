# Phase 3: Fix Design and Validation Plan

**Bug**: VirtQueue DMA Memory Allocation
**TEAM_105**: Following /make-a-bugfix-plan workflow

---

## 3.1 Root Cause Summary

**What is wrong**: VirtQueue memory lacks 16-byte alignment and is not DMA-safe.

**Where it lives**:
- `levitate-virtio/src/queue.rs:38-55` - Descriptor struct
- `levitate-virtio/src/queue.rs:65-95` - VirtQueue struct
- `levitate-drivers-gpu/src/device.rs:~130` - Queue allocation

---

## 3.2 Fix Strategy

### Option A: Minimal Fix (RECOMMENDED)

Add alignment attribute to structs and change allocation method.

**Pros**:
- Minimal code changes
- Low risk
- Matches reference implementations

**Cons**:
- Requires restructuring VirtioGpu to manage raw pointer

### Option B: Full Refactor

Match virtio-drivers architecture with separate DMA regions for each queue part.

**Pros**:
- More flexible
- Better separation of concerns

**Cons**:
- Much more code change
- Higher risk
- Not necessary for our use case

**Decision**: **Option A** - Minimal fix is sufficient and lower risk.

---

## 3.3 Reversal Strategy

**How to revert**:
1. Remove `align(16)` attributes from Descriptor/VirtQueue
2. Revert VirtioGpu to use `Box::new(VirtQueue::new())`
3. Remove Drop implementation for DMA deallocation

**Signals to revert**:
- GPU still times out after fix
- Memory corruption or panics during queue operations
- Regression in other VirtIO devices

**Steps to undo**:
```bash
git revert <commit-hash>
```

---

## 3.4 Test Strategy

### Existing Tests (must pass)
- All 22 regression tests via `cargo xtask test`

### New Tests to Add
1. **Alignment assertion** in VirtQueue::new():
   ```rust
   debug_assert!(core::mem::align_of::<Descriptor>() >= 16);
   ```

2. **Integration test**: GPU init succeeds without timeout

### Edge Cases
- Queue with size 1
- Queue with max size (256)
- Multiple commands in sequence

---

## 3.5 Impact Analysis

### API Changes
- **VirtQueue::new()** - May need HAL parameter or closure for allocation
- **VirtioGpu struct** - Changes from `Box<VirtQueue>` to raw pointer management

### Downstream Impact
- `levitate-drivers-gpu` - Must update allocation code
- `kernel` - No changes needed (uses same API)

### Performance
- No negative impact (DMA memory same speed as heap)

### Memory
- Slight increase due to page-aligned allocation
- Acceptable tradeoff for correctness

---

## 3.6 Detailed Fix Design

### Fix 1: Add Alignment to Descriptor

**File**: `levitate-virtio/src/queue.rs`

```rust
// Change from:
#[repr(C)]
pub struct Descriptor { ... }

// To:
#[repr(C, align(16))]
pub struct Descriptor { ... }
```

**Rationale**: VirtIO 1.1 spec section 2.6 requires 16-byte alignment.

### Fix 2: Add Alignment to VirtQueue

**File**: `levitate-virtio/src/queue.rs`

```rust
// Change from:
pub struct VirtQueue<const SIZE: usize> { ... }

// To:
#[repr(C, align(16))]
pub struct VirtQueue<const SIZE: usize> { ... }
```

**Rationale**: Ensures descriptor table at start of struct is 16-byte aligned.

### Fix 3: DMA Allocation in VirtioGpu

**File**: `levitate-drivers-gpu/src/device.rs`

```rust
// Change from:
let mut control_queue = Box::new(VirtQueue::new());

// To:
let queue_size = core::mem::size_of::<VirtQueue<QUEUE_SIZE>>();
let queue_pages = (queue_size + 4095) / 4096;
let (queue_paddr, queue_ptr) = H::dma_alloc(queue_pages, BufferDirection::Both);
let control_queue = unsafe {
    let ptr = queue_ptr.as_ptr() as *mut VirtQueue<QUEUE_SIZE>;
    ptr.write(VirtQueue::new());
    &mut *ptr
};
```

### Fix 4: Update VirtioGpu Struct

**File**: `levitate-drivers-gpu/src/device.rs`

```rust
pub struct VirtioGpu<H: VirtioHal> {
    transport: MmioTransport,
    // Change from Box to raw pointer management
    control_queue_ptr: NonNull<VirtQueue<QUEUE_SIZE>>,
    control_queue_paddr: u64,
    control_queue_pages: usize,
    // ... rest unchanged
}
```

### Fix 5: Implement Drop for Cleanup

**File**: `levitate-drivers-gpu/src/device.rs`

```rust
impl<H: VirtioHal> Drop for VirtioGpu<H> {
    fn drop(&mut self) {
        self.transport.reset();
        unsafe {
            H::dma_dealloc(
                self.control_queue_paddr,
                self.control_queue_ptr.cast(),
                self.control_queue_pages
            );
        }
        // ... existing framebuffer cleanup
    }
}
```

---

## Phase 3 Complete

Fix design documented. Proceed to Phase 4 for implementation steps.
