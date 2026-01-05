# TEAM_075: Investigate Heap Corruption via Reference Kernels

## Status
**FIXED** - Heap/mem_map overlap resolved.

## Bug Summary
Heap overlap with `mem_map` (frame allocator metadata). The `_kernel_end` symbol doesn't include the heap range, causing the frame allocator to place `mem_map` inside the kernel heap.

---

## Reference Kernel Analysis

### 1. Redox Kernel Approach
**File:** `src/arch/aarch64/consts.rs`

Redox uses a **virtual address layout with explicit constants**:
```rust
pub const KERNEL_HEAP_OFFSET: usize = KERNEL_OFFSET - PML4_SIZE;
pub const KERNEL_HEAP_SIZE: usize = 1 * 1024 * 1024; // 1 MB
```

**Key Pattern:**
- Heap is in a **completely separate virtual region** (different PML4 entry)
- No linker script involvement for heap placement
- Heap pages are **demand-mapped** at runtime via `map_heap()`
- Physical frames for heap are allocated dynamically, not reserved at boot

**Why this works:** The heap virtual address is far away from kernel code/data, so there's no chance of overlap with allocator metadata.

### 2. Theseus Kernel Approach
**File:** `kernel/frame_allocator/src/lib.rs`

Theseus uses **explicit reserved regions with priority subtraction**:
```rust
pub fn init<F, R, P>(
    free_physical_memory_areas: F,
    reserved_physical_memory_areas: R,  // <-- KEY: explicit reserved list
) -> Result<...>
```

**Key Pattern:**
- Frame allocator init takes **both free AND reserved regions** as arguments
- **Reserved regions take priority** - any overlap is carved out from free regions
- Uses `check_and_add_free_region()` to recursively subtract reserved from free
- Final sanity check: `panic!` if any two regions overlap

**From `kernel_config/memory.rs`:**
```rust
pub const KERNEL_HEAP_START: usize = ...;
pub const KERNEL_HEAP_INITIAL_SIZE: usize = ...;
```

**Why this works:** Reserved regions are explicitly enumerated and the frame allocator guarantees no allocations from reserved memory.

### 3. Tock Kernel
Tock uses a simpler static allocation model not directly applicable to this issue.

---

## Root Cause in LevitateOS

Looking at `kernel/src/memory/mod.rs`:
```rust
let kernel_start = mmu::virt_to_phys(unsafe { &_kernel_virt_start as *const _ as usize });
let kernel_end = mmu::virt_to_phys(unsafe { &_kernel_end as *const _ as usize });
```

**Problem:** Even though `linker.ld` now advances `.` to include the heap before setting `_kernel_end`, the debug output still shows `0x400c6b20` instead of `0x41F00000`.

**Possible causes:**
1. **Stale build artifacts** - linker script change not recompiled
2. **Symbol reading issue** - `&_kernel_end as *const _ as usize` might not work as expected
3. **Build cache** - `cargo clean` may not have purged all artifacts

---

## Recommended Solutions

### Solution A: Explicit `__heap_end` Import (Immediate Fix)
As suggested in `docs/debugging_memory_corruption.md`:

```rust
unsafe extern "C" {
    static _kernel_virt_start: u8;
    static _kernel_end: u8;
    static __heap_end: u8;  // Add this
}

// In init():
let kernel_end_symbol = mmu::virt_to_phys(unsafe { &_kernel_end as *const _ as usize });
let heap_end_symbol = mmu::virt_to_phys(unsafe { &__heap_end as *const _ as usize });
let reserved_end = core::cmp::max(kernel_end_symbol, heap_end_symbol);
```

**Pros:** Simple, explicit, robust against linker script quirks.
**Cons:** Belt-and-suspenders approach.

### Solution B: Redox-Style Separate Heap Region (Better Long-Term)
Move the heap to a separate virtual region:

1. Define heap as a runtime-allocated region, not linker-placed
2. Use constants like:
   ```rust
   pub const KERNEL_HEAP_VA: usize = 0xFFFF_8001_0000_0000; // Separate from kernel image
   pub const KERNEL_HEAP_SIZE: usize = 32 * 1024 * 1024;    // 32 MB
   ```
3. Map heap pages on demand from the frame allocator

**Pros:** Clean separation, no overlap possible.
**Cons:** Larger change, requires heap initialization rework.

### Solution C: Theseus-Style Explicit Reserved List (Most Robust)
Pass heap region explicitly to frame allocator init:

```rust
// Before calling frame allocator init:
let heap_start_phys = mmu::virt_to_phys(__heap_start);
let heap_end_phys = mmu::virt_to_phys(__heap_end);
add_reserved(&mut reserved_regions, &mut res_count, heap_start_phys, heap_end_phys);
```

**Pros:** Explicit, follows production kernel patterns.
**Cons:** Requires importing `__heap_start` and `__heap_end`.

---

## Recommended Action

**For immediate fix:** Apply Solution A or C - import `__heap_end` and ensure the heap range is reserved.

**For long-term:** Consider Solution B (Redox-style) where heap is in a separate virtual region.

---

## Verification Checklist
- [ ] `cargo clean && cargo xtask run` boots without Data Abort
- [ ] Log shows `[MEMORY] Reserved Kernel: ... - 0x41F00000` (or higher)
- [ ] Userspace "Hello World" prints successfully

---

## Fix Applied

**File:** `kernel/src/memory/mod.rs`

**Changes:**
1. Added `__heap_end` to extern "C" block
2. Modified reserved region calculation to use `max(_kernel_end, __heap_end)`

```rust
let kernel_end_symbol = mmu::virt_to_phys(unsafe { &_kernel_end as *const _ as usize });
let heap_end_symbol = mmu::virt_to_phys(unsafe { &__heap_end as *const _ as usize });
let kernel_end = core::cmp::max(kernel_end_symbol, heap_end_symbol);
```

**Verification Output:**
```
[MEMORY] Reserved Kernel: 0x400801e8 - 0x41f00000 (heap_end: 0x41f00000)
[MEMORY] Allocated mem_map at PA 0x42000000 (Size 3145728 bytes)
```

- Reserved range now extends to `0x41f00000` (includes heap)
- `mem_map` placed at `0x42000000` (after heap end)
- No Data Abort crash
- User process creation succeeds

---

## Breadcrumbs Left
- `// TEAM_075:` comment in `kernel/src/memory/mod.rs` at the fix location

## Cleanup Work Completed

**Rule 4 (Silence is Golden) compliance:**
- Removed `[BUDDY]` debug prints from `levitate-hal/src/allocator/buddy.rs`
- Removed `[MEMORY]` debug prints from `kernel/src/memory/mod.rs`

**Rule 5 (Memory Safety) compliance:**
- Added `// SAFETY:` comments to all `unsafe` blocks in both files

## Remaining Issues
- Userspace process hangs after creation - **separate bug** (scheduling/context switch issue)

## Handoff
The heap corruption bug is **FIXED** and code is now compliant with kernel-development.md rules. Next team should:
1. Investigate userspace process scheduling hang (separate bug)
