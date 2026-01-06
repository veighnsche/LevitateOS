// TEAM_051: Slab Allocator - Cache (Per-Size-Class)
// See docs/planning/slab-allocator/phase-2.md behavioral contracts [S1]-[S6]
// TEAM_158: Added behavior ID traceability [SC1]-[SC3]

// TEAM_135: Use shared IntrusiveList instead of slab-local SlabList
use super::super::intrusive_list::IntrusiveList;
use super::page::SlabPage;
use core::ptr::NonNull;

/// Size class configuration.
pub struct SizeClass {
    pub object_size: usize,
    pub objects_per_page: usize,
}

/// [SC1] Size classes optimized for Google Tensor GS101 (64-byte cache lines).
/// 6 size classes: 64B to 2048B
///
/// Uses power-of-two sizes from 64B to 2048B.
/// [SC2] Objects per page = floor(DATA_SIZE / object_size)
pub const SIZE_CLASSES: [SizeClass; 6] = [
    SizeClass {
        object_size: 64,
        objects_per_page: 63,
    }, // 4032/64 = 63
    SizeClass {
        object_size: 128,
        objects_per_page: 31,
    }, // 4032/128 = 31
    SizeClass {
        object_size: 256,
        objects_per_page: 15,
    }, // 4032/256 = 15
    SizeClass {
        object_size: 512,
        objects_per_page: 7,
    }, // 4032/512 = 7
    SizeClass {
        object_size: 1024,
        objects_per_page: 3,
    }, // 4032/1024 = 3
    SizeClass {
        object_size: 2048,
        objects_per_page: 1,
    }, // 4032/2048 = 1
];

/// Per-size-class allocator managing slabs in three lists.
///
/// # Lists
/// - `partial`: Pages with some free objects (allocation target)
/// - `full`: Pages with all objects allocated (skip during alloc)
/// - `empty`: Pages with no objects allocated (can reuse or return to Buddy)
pub struct SlabCache {
    /// Size class index (0-5).
    pub(super) class_index: usize,

    // TEAM_135: Use shared IntrusiveList instead of SlabList
    /// Pages with some free objects (allocation target).
    partial: IntrusiveList<SlabPage>,

    /// Pages with all objects allocated (skip during alloc).
    full: IntrusiveList<SlabPage>,

    /// Pages with no objects allocated (can return to Buddy).
    empty: IntrusiveList<SlabPage>,

    /// Statistics.
    total_allocs: usize,
    total_frees: usize,
}

impl SlabCache {
    /// [SC3] Create a new slab cache for the given size class.
    /// New cache has empty lists.
    // TEAM_135: IntrusiveList::new() is const, so this remains const-compatible
    pub const fn new(class_index: usize) -> Self {
        Self {
            class_index,
            partial: IntrusiveList::new(), // [SC3] empty
            full: IntrusiveList::new(),    // [SC3] empty
            empty: IntrusiveList::new(),   // [SC3] empty
            total_allocs: 0,
            total_frees: 0,
        }
    }

    /// Allocate an object from this cache.
    ///
    /// # Behavior
    /// [S1] Try partial list first
    /// [S2] If empty, try to reclaim from empty list
    /// [S3] If still empty, request new page from BuddyAllocator
    /// Updates page state (partial → full if needed)
    pub fn alloc(&mut self) -> Option<NonNull<u8>> {
        // [S1] Try partial list
        let page = if !self.partial.is_empty() {
            self.partial.head()?
        }
        // [S2] Try empty list (promote to partial)
        else if !self.empty.is_empty() {
            let page_ptr = self.empty.pop_front()?;
            unsafe {
                let page_ref = page_ptr.as_ptr().as_mut().unwrap();
                self.partial.push_front(page_ref);
            }
            page_ptr
        }
        // [S3] Grow cache
        else {
            self.grow()?
        };

        let config = &SIZE_CLASSES[self.class_index];

        // Allocate from page
        unsafe {
            let page_ref = page.as_ptr().as_mut().unwrap();
            let offset = page_ref.alloc_object(config.object_size, config.objects_per_page)?;

            // Update page state if needed
            if page_ref.is_full(config.objects_per_page) {
                self.partial.remove(page_ref);
                self.full.push_front(page_ref);
            }

            let base = page_ref.base_addr();
            self.total_allocs += 1;

            NonNull::new((base + offset) as *mut u8)
        }
    }

    /// Free an object back to this cache.
    ///
    /// # Safety
    /// Caller must ensure ptr was allocated from this cache.
    ///
    /// # Behavior
    /// [S4] Deallocation to partial slab
    /// [S5] Deallocation making page empty
    /// [S6] Deallocation from full slab
    pub unsafe fn free(&mut self, ptr: NonNull<u8>) {
        let config = &SIZE_CLASSES[self.class_index];

        // Compute page from pointer (mask lower 12 bits)
        let page_addr = (ptr.as_ptr() as usize) & !0xFFF;
        // SAFETY: Caller guarantees ptr was allocated from this cache, so page_addr is valid
        let page = unsafe { &mut *(page_addr as *mut SlabPage) };

        // Compute offset within page
        let offset = (ptr.as_ptr() as usize) - page_addr;

        // Check if page was full before freeing
        let was_full = page.is_full(config.objects_per_page);

        // Free object
        page.free_object(offset, config.object_size);
        self.total_frees += 1;

        // Update page state
        if was_full {
            // [S6] Full → Partial
            self.full.remove(page);
            self.partial.push_front(page);
        } else if page.is_empty() {
            // [S5] Partial → Empty
            self.partial.remove(page);
            self.empty.push_front(page);
        }
        // [S4] Remains in partial list
    }

    /// Request a new backing page from BuddyAllocator.
    ///
    /// Returns pointer to the new page, which has been added to partial list.
    fn grow(&mut self) -> Option<NonNull<SlabPage>> {
        use crate::memory::FRAME_ALLOCATOR; // TEAM_051: Now in HAL
        use crate::mmu;

        // Get physical page from buddy allocator (order 0 = 4KB)
        let phys_addr = FRAME_ALLOCATOR.0.lock().alloc(0)?;

        // Convert to virtual address
        let virt_addr = mmu::phys_to_virt(phys_addr);
        let page_ptr = virt_addr as *mut SlabPage;

        // Initialize metadata
        unsafe {
            SlabPage::init(page_ptr as *mut u8, self.class_index as u8, phys_addr);

            // Add to partial list
            let page_ref = &mut *page_ptr;
            self.partial.push_front(page_ref);
        }

        NonNull::new(page_ptr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests: [SC1] 6 size classes 64B-2048B, [SC2] objects per page calculated correctly
    #[test]
    fn test_size_class_constants() {
        use super::super::page::DATA_SIZE; // TEAM_051: Import for tests

        assert_eq!(SIZE_CLASSES.len(), 6); // [SC1] 6 size classes

        // Verify sizes are power-of-two
        for class in &SIZE_CLASSES {
            assert!(class.object_size.is_power_of_two());
        }

        // [SC1] Verify 64B minimum (cache line)
        assert_eq!(SIZE_CLASSES[0].object_size, 64);

        // [SC2] Verify objects per page calculation
        for class in &SIZE_CLASSES {
            let actual = DATA_SIZE / class.object_size;
            assert_eq!(class.objects_per_page, actual); // [SC2]
        }
    }

    /// Tests: [SC3] New cache has empty lists
    #[test]
    fn test_new_cache() {
        let cache = SlabCache::new(0);
        assert_eq!(cache.class_index, 0);
        assert!(cache.partial.is_empty()); // [SC3] empty
        assert!(cache.full.is_empty());    // [SC3] empty
        assert!(cache.empty.is_empty());   // [SC3] empty
        assert_eq!(cache.total_allocs, 0);
        assert_eq!(cache.total_frees, 0);
    }
}

// TEAM_051: SAFETY: SlabCache is safe to Send/Sync because:
// - All pointer operations are protected by global Mutex in SLAB_ALLOCATOR
// - NonNull pointers are only dereferenced under lock protection
unsafe impl Send for SlabCache {}
unsafe impl Sync for SlabCache {}
