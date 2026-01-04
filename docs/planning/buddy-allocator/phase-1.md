# Phase 1: Buddy Allocator Discovery

**Status**: [ ] Draft | [ ] In Review | [ ] Approved
**Owner**: TEAM_041
**Related Feature**: Memory Management II (Phase 5)

## 1. Feature Summary

LevitateOS currently relies on static memory regions and a simple `linked_list_allocator` for the kernel heap. There is no dynamic physical frame allocator. To support dynamic page table creation (beyond the static pool), userspace process creation, and large contiguous DMA buffers, we need a robust physical memory allocator.

The **Buddy Allocator** will manage the physical RAM availability, allowing the kernel to request contiguous blocks of physical memory in powers of two (2^order pages).

## 2. Success Criteria

- [ ] **Frame Allocation**: Can allocate and free 4KB frames (Order 0).
- [ ] **Multi-order Support**: Can allocate contiguous regions (e.g., 2MB = Order 9) for huge pages.
- [ ] **Coalescing**: Freeing adjacent blocks merges them into a larger block.
- [ ] **Splitting**: Allocating a small block splits larger blocks as needed.
- [ ] **Efficiency**: O(1) or O(log N) operations; minimal fragmentation overhead.
- [ ] **Integration**: Replaces or supplements the static `PT_POOL` in `mmu.rs`.

## 3. Current State Analysis

### 3.1 Memory Management Today
- **Heap**: `linked_list_allocator::LockedHeap` manages a fixed range (`__heap_start` to `__heap_end`). This is a byte-granularity allocator for `Box`, `Vec`, etc.
- **Page Tables**: `levitate-hal/src/mmu.rs` uses a `static mut PT_POOL` (array of 16 tables) for creating new page tables. This is non-scalable and prone to exhaustion.
- **Physical Memory**: No centralized tracking of which physical frames are free or used. Boot assembly sets up initial IDENTITY and HIGH-KERNEL mappings, but "free RAM" is just implicitly anything not used by the kernel image or stack.

### 3.2 Codebase Reconnaissance
- `levitate-hal/src/mmu.rs`:
  - `PT_POOL`: Needs to be replaced by dynamic allocation.
  - `map_page`: Currently takes `pa: usize`. A higher-level API might need `map_new_page` which asks the allocator for a frame.
- `kernel/src/main.rs`:
  - Initializes `ALLOCATOR` (heap).
  - Explicitly maps regions in `kmain`.
- `linker.ld` (implied):
  - Defines `__heap_start` / `__heap_end`. We may need to redefine `__heap_end` to be the end of the *static* kernel data, and give the rest of RAM to the Buddy Allocator.

## 4. Constraints

- **No_std**: Must operate without the standard library.
- **Concurrency**: Must be thread-safe (spinlocked) as it will be a global resource.
- **Bootstrapping**: The allocator itself needs memory to store its metadata (bitmaps or linked lists). This metadata must be placed carefully in memory that isn't yet managed.
- **Memory Holes**: Physical memory is not contiguous (MMIO holes, reserved regions). The allocator must handle disjoint ranges (e.g., using an array of free lists + a frame map).

## 5. Preliminary Strategy

1. **Metadata Storage**: Since `Box` isn't available until the heap is up, and the heap needs pages from the allocator (chicken-and-egg), the Buddy Allocator's metadata usually lives inside the free pages themselves (linked list of free nodes), or in a statically reserved region.
2. **Initialization**:
   - Parse Device Tree / UEFI / Atags to find usable RAM regions.
   - Mark kernel code/data/stack as "used".
   - Add remaining regions to the Buddy Allocator.
3. **API**:
   - `alloc_frame() -> Option<PhysAddr>`
   - `alloc_frames(order: usize) -> Option<PhysAddr>`
   - `free_frame(frame: PhysAddr)`
