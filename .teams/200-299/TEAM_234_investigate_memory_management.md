# TEAM_234: Memory Management Investigation

## Status: COMPLETE

## Summary

Investigated and documented the complete memory management architecture in LevitateOS.

---

## Memory Management Architecture Overview

LevitateOS implements a layered memory management system supporting both kernel and userspace memory needs on AArch64.

### 1. Physical Memory Management (Buddy Allocator)

**Location:** `crates/hal/src/allocator/buddy.rs`

- **Algorithm:** Classic buddy system with coalescing
- **Max Order:** 21 (supports up to 8GB: 2^21 × 4KB)
- **Page Size:** 4KB (4096 bytes)
- **Data Structure:** Per-page descriptors in `Page` struct with intrusive doubly-linked lists

**Key Features:**
- O(log n) allocation and free operations
- Automatic buddy coalescing on free
- Memory map dynamically placed during boot (avoids reserved regions)
- Thread-safe via `Mutex<BuddyAllocator>` wrapper

**Page Descriptor (`crates/hal/src/allocator/page.rs`):**
```rust
pub struct Page {
    pub flags: PhysPageFlags,  // ALLOCATED, HEAD, TAIL, FREE
    pub order: u8,             // Order of allocation
    pub refcount: u16,         // For future CoW support
    pub next: Option<NonNull<Page>>,
    pub prev: Option<NonNull<Page>>,
}
```

### 2. Kernel Heap (linked_list_allocator)

**Location:** `kernel/src/arch/aarch64/boot.rs`

- Uses `linked_list_allocator` crate
- Initialized from linker symbols (`__heap_start` to `__heap_end`)
- Provides `#[global_allocator]` for kernel `alloc` crate usage

```rust
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();
```

### 3. Virtual Memory / MMU (`crates/hal/src/mmu.rs`)

**Architecture:** AArch64 4-level page tables (4KB granule, 48-bit VA)

**Address Spaces:**
- **TTBR1 (Kernel):** `0xFFFF_8000_0000_0000` and above
- **TTBR0 (User):** Below `0x0000_8000_0000_0000`

**Page Table Structure:**
- L0 → L1 → L2 → L3 (each 512 entries, 8 bytes each)
- Supports 2MB block mappings at L2 for efficiency
- Dynamic page table allocation via `PageAllocator` trait

**Key Functions:**
- `map_page()` - Map single 4KB page
- `map_block_2mb()` - Map 2MB block
- `unmap_page()` - Unmap with table reclamation
- `translate()` - VA → PA translation
- `switch_ttbr0()` - Context switch user address space

**Page Flags:**
- `KERNEL_CODE`, `KERNEL_DATA`, `DEVICE` - Kernel mappings
- `USER_CODE`, `USER_DATA`, `USER_STACK` - User mappings
- `*_BLOCK` variants for 2MB mappings

### 4. User Address Space (`kernel/src/memory/user.rs`)

**Layout Constants:**
```rust
STACK_TOP: 0x0000_7FFF_FFFF_0000
STACK_SIZE: 65536 (64KB)
USER_SPACE_END: 0x0000_8000_0000_0000
```

**Key Functions:**
- `create_user_page_table()` - Allocate new TTBR0 table
- `setup_user_stack()` - Allocate and map stack pages
- `setup_stack_args()` - Set up argc/argv/envp/auxv (Linux ABI)
- `alloc_and_map_heap_page()` - Heap growth via sbrk
- `user_va_to_kernel_ptr()` - Translate user VA for kernel access
- `validate_user_buffer()` - Validate user pointers in syscalls

### 5. Process Heap (`kernel/src/memory/heap.rs`)

**Per-Process State:**
```rust
pub struct ProcessHeap {
    pub base: usize,    // Start of heap (from ELF brk)
    pub current: usize, // Current program break
    pub max: usize,     // base + 64MB max
}
```

- Maximum heap: 64MB per process
- `grow()` handles both positive and negative increments
- Bounds checking prevents underflow/overflow

### 6. Userspace Allocator (`userspace/ulib/src/alloc.rs`)

**Type:** Bump allocator backed by `sbrk` syscall

**Characteristics:**
- O(1) allocation
- No deallocation (memory reclaimed on process exit)
- Grows heap on demand via `sbrk()`
- Handles zero-size allocations per `GlobalAlloc` contract

---

## Memory Initialization Sequence

1. **Early Boot (`boot.rs`):**
   - `init_heap()` - Initialize kernel heap from linker symbols
   - `init_mmu()` - Set up kernel page tables, enable MMU

2. **System Init (`init.rs`):**
   - `crate::memory::init(dtb)` - Initialize buddy allocator
   - Parses DTB for RAM regions and reserved areas
   - Dynamically places mem_map in available RAM
   - Registers allocator with MMU for page table allocation

3. **Process Creation:**
   - `create_user_page_table()` - New TTBR0
   - ELF loading maps code/data segments
   - `setup_user_stack()` - Stack allocation
   - `ProcessHeap::new(brk)` - Initialize heap state

---

## Key Design Decisions

1. **Buddy Allocator for Physical Memory:** Efficient for variable-size allocations with coalescing
2. **linked_list_allocator for Kernel Heap:** Simple, proven, no external dependencies
3. **Bump Allocator for Userspace:** Simple, fast, sufficient for single-threaded processes
4. **Separate TTBR0/TTBR1:** Clean kernel/user separation, kernel always accessible
5. **2MB Block Mappings:** Reduces TLB pressure for large mappings
6. **Page Table Reclamation:** `unmap_page()` frees empty tables

---

## TODOs Found

1. `destroy_user_page_table()` - Not implemented (leaks pages)
2. Full page table teardown on process exit
3. CoW support (refcount field exists but unused)
4. More sophisticated userspace allocator (e.g., dlmalloc)

---

## Files Analyzed

- `crates/hal/src/allocator/buddy.rs` - Buddy allocator
- `crates/hal/src/allocator/page.rs` - Page descriptor
- `crates/hal/src/allocator/intrusive_list.rs` - List implementation
- `crates/hal/src/mmu.rs` - MMU/page table management
- `kernel/src/memory/mod.rs` - Frame allocator wrapper
- `kernel/src/memory/user.rs` - User address space
- `kernel/src/memory/heap.rs` - Process heap state
- `kernel/src/arch/aarch64/boot.rs` - Kernel heap init
- `kernel/src/syscall/mm.rs` - sbrk syscall
- `userspace/ulib/src/alloc.rs` - Userspace allocator

## Handoff

Investigation complete. No blockers. The memory management system is well-structured and documented.
