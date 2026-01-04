# Phase 2: Buddy Allocator Design

**Status**: [ ] Draft | [ ] In Review | [ ] Approved
**Owner**: TEAM_041

## 1. Proposed Solution
We will implement a **Binary Buddy Allocator**.
- **Structure**: An array of linked lists, `free_lists[order]`.
- **Order 0**: 4KB pages.
- **Max Order**: Likely 18 (1GB) or similar, depending on max contiguous requirement. 2MB (Order 9) is critical for huge pages.
- **Mechanism**:
  - `alloc(order)`: Check `free_lists[order]`. If empty, go to `order+1`, split it, put half in `free_lists[order]`, return other half.
  - `free(ptr, order)`: Check if buddy is free. If yes, merge and move to `free_lists[order+1]`. Repeat.

## 2. Behavioral Decisions & Open Questions

> [!IMPORTANT]
> The following questions need answers before implementation.

### Q1: Metadata Storage
How do we store the "next" pointers for the free lists?
- **Option A (Intrusive)**: Store the `next` pointer *inside* the free page itself.
  - *Pros*: Zero external memory overhead.
  - *Cons*: Can't read metadata if the page is mapped as Request-Only or Device (not an issue for free RAM).
- **Option B (Bitmap/Array)**: Static array of `Page` structs.
  - *Pros*: Easier to lookup buddy status.
  - *Cons*: Significant persistent memory overhead (e.g. `sizeof(Page) * num_pages`).
- **Recommendation**: **Option A (Intrusive)** for the free lists, combined with a minimal **Bitset** for "is_allocated" status if needed for `merge` checks, although buddy addresses can be calculated arithmetically.

### Q2: Heap Integration
How does this interact with `linked_list_allocator`?
- **Option A**: `LockedHeap` takes a fixed large static region (current). Buddy Allocator manages *rest* of RAM.
- **Option B**: `LockedHeap` starts small and requests pages from Buddy Allocator to grow.
- **Recommendation**: **Option A** for Phase 5 initial steps (keep it simple), then move to **Option B** later.

### Q3: Memory Map Parsing
Where do we get the available RAM regions?
- **Current**: Hardcoded/Linker script.
- **Future**: Device Tree (DTB) parsing (from Phase 4).
- **Plan**: Phase 4 added DTB parsing. Phase 5 should use it.

## 3. API Design

```rust
pub struct BuddyAllocator {
    heads: [Option<NonNull<FreeBlock>>; MAX_ORDER],
}

impl BuddyAllocator {
    /// Add a range of physical memory to the allocator.
    /// unsafe: Caller must ensure range is unused and valid RAM.
    pub unsafe fn add_region(&mut self, start: PhysAddr, end: PhysAddr);

    /// Allocate a frame of order `order`.
    pub fn allocate(&mut self, order: usize) -> Option<PhysAddr>;

    /// Free a frame of order `order`.
    /// unsafe: Caller must ensure frame was allocated and size matches.
    pub unsafe fn deallocate(&mut self, frame: PhysAddr, order: usize);
}
```

## 4. Dependencies
- `levitate-hal::mmu`: Access to physical address types.
- `levitate-utils::spinlock`: Need `IrqSafeLock<BuddyAllocator>`? Using a globally locked allocator is standard.
