//! TEAM_238: Virtual Memory Area tracking for user processes.
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
        let pos = self
            .vmas
            .iter()
            .position(|v| v.start > vma.start)
            .unwrap_or(self.vmas.len());
        self.vmas.insert(pos, vma);

        Ok(())
    }

    /// Remove VMA(s) covering the given range.
    ///
    /// Handles partial overlaps by splitting VMAs.
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
        self.vmas
            .iter()
            .filter(|v| v.overlaps(start, end))
            .collect()
    }

    /// Iterate over all VMAs.
    pub fn iter(&self) -> impl Iterator<Item = &Vma> {
        self.vmas.iter()
    }

    /// TEAM_239: Update protection flags for VMAs in the given range.
    ///
    /// Updates the flags of VMAs that overlap with [start, end).
    /// If a VMA partially overlaps, it is split and only the overlapping
    /// portion gets the new flags.
    pub fn update_protection(&mut self, start: usize, end: usize, new_flags: VmaFlags) {
        let mut i = 0;

        while i < self.vmas.len() {
            let vma = &self.vmas[i];

            if !vma.overlaps(start, end) {
                i += 1;
                continue;
            }

            let old_flags = vma.flags;
            let vma_start = vma.start;
            let vma_end = vma.end;

            // Remove the existing VMA - we'll re-insert modified version(s)
            self.vmas.remove(i);

            // Case 1: Range covers entire VMA
            if start <= vma_start && end >= vma_end {
                // Re-insert with new flags
                self.vmas.insert(i, Vma::new(vma_start, vma_end, new_flags));
                i += 1;
                continue;
            }

            // Case 2: Range is inside VMA - split into three
            if start > vma_start && end < vma_end {
                // Left portion with old flags
                self.vmas.insert(i, Vma::new(vma_start, start, old_flags));
                i += 1;
                // Middle portion with new flags
                self.vmas.insert(i, Vma::new(start, end, new_flags));
                i += 1;
                // Right portion with old flags
                self.vmas.insert(i, Vma::new(end, vma_end, old_flags));
                i += 1;
                continue;
            }

            // Case 3: Range overlaps left side
            if start <= vma_start && end < vma_end {
                // Left portion (overlapping) with new flags
                self.vmas.insert(i, Vma::new(vma_start, end, new_flags));
                i += 1;
                // Right portion with old flags
                self.vmas.insert(i, Vma::new(end, vma_end, old_flags));
                i += 1;
                continue;
            }

            // Case 4: Range overlaps right side
            if start > vma_start && end >= vma_end {
                // Left portion with old flags
                self.vmas.insert(i, Vma::new(vma_start, start, old_flags));
                i += 1;
                // Right portion (overlapping) with new flags
                self.vmas.insert(i, Vma::new(start, vma_end, new_flags));
                i += 1;
                continue;
            }
        }
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

    #[test]
    fn test_insert_non_overlapping() {
        let mut list = VmaList::new();
        assert!(
            list.insert(Vma::new(0x1000, 0x2000, VmaFlags::READ))
                .is_ok()
        );
        assert!(
            list.insert(Vma::new(0x3000, 0x4000, VmaFlags::READ))
                .is_ok()
        );
        assert!(
            list.insert(Vma::new(0x2000, 0x3000, VmaFlags::READ))
                .is_ok()
        );
    }

    #[test]
    fn test_insert_overlapping_rejected() {
        let mut list = VmaList::new();
        list.insert(Vma::new(0x2000, 0x4000, VmaFlags::READ))
            .unwrap();
        assert!(
            list.insert(Vma::new(0x3000, 0x5000, VmaFlags::READ))
                .is_err()
        );
    }

    #[test]
    fn test_find() {
        let mut list = VmaList::new();
        list.insert(Vma::new(0x1000, 0x2000, VmaFlags::READ))
            .unwrap();
        list.insert(Vma::new(0x3000, 0x4000, VmaFlags::WRITE))
            .unwrap();

        assert!(list.find(0x1500).is_some());
        assert!(list.find(0x2500).is_none());
        assert!(list.find(0x3500).is_some());
    }

    #[test]
    fn test_remove_exact() {
        let mut list = VmaList::new();
        list.insert(Vma::new(0x1000, 0x2000, VmaFlags::READ))
            .unwrap();
        assert!(list.remove(0x1000, 0x2000).is_ok());
        assert!(list.find(0x1500).is_none());
    }

    #[test]
    fn test_remove_split() {
        let mut list = VmaList::new();
        list.insert(Vma::new(0x1000, 0x4000, VmaFlags::READ))
            .unwrap();

        // Remove middle portion
        assert!(list.remove(0x2000, 0x3000).is_ok());

        // Should have left and right portions
        assert!(list.find(0x1500).is_some()); // Left portion
        assert!(list.find(0x2500).is_none()); // Removed
        assert!(list.find(0x3500).is_some()); // Right portion
    }
}
