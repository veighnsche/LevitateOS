# Task 7.1: Virtual Memory Reclamation (`unmap_page`)

## Objective
Implement a robust `unmap_page` function in the MMU module to allow safe removal of virtual-to-physical mappings and reclamation of physical frames.

## Requirements
1. **Precision**: Must only remove the requested page(s).
2. **TLB Consistency**: Must invalidate the TLB for the unmapped address to prevent stale entries.
3. **Safety**: Must handle cases where the mapping doesn't exist (return error).
4. **Intermediate Table Management**: (Phase B) Detect if a page table becomes empty after an unmap and free it via `PageAllocator`.

## Success Criteria
- [ ] **SC1**: `unmap_page(va)` clears the correct L3 entry.
- [ ] **SC2**: Reading from a previously unmapped address triggers a Translation Fault.
- [ ] **SC3**: `unmap_page` calls `tlb_flush_page(va)`.
- [ ] **SC4**: intermediate tables are reclaimed when empty.

## Implementation Plan

### Step 1: MMU Helper for Table Walking
Refactor existing `map_page` logic to separate the "walking" from the "setting".
Create `fn walk_to_entry(root: &mut PageTable, va: usize, create: bool) -> Result<&mut PageTableEntry, Error>`.

### Step 2: Basic `unmap_page`
```rust
pub fn unmap_page(root: &mut PageTable, va: usize) -> Result<(), &'static str> {
    // 1. Walk to L3 entry WITHOUT creating new tables
    // 2. Clear the entry
    // 3. Invalidate TLB
    // 4. Return success
}
```

### Step 3: Verification via Unit Test
Create a test in `mmu.rs` that:
1. Maps a page.
2. Verifies entry is valid.
3. Unmaps the page.
4. Verifies entry is invalid.

## Next Steps
1. Refactor `mmu.rs` to support granular unmapping.
2. Implement TLB flushing logic.
