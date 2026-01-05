# Phase 2, Step 1: Fix VirtQueue DMA Bugs

**Parent:** [phase-2.md](./phase-2.md)  
**Status:** Planned  
**Estimated Effort:** 3-5 UoW

---

## Objective

Fix the VirtQueue implementation in `levitate-virtio` so that the GPU driver can communicate with the VirtIO device correctly.

---

## Root Cause Analysis (from TEAM_100)

The VirtQueue has fundamental DMA issues:

### Problem 1: Virtual vs Physical Addresses

**Current behavior:**
```rust
pub fn addresses(&self) -> (usize, usize, usize) {
    let desc_addr = self.descriptors.as_ptr() as usize;  // ❌ Virtual!
    let avail_addr = &self.avail_flags as *const u16 as usize;  // ❌ Virtual!
    let used_addr = &self.used_flags as *const u16 as usize;  // ❌ Virtual!
    (desc_addr, avail_addr, used_addr)
}
```

**Problem:** Device receives virtual addresses but needs physical addresses for DMA.

### Problem 2: Queue Memory Location

**Current behavior:**
- VirtQueue is stored as a field in the VirtioGpu struct
- This struct lives on the heap (dynamic allocation)
- Queue addresses are passed to device at init time

**Problem:** If struct moves, addresses become invalid. Also, queue memory must be DMA-accessible.

### Problem 3: Buffer Addresses in Descriptors

**Current behavior:**
```rust
desc.addr = input.as_ptr() as u64;  // ❌ Virtual address!
```

**Problem:** Command/response buffer addresses must be physical for device DMA.

### Problem 4: Volatile Access

**Current behavior:**
- `used_idx` is read as a normal field
- Device writes to this via DMA

**Problem:** Compiler may cache the value; need volatile reads.

---

## Solution Design

### Solution 1: Address Translation

Pass virt_to_phys function to all address-returning methods:

```rust
pub fn addresses<F>(&self, virt_to_phys: F) -> (usize, usize, usize)
where F: Fn(usize) -> usize
{
    let desc_addr = virt_to_phys(self.descriptors.as_ptr() as usize);
    let avail_addr = virt_to_phys(&self.avail_flags as *const u16 as usize);
    let used_addr = virt_to_phys(&self.used_flags as *const u16 as usize);
    (desc_addr, avail_addr, used_addr)
}
```

### Solution 2: DMA-Safe Memory Allocation

Option A: Allocate queue in DMA memory via HAL
```rust
let queue_mem = H::dma_alloc(pages_for(size_of::<VirtQueueMem>()));
```

Option B: Keep in-struct but ensure struct is in identity-mapped memory
- Current higher-half kernel has identity mapping for heap
- Virtual address = Physical address in mapped region
- Verify this assumption holds

### Solution 3: Buffer Address Translation

Already partially implemented - need to ensure add_buffer uses virt_to_phys:
```rust
pub fn add_buffer<F>(&mut self, inputs, outputs, virt_to_phys: F)
```

### Solution 4: Volatile Access

Already partially implemented - verify all device-written fields use volatile:
```rust
let device_used_idx = unsafe {
    core::ptr::read_volatile(&self.used_idx as *const u16)
};
```

### Solution 5: Async-Ready API Design

**Per user requirement (Q4 in TEAM_094 questions): Design API to be async-first.**

- Use polling/completion pattern instead of blocking waits
- Return `Poll<Result<T, E>>` or similar async-compatible type
- Avoid spin-waits in hot paths
- Design for future waker integration

Example pattern:
```rust
pub fn poll_command(&mut self) -> Poll<Result<Response, VirtQueueError>> {
    if self.is_complete() {
        Poll::Ready(self.take_response())
    } else {
        Poll::Pending
    }
}
```

---

## Tasks

### UoW 1: Verify Memory Mapping Assumptions
- Check if kernel heap is identity-mapped
- If yes, virt_to_phys is simple subtraction
- If no, need DMA allocation

### UoW 2: Fix addresses() Method
- Add virt_to_phys parameter
- Update all callers

### UoW 3: Verify add_buffer() Translation
- Confirm all buffer addresses translated
- Test with debug logging

### UoW 4: Verify Volatile Access
- Audit all device-written fields
- Add volatile read/write where missing

### UoW 5: Integration Test
- Run GPU init with fixed queue
- Verify GET_DISPLAY_INFO completes
- Run full boot test

---

## Fallback Plan

If VirtQueue DMA fix proves infeasible within 1 week of effort:

1. **Revisit Option B:** Keep `levitate-gpu` as canonical driver
2. **Delete `levitate-virtio-gpu`** instead of fixing it
3. **Focus remaining effort** on HAL cleanup and driver extraction only
4. **Document learnings** for future VirtIO implementation attempts

This ensures the reorganization can still proceed even if our custom VirtQueue has fundamental issues.

---

## Exit Criteria

- [ ] VirtQueue correctly translates all addresses to physical
- [ ] Device can read descriptor table
- [ ] Device can write to used ring
- [ ] GPU driver GET_DISPLAY_INFO succeeds
- [ ] No timeout errors
- [ ] Golden boot test passes
