# Phase 3: Buddy Allocator Implementation

**Status**: [ ] Pending Design Approval | [x] In Review | [ ] Approved
**Owner**: TEAM_041
**Reviewed By**: TEAM_042 (2026-01-04)
**Target Hardware**: Pixel 6 (8GB RAM)

## 1. Implementation Steps

### Step 1: Memory Map & Data Structures
**File**: `kernel/src/memory/page.rs` (New)
- Define `struct Page`.
- Define flags (HEAD, TAIL, FREE).
- *Complexity*: Needs to be compact.

### Step 2: DTB Memory Parsing
**File**: `levitate-hal/src/fdt.rs`
- Add `get_memory_regions(dtb) -> Iterator<Item=Range>`.
- Add `get_reserved_regions(dtb) -> Iterator<Item=Range>`.
- *Validation*: Verify against QEMU dump.

### Step 3: Bootstrapping Logic (The "Hard" Part)
**File**: `kernel/src/memory/mod.rs` (`init`)
- **Action**:
  1. Parse DTB to find total RAM range.
  2. Calculate required `mem_map` size.
  3. Find a hole in RAM that fits `mem_map` (and doesn't overlap kernel/initrd).
  4. "Steal" that range for the `mem_map`.
  5. Initialize `BuddyAllocator` with that `mem_map`.
  6. Loop through RAM again, calling `allocator.add_memory()` for all non-reserved regions.
- *Note*: This must happen *before* the Heap is valid.

### Step 4: Allocator Logic
**File**: `kernel/src/memory/buddy.rs`
- Implement `alloc(order)`:
  - Find first available order >= requested.
  - Split down to target order.
  - Update `Page` flags (Head/Tail/Order).
- Implement `free(ptr, order)`:
  - Validate `Page` flags.
  - Check buddy `Page`. If free & same order, merge.
  - Repeat.

### Step 5: Integration
**File**: `kernel/src/main.rs`
- Replace hardcoded Heap init (lines 261-272 currently use `__heap_start`/`__heap_end` linker symbols).
- Call `memory::init()` **before** any heap usage.
- Calculate heap size dynamically:
  ```rust
  let heap_size = buddy.recommended_heap_size(); // total_ram/128, [16MB, 64MB]
  let heap_order = (heap_size.trailing_zeros() - 12) as usize; // Convert to order
  let heap_pa = buddy.alloc(heap_order).expect("Failed to allocate heap");
  let heap_va = phys_to_virt(heap_pa);
  ALLOCATOR.lock().init(heap_va as *mut u8, heap_size);
  ```

### Step 6: MMU Integration (Q3 Decision: Trait Injection)
**File**: `levitate-hal/src/mmu.rs`
1. Define `PageAllocator` trait (see phase-2.md Section 3.2)
2. Add `set_page_allocator(alloc: &'static dyn PageAllocator)` function
3. Modify `alloc_page_table()`:
   ```rust
   fn alloc_page_table() -> Option<&'static mut PageTable> {
       // Try dynamic allocator first
       if let Some(alloc) = get_page_allocator() {
           if let Some(pa) = alloc.alloc_page() {
               let va = phys_to_virt(pa);
               let pt = unsafe { &mut *(va as *mut PageTable) };
               pt.zero();
               return Some(pt);
           }
       }
       // Fallback to PT_POOL during early boot
       alloc_from_static_pool()
   }
   ```
4. Keep `PT_POOL` for boot-time allocation before Buddy is initialized

### Step 7: Error Handling (Q4 Decision: Panic)
**File**: `kernel/src/memory/mod.rs`
Add explicit panics with clear messages:
```rust
pub fn init(dtb: &[u8]) {
    let regions = fdt::get_memory_regions(dtb);
    if regions.is_empty() {
        panic!("PANIC: No RAM regions found in DTB");
    }
    
    let free_regions = subtract_reserved(regions, reserved);
    if free_regions.is_empty() {
        panic!("PANIC: All RAM is reserved — no free memory");
    }
    
    let mem_map_size = calculate_mem_map_size(total_pages);
    let mem_map_location = find_contiguous_hole(free_regions, mem_map_size)
        .expect("PANIC: Cannot allocate mem_map — insufficient contiguous memory");
    
    // ... continue initialization
}
```

## 2. Dependencies
- `levitate-hal` FDT updates must happen first.
- ✅ Questions Q1-Q4 answered (TEAM_042, 2026-01-04) — see `.questions/TEAM_042_buddy_allocator_questions.md`
