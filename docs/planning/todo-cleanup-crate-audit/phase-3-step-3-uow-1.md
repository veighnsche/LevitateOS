# UoW 3.1: Create VMA Types

**Parent:** Phase 3, Step 3 (VMA Tracking)  
**File:** `kernel/src/memory/vma.rs` (NEW FILE)  
**Complexity:** Low

---

## Context

To implement proper `munmap`, we need to track which virtual memory regions are mapped. This UoW creates the basic VMA data structures.

---

## Task

Create a new file `kernel/src/memory/vma.rs` with VMA types.

---

## Implementation

**Create new file:** `kernel/src/memory/vma.rs`

```rust
//! TEAM_235: Virtual Memory Area tracking for user processes.
//!
//! Tracks mapped regions to enable proper munmap and mprotect.

extern crate alloc;

use alloc::vec::Vec;
use bitflags::bitflags;

bitflags! {
    /// VMA permission flags (matches mmap prot flags).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct VmaFlags: u32 {
        /// Region is readable
        const READ = 1 << 0;
        /// Region is writable
        const WRITE = 1 << 1;
        /// Region is executable
        const EXEC = 1 << 2;
    }
}

/// A contiguous virtual memory region.
#[derive(Debug, Clone)]
pub struct Vma {
    /// Start address (page-aligned)
    pub start: usize,
    /// End address (exclusive, page-aligned)
    pub end: usize,
    /// Permission flags
    pub flags: VmaFlags,
}

impl Vma {
    /// Create a new VMA.
    #[must_use]
    pub fn new(start: usize, end: usize, flags: VmaFlags) -> Self {
        debug_assert!(start < end, "VMA start must be < end");
        debug_assert!(start & 0xFFF == 0, "VMA start must be page-aligned");
        debug_assert!(end & 0xFFF == 0, "VMA end must be page-aligned");
        Self { start, end, flags }
    }
    
    /// Length of the VMA in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    
    /// Check if VMA is empty (shouldn't happen in practice).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
    
    /// Check if address is within this VMA.
    #[must_use]
    pub fn contains(&self, addr: usize) -> bool {
        addr >= self.start && addr < self.end
    }
    
    /// Check if this VMA overlaps with a range.
    #[must_use]
    pub fn overlaps(&self, start: usize, end: usize) -> bool {
        self.start < end && start < self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vma_contains() {
        let vma = Vma::new(0x1000, 0x3000, VmaFlags::READ);
        assert!(vma.contains(0x1000));
        assert!(vma.contains(0x2FFF));
        assert!(!vma.contains(0x0FFF));
        assert!(!vma.contains(0x3000));
    }
    
    #[test]
    fn test_vma_overlaps() {
        let vma = Vma::new(0x2000, 0x4000, VmaFlags::READ);
        
        // Overlapping cases
        assert!(vma.overlaps(0x1000, 0x3000)); // Partial left
        assert!(vma.overlaps(0x3000, 0x5000)); // Partial right
        assert!(vma.overlaps(0x2500, 0x3500)); // Inside
        assert!(vma.overlaps(0x1000, 0x5000)); // Contains
        
        // Non-overlapping
        assert!(!vma.overlaps(0x0000, 0x2000)); // Before
        assert!(!vma.overlaps(0x4000, 0x5000)); // After
    }
    
    #[test]
    fn test_vma_len() {
        let vma = Vma::new(0x1000, 0x5000, VmaFlags::empty());
        assert_eq!(vma.len(), 0x4000);
    }
}
```

---

## Also Required

Add the module to `kernel/src/memory/mod.rs`:

```rust
pub mod vma; // TEAM_235: VMA tracking
```

---

## Acceptance Criteria

- [ ] File `kernel/src/memory/vma.rs` exists
- [ ] `VmaFlags` has READ, WRITE, EXEC
- [ ] `Vma` struct has start, end, flags
- [ ] `new()`, `len()`, `contains()`, `overlaps()` implemented
- [ ] Unit tests pass
- [ ] Module exported from `memory/mod.rs`

---

## Testing

```bash
cargo test -p levitate-kernel vma
```

---

## Next UoW

Proceed to **UoW 3.2** to create the VmaList container.
