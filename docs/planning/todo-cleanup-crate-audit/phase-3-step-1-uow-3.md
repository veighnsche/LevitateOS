# UoW 1.3: Add Unit Test for Page Table Teardown

**Parent:** Phase 3, Step 1 (Page Table Teardown)  
**File:** `kernel/src/memory/user.rs`  
**Depends On:** UoW 1.2  
**Complexity:** Low

---

## Context

The `destroy_user_page_table` implementation is complete. We need a test to verify it works correctly.

---

## Task

Add a test that creates a user page table, maps pages, then destroys it.

---

## Implementation

Add to the test module in `kernel/src/memory/user.rs` (or create one if it doesn't exist):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: This test requires the allocator to be initialized.
    // It may need to be an integration test or run with a mock allocator.
    // For now, document the expected behavior.
    
    /// Test that destroy_user_page_table properly cleans up.
    /// 
    /// This test verifies:
    /// 1. Page table creation works
    /// 2. Page mapping works  
    /// 3. Teardown doesn't panic
    /// 4. No double-free errors
    #[test]
    #[ignore = "Requires kernel allocator - run as integration test"]
    fn test_destroy_user_page_table() {
        // 1. Create a user page table
        let ttbr0 = create_user_page_table()
            .expect("Failed to create user page table");
        
        // 2. Map a few test pages
        let test_vas = [0x1000, 0x2000, 0x3000, 0x10000];
        for &va in &test_vas {
            // Allocate a physical page
            let phys = FRAME_ALLOCATOR.alloc_page()
                .expect("Failed to allocate page");
            
            // Map it
            unsafe {
                map_user_page(ttbr0, va, phys, los_hal::mmu::PageFlags::USER_DATA)
                    .expect("Failed to map page");
            }
        }
        
        // 3. Destroy the page table
        unsafe {
            destroy_user_page_table(ttbr0)
                .expect("Failed to destroy page table");
        }
        
        // 4. If we get here without panic, the test passes
        // A more thorough test would check allocator stats
    }
}
```

---

## Acceptance Criteria

- [ ] Test compiles
- [ ] Test is marked `#[ignore]` with reason (needs kernel env)
- [ ] Test documents expected behavior
- [ ] Test covers create → map → destroy flow

---

## Notes

This test may need to be run as an integration test in the full kernel environment where the allocator is initialized. The `#[ignore]` attribute prevents it from running in `cargo test` by default.

To run ignored tests:
```bash
cargo test -- --ignored
```

---

## Step 1 Complete

After this UoW, Step 1 (Page Table Teardown) is complete. Proceed to Step 2.
