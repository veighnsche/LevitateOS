# UoW 2.2: Integrate MmapGuard into sys_mmap

**Parent:** Phase 3, Step 2 (mmap Failure Cleanup)  
**File:** `kernel/src/syscall/mm.rs`  
**Depends On:** UoW 2.1  
**Complexity:** Low

---

## Context

UoW 2.1 created `MmapGuard`. Now we integrate it into `sys_mmap` to automatically clean up on failure.

---

## Task

Modify `sys_mmap` to use `MmapGuard` for automatic cleanup.

---

## Current Code Location

Look for `sys_mmap` in `kernel/src/syscall/mm.rs`. The allocation loop looks something like:

```rust
for page in start_page..end_page {
    let va = page * PAGE_SIZE;
    // ... allocate and map ...
}
```

---

## Implementation

Modify `sys_mmap` to:

1. Create guard at start
2. Track each allocation
3. Commit on success

```rust
pub fn sys_mmap(addr: usize, len: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> i64 {
    // ... existing validation code ...
    
    let task = crate::task::current_task();
    let ttbr0 = task.ttbr0;
    
    // TEAM_235: Create RAII guard for cleanup on failure
    let mut guard = MmapGuard::new(ttbr0);
    
    // ... calculate start_page, end_page, page_flags ...
    
    for page in start_page..end_page {
        let va = page * los_hal::mmu::PAGE_SIZE;
        
        // Skip if already mapped
        if mm_user::user_va_to_kernel_ptr(ttbr0, va).is_some() {
            continue;
        }
        
        // Allocate physical page
        let phys = match FRAME_ALLOCATOR.alloc_page() {
            Some(p) => p,
            None => {
                // Guard will clean up on drop
                return ENOMEM;
            }
        };
        
        // Zero the page
        let page_ptr = los_hal::mmu::phys_to_virt(phys) as *mut u8;
        unsafe {
            core::ptr::write_bytes(page_ptr, 0, los_hal::mmu::PAGE_SIZE);
        }
        
        // Map into user address space
        if unsafe { mm_user::map_user_page(ttbr0, va, phys, page_flags) }.is_err() {
            // Free this page (not tracked yet) and let guard clean up rest
            FRAME_ALLOCATOR.free_page(phys);
            return ENOMEM;
        }
        
        // TEAM_235: Track successful allocation
        guard.track(va, phys);
    }
    
    // TEAM_235: Success - commit the guard (pages kept)
    guard.commit();
    
    aligned_addr as i64
}
```

---

## Key Changes

1. Add `let mut guard = MmapGuard::new(ttbr0);` after getting task
2. Add `guard.track(va, phys);` after successful map
3. Add `guard.commit();` before returning success
4. Remove any manual cleanup TODO comments

---

## Acceptance Criteria

- [ ] sys_mmap creates MmapGuard
- [ ] Each successful allocation is tracked
- [ ] Guard is committed only on success
- [ ] Early returns (errors) trigger automatic cleanup
- [ ] Existing behavior unchanged for success path

---

## Testing

```bash
cargo build -p levitate-kernel
cargo xtask test  # Run full test suite
```

---

## Step 2 Complete

After this UoW, Step 2 (mmap Failure Cleanup) is complete. Proceed to Step 3.
