# UoW 3.2: Create VmaList Container

**Parent:** Phase 3, Step 3 (VMA Tracking)  
**File:** `kernel/src/memory/vma.rs`  
**Depends On:** UoW 3.1  
**Complexity:** Medium

---

## Context

UoW 3.1 created the `Vma` type. Now we need a container to manage multiple VMAs for a process.

---

## Task

Add `VmaList` and `VmaError` to `kernel/src/memory/vma.rs`.

---

## Implementation

Add to `kernel/src/memory/vma.rs` after the `Vma` impl:

```rust
use los_error::define_kernel_error;

define_kernel_error! {
    /// VMA operation errors.
    pub enum VmaError(0x05) {
        /// Attempted to insert overlapping VMA
        Overlapping = 0x01 => "VMA overlaps existing region",
        /// VMA not found
        NotFound = 0x02 => "VMA not found",
    }
}

/// List of VMAs for a process.
///
/// Maintains a sorted list of non-overlapping VMAs.
#[derive(Debug, Default)]
pub struct VmaList {
    /// VMAs sorted by start address
    vmas: Vec<Vma>,
}

impl VmaList {
    /// Create an empty VMA list.
    #[must_use]
    pub fn new() -> Self {
        Self { vmas: Vec::new() }
    }
    
    /// Insert a new VMA. Returns error if it overlaps existing.
    pub fn insert(&mut self, vma: Vma) -> Result<(), VmaError> {
        // Check for overlaps
        for existing in &self.vmas {
            if existing.overlaps(vma.start, vma.end) {
                return Err(VmaError::Overlapping);
            }
        }
        
        // Insert maintaining sorted order
        let pos = self.vmas
            .iter()
            .position(|v| v.start > vma.start)
            .unwrap_or(self.vmas.len());
        self.vmas.insert(pos, vma);
        
        Ok(())
    }
    
    /// Remove VMA(s) covering the given range.
    ///
    /// Handles partial overlaps by splitting VMAs.
    /// Returns the removed/modified VMAs.
    pub fn remove(&mut self, start: usize, end: usize) -> Result<(), VmaError> {
        let mut i = 0;
        let mut found_any = false;
        
        while i < self.vmas.len() {
            let vma = &self.vmas[i];
            
            if !vma.overlaps(start, end) {
                i += 1;
                continue;
            }
            
            found_any = true;
            let vma = self.vmas.remove(i);
            
            // Case 1: Range covers entire VMA - just remove
            if start <= vma.start && end >= vma.end {
                // Already removed, continue
                continue;
            }
            
            // Case 2: Range is inside VMA - split into two
            if start > vma.start && end < vma.end {
                // Left portion
                self.vmas.insert(i, Vma::new(vma.start, start, vma.flags));
                i += 1;
                // Right portion
                self.vmas.insert(i, Vma::new(end, vma.end, vma.flags));
                i += 1;
                continue;
            }
            
            // Case 3: Range overlaps left side
            if start <= vma.start && end < vma.end {
                self.vmas.insert(i, Vma::new(end, vma.end, vma.flags));
                i += 1;
                continue;
            }
            
            // Case 4: Range overlaps right side
            if start > vma.start && end >= vma.end {
                self.vmas.insert(i, Vma::new(vma.start, start, vma.flags));
                i += 1;
                continue;
            }
        }
        
        if found_any {
            Ok(())
        } else {
            Err(VmaError::NotFound)
        }
    }
    
    /// Find VMA containing the given address.
    #[must_use]
    pub fn find(&self, addr: usize) -> Option<&Vma> {
        self.vmas.iter().find(|v| v.contains(addr))
    }
    
    /// Find all VMAs overlapping the given range.
    pub fn find_overlapping(&self, start: usize, end: usize) -> Vec<&Vma> {
        self.vmas.iter().filter(|v| v.overlaps(start, end)).collect()
    }
    
    /// Iterate over all VMAs.
    pub fn iter(&self) -> impl Iterator<Item = &Vma> {
        self.vmas.iter()
    }
}

#[cfg(test)]
mod vma_list_tests {
    use super::*;
    
    #[test]
    fn test_insert_non_overlapping() {
        let mut list = VmaList::new();
        assert!(list.insert(Vma::new(0x1000, 0x2000, VmaFlags::READ)).is_ok());
        assert!(list.insert(Vma::new(0x3000, 0x4000, VmaFlags::READ)).is_ok());
        assert!(list.insert(Vma::new(0x2000, 0x3000, VmaFlags::READ)).is_ok());
    }
    
    #[test]
    fn test_insert_overlapping_rejected() {
        let mut list = VmaList::new();
        list.insert(Vma::new(0x2000, 0x4000, VmaFlags::READ)).unwrap();
        assert!(list.insert(Vma::new(0x3000, 0x5000, VmaFlags::READ)).is_err());
    }
    
    #[test]
    fn test_find() {
        let mut list = VmaList::new();
        list.insert(Vma::new(0x1000, 0x2000, VmaFlags::READ)).unwrap();
        list.insert(Vma::new(0x3000, 0x4000, VmaFlags::WRITE)).unwrap();
        
        assert!(list.find(0x1500).is_some());
        assert!(list.find(0x2500).is_none());
        assert!(list.find(0x3500).is_some());
    }
    
    #[test]
    fn test_remove_exact() {
        let mut list = VmaList::new();
        list.insert(Vma::new(0x1000, 0x2000, VmaFlags::READ)).unwrap();
        assert!(list.remove(0x1000, 0x2000).is_ok());
        assert!(list.find(0x1500).is_none());
    }
    
    #[test]
    fn test_remove_split() {
        let mut list = VmaList::new();
        list.insert(Vma::new(0x1000, 0x4000, VmaFlags::READ)).unwrap();
        
        // Remove middle portion
        assert!(list.remove(0x2000, 0x3000).is_ok());
        
        // Should have left and right portions
        assert!(list.find(0x1500).is_some()); // Left portion
        assert!(list.find(0x2500).is_none()); // Removed
        assert!(list.find(0x3500).is_some()); // Right portion
    }
}
```

---

## Acceptance Criteria

- [ ] `VmaError` enum defined
- [ ] `VmaList` struct with Vec<Vma>
- [ ] `insert()` rejects overlapping VMAs
- [ ] `remove()` handles all cases (exact, split, partial)
- [ ] `find()` returns correct VMA
- [ ] All unit tests pass

---

## Testing

```bash
cargo test -p levitate-kernel vma
```

---

## Next UoW

Proceed to **UoW 3.3** to add VmaList to TaskControlBlock.
