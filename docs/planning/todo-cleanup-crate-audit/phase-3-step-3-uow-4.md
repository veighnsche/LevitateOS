# UoW 3.4: Update sys_mmap to Record VMAs

**Parent:** Phase 3, Step 3 (VMA Tracking)  
**File:** `kernel/src/syscall/mm.rs`  
**Depends On:** UoW 3.3, UoW 2.2  
**Complexity:** Low

---

## Context

Now that TaskControlBlock has a VmaList, we need to record each successful mmap as a VMA.

---

## Task

After mmap succeeds, create and insert a VMA entry.

---

## Implementation

### Step 1: Add import

At the top of `kernel/src/syscall/mm.rs`:

```rust
use crate::memory::vma::{Vma, VmaFlags};
```

### Step 2: Convert prot to VmaFlags

Add helper function:

```rust
/// TEAM_235: Convert mmap prot flags to VmaFlags.
fn prot_to_vma_flags(prot: i32) -> VmaFlags {
    let mut flags = VmaFlags::empty();
    if prot & PROT_READ != 0 {
        flags |= VmaFlags::READ;
    }
    if prot & PROT_WRITE != 0 {
        flags |= VmaFlags::WRITE;
    }
    if prot & PROT_EXEC != 0 {
        flags |= VmaFlags::EXEC;
    }
    flags
}
```

### Step 3: Record VMA after successful mmap

In `sys_mmap`, after `guard.commit()` but before returning:

```rust
    // TEAM_235: Success - commit the guard (pages kept)
    guard.commit();
    
    // TEAM_235: Record the VMA
    {
        let vma_flags = prot_to_vma_flags(prot);
        let vma = Vma::new(aligned_addr, aligned_addr + aligned_len, vma_flags);
        let mut vmas = task.vmas.lock();
        // Ignore error if overlapping (shouldn't happen with proper mmap)
        let _ = vmas.insert(vma);
    }
    
    aligned_addr as i64
```

Where `aligned_len` is the page-aligned length (calculate from `len`).

---

## Full Change Summary

1. Add imports for `Vma`, `VmaFlags`
2. Add `prot_to_vma_flags()` helper
3. After `guard.commit()`, insert VMA into task's VmaList

---

## Acceptance Criteria

- [ ] `prot_to_vma_flags()` helper works correctly
- [ ] VMA created with correct start/end/flags
- [ ] VMA inserted into task's VmaList
- [ ] Code compiles without errors

---

## Testing

```bash
cargo build -p levitate-kernel
cargo xtask test
```

---

## Next UoW

Proceed to **UoW 3.5** to implement sys_munmap using VMA.
