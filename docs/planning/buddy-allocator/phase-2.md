# Phase 2: Buddy Allocator Design

**Status**: [ ] Draft | [x] In Review | [ ] Approved
**Owner**: TEAM_041
**Reviewed By**: TEAM_042 (2026-01-04)
**Target Hardware**: Pixel 6 (8GB RAM)

## 1. Proposed Solution
We will implement a **Production-Grade Binary Buddy Allocator**.

### 1.1 Core Architecture
- **Global `mem_map`**: A contiguous array of `struct Page` descriptors mapping 1:1 to physical frames.
  - `struct Page` tracks: `flags` (IsFree, IsHead, IsTail), `order` (if Head), `refcount`.
- **Free Lists**: An array of linked lists `free_lists[order]` (Order 0..18).
  - Nodes in the linked list are stored *inside* the `struct Page` entries themselves (or the physical pages), but using `struct Page` allows safe validation of buddy state without touching potentially uncached/device physical memory.

### 1.2 Bootstrapping Flow
1. **DTB Parse**: Identify all physical RAM banks.
2. **Reservation**: Mark kernel sections (`.text`, `.data`, `.bss`), DTB, and Initramfs as RESERVED.
3. **MemMap Allocation**: Calculate `sizeof(Page) * total_frames`. Find a hole large enough to hold this array.
4. **Initialization**: 
   - Zero the `mem_map`.
   - Iterate all free RAM regions.
   - For each valid PFN, marking it as FREE and adding it to the Buddy System.

## 2. Behavioral Decisions

### Q1: Metadata Storage
**Decision**: **Separate `struct Page` Array (`mem_map`)**.
- **Reasoning**: To implement a robust kernel, we need to know the state of every page (e.g., "Is this page shared?", "Is this a head of a compound page?"). Relying solely on intrusive lists in free memory is fragile and prevents advanced features (CoW, shared memory) later.
- **Implementation**: A static slice `&'static mut [Page]` initialized at boot.

### Q2: Heap Integration
**Decision**: **Dynamic Hierarchy with Scaled Sizing** (Q1 Answered).
- **Reasoning**: The Buddy Allocator is the SSOT for physical memory. The Byte-granularity Heap (`LockedHeap`) is just a client.
- **Implementation**:
  - `LockedHeap` initialized as empty.
  - Calculate heap size: `total_ram / 128`, clamped to `[16MB, 64MB]`
    - QEMU 1GB → 16MB heap (Order 12)
    - Pixel 6 8GB → 64MB heap (Order 14)
  - Allocate from Buddy and pass to `LockedHeap::init()`
  - No expansion logic needed initially — 64MB is generous for kernel heap

### Q3: Memory Map Parsing
**Decision**: **DTB Driven**.
- **Reasoning**: Hardcoding RAM sizes prevents portability.
- **Implementation**: Update `levitate-hal/fdt.rs` to parse `/memory` nodes and `/reserved-memory` nodes.

## 3. API Design

### 3.1 Data Structures

```rust
#[repr(C)]
pub struct Page {
    flags: u8,      // 1=Allocated, 2=Head, 4=Tail
    order: u8,      // Only valid if Head
    refcount: u16,  // For future shared pages
    // Intrusive list pointers for free_list (only used if flags & FREE)
    next: Option<NonNull<Page>>,
    prev: Option<NonNull<Page>>, 
} // Size: ~24 bytes (on 64-bit)

pub struct BuddyAllocator {
    mem_map: &'static mut [Page],
    free_lists: [ListHead; MAX_ORDER],
    start_pfn: usize,
}

// Q2 Decision: No cap on RAM size. 8GB Pixel 6 = 48MB metadata (~0.6% overhead).
pub const MAX_ORDER: usize = 21; // 2^21 pages = 8TB theoretical max
```

### 3.2 PageAllocator Trait (Q3 Decision)

```rust
// Defined in levitate-hal/src/mmu.rs
pub trait PageAllocator: Send + Sync {
    /// Allocate a single 4KB page, returns physical address
    fn alloc_page(&self) -> Option<usize>;
    /// Free a single 4KB page
    fn free_page(&self, pa: usize);
}

// Storage for runtime allocator (set after Buddy init)
static PAGE_ALLOCATOR: AtomicPtr<dyn PageAllocator> = AtomicPtr::new(core::ptr::null_mut());

pub fn set_page_allocator(alloc: &'static dyn PageAllocator) {
    PAGE_ALLOCATOR.store(alloc as *const _ as *mut _, Ordering::Release);
}
```

### 3.3 Interface

```rust
impl BuddyAllocator {
    /// Initialize with a slice of descriptors and a physical offset
    pub unsafe fn init(&mut self, mem_map: &'static mut [Page], start_pfn: usize);

    /// Bring a physical region online (mark as free and merge)
    /// Q4 Decision: Silently skips MMIO holes — not an error
    pub unsafe fn add_memory(&mut self, range: Range<PhysAddr>);

    pub fn alloc(&mut self, order: usize) -> Option<PhysAddr>;
    pub fn free(&mut self, frame: PhysAddr, order: usize);
    
    /// Calculate heap size: total_ram / 128, clamped [16MB, 64MB]
    pub fn recommended_heap_size(&self) -> usize {
        let total = self.total_pages() * PAGE_SIZE;
        (total / 128).clamp(16 * 1024 * 1024, 64 * 1024 * 1024)
    }
}

// Implement PageAllocator trait for MMU integration
impl PageAllocator for SpinLock<BuddyAllocator> {
    fn alloc_page(&self) -> Option<usize> {
        self.lock().alloc(0)
    }
    fn free_page(&self, pa: usize) {
        self.lock().free(pa, 0)
    }
}
```

## 4. Dependencies
- **Existing**: `levitate-hal::fdt` (needs expansion).
- **New**: `levitate-kernel::memory::buddy`.
