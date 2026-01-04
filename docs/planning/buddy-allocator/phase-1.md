# Phase 1: Buddy Allocator Discovery

**Status**: [ ] Draft | [x] In Review | [ ] Approved
**Owner**: TEAM_041
**Reviewed By**: TEAM_042 (2026-01-04)
**Related Feature**: Memory Management II (Phase 5)
**Target Hardware**: Pixel 6 (8GB RAM)

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
- [ ] **Robustness**: Uses a `struct Page` array (mem_map) for reliable state tracking, not just intrusive lists.
- [ ] **Flexibility**: Initializes memory sizes dynamically via Device Tree (DTB), not hardcoded constants.

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
  - `get_dtb_phys` exists but is only used for initramfs detection.
- `linker.ld` (implied):
  - Defines `__heap_start` / `__heap_end`.

## 4. Constraints

- **No_std**: Must operate without the standard library.
- **Concurrency**: Must be thread-safe (spinlocked) as it will be a global resource.
- **Bootstrapping**: The allocator itself needs memory to store its metadata (`mem_map` array). This metadata requires a significant chunk of early memory (e.g., 24 bytes per 4KB page -> ~6MB for 1GB RAM, **48MB for 8GB Pixel 6**). This logic is complex: we must find where to put the map *before* the map exists.
- **Memory Holes**: Physical memory is not contiguous (MMIO holes, reserved regions). The allocator must handle disjoint ranges (e.g., using an array of free lists + a frame map).
- **Target Scale**: Must support 8GB RAM (Pixel 6) without artificial caps.

## 5. Preliminary Strategy

1. **Metadata Storage**: Use a global `mem_map` (slice of `struct Page`) to track every physical frame's state (Free/Allocated, Order, Flags).
   - *Challenge*: Computing the size of this array and placing it in memory during early boot.
2. **Initialization**:
   - Parse Device Tree (DTB) to find all valid RAM regions.
   - Subtract reserved regions (Kernel image, Initramfs, DTB itself).
   - Calculate size required for `mem_map`.
   - Place `mem_map` in the first available large contiguous block.
   - Initialize the `mem_map` and populate the Buddy free lists.
3. **API**:
   - `alloc_frame() -> Option<PhysAddr>`
   - `alloc_frames(order: usize) -> Option<PhysAddr>`
   - `free_frame(frame: PhysAddr)`
4. **Heap Hierarchy**:
   - The Kernel Heap (`LockedHeap`) starts empty.
   - When `alloc` fails, it requests a large block (e.g., 64KB) from Buddy Allocator and adds it to the heap.
