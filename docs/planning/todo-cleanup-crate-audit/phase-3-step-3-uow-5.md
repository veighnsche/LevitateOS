# UoW 3.5: Implement sys_munmap Using VMA

**Parent:** Phase 3, Step 3 (VMA Tracking)  
**File:** `kernel/src/syscall/mm.rs`  
**Depends On:** UoW 3.4  
**Complexity:** Medium

---

## Context

With VMA tracking in place, we can now implement proper `sys_munmap`.

**Current stub location:**
```rust
// kernel/src/syscall/mm.rs:177
// TODO(TEAM_228): Implement proper VMA tracking and page unmapping
```

---

## Task

Implement `sys_munmap` that:
1. Looks up VMAs covering the range
2. Unmaps and frees pages
3. Updates the VMA list

---

## Implementation

Replace the stub `sys_munmap` with:

```rust
/// TEAM_235: sys_munmap - Unmap a memory region.
pub fn sys_munmap(addr: usize, len: usize) -> i64 {
    use los_hal::mmu::{PageAllocator, PAGE_SIZE, phys_to_virt, PageTable, tlb_flush_page};
    
    // Validate alignment
    if addr & 0xFFF != 0 {
        return EINVAL;
    }
    
    if len == 0 {
        return EINVAL;
    }
    
    // Page-align length
    let aligned_len = (len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
    let end = match addr.checked_add(aligned_len) {
        Some(e) => e,
        None => return EINVAL, // Overflow
    };
    
    let task = crate::task::current_task();
    let ttbr0 = task.ttbr0;
    
    // 1. Remove VMA(s) from tracking
    {
        let mut vmas = task.vmas.lock();
        if vmas.remove(addr, end).is_err() {
            // No VMA found - per POSIX, this is not necessarily an error
            // but we'll return success anyway (Linux behavior)
            log::trace!("[MUNMAP] No VMA found for 0x{:x}-0x{:x}", addr, end);
        }
    }
    
    // 2. Unmap and free pages
    let l0_va = phys_to_virt(ttbr0);
    let l0 = unsafe { &mut *(l0_va as *mut PageTable) };
    
    let mut current = addr;
    while current < end {
        // Try to find the page mapping
        if let Ok(walk) = los_hal::mmu::walk_to_entry(l0, current, 3, false) {
            let entry = walk.table.entry(walk.index);
            if entry.is_valid() {
                // Get physical address before clearing
                let phys = entry.address();
                
                // Clear the entry
                walk.table.entry_mut(walk.index).clear();
                
                // Flush TLB for this page
                tlb_flush_page(current);
                
                // Free the physical page
                FRAME_ALLOCATOR.free_page(phys);
            }
        }
        
        current += PAGE_SIZE;
    }
    
    log::trace!("[MUNMAP] Unmapped 0x{:x}-0x{:x}", addr, end);
    
    0 // Success
}
```

---

## Key Behaviors

1. **Alignment:** Address must be page-aligned, length is rounded up
2. **VMA Removal:** Updates VmaList (handles splits if partial unmap)
3. **Page Freeing:** Each mapped page is unmapped and freed
4. **TLB Flush:** Each page's TLB entry is invalidated
5. **Error Handling:** Returns EINVAL for bad args, 0 on success

---

## Acceptance Criteria

- [ ] Address alignment validated
- [ ] Length rounded to page boundary
- [ ] VMA removed/split correctly
- [ ] All pages in range unmapped
- [ ] Physical pages freed
- [ ] TLB flushed
- [ ] Returns 0 on success

---

## Testing

```bash
cargo build -p levitate-kernel
cargo xtask test
```

---

## Step 3 Complete

After this UoW, Step 3 (VMA Tracking) is complete. Phase 3 is now ready for implementation.
