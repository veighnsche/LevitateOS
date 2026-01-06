// TEAM_051: Slab Allocator - Top Level Module
// See docs/planning/slab-allocator/phase-2.md for design
// TEAM_158: Added behavior ID traceability [SA1]-[SA4]

mod cache;
// TEAM_135: Removed mod list - now using shared intrusive_list module
mod page;

pub use cache::{SIZE_CLASSES, SlabCache};
pub use page::{DATA_SIZE, PAGE_SIZE, SlabPage};

use core::alloc::Layout;
use core::ptr::NonNull;
use los_utils::Mutex;

/// [SA1] Top-level slab allocator managing all size classes.
/// Initializes 6 caches (one per size class).
///
/// Routes allocation requests to the appropriate size class cache.
/// Thread-safety is provided by a global Mutex.
pub struct SlabAllocator {
    caches: [SlabCache; 6], // [SA1] 6 caches
}

impl SlabAllocator {
    /// [SA1] Create a new slab allocator with all size classes.
    /// Initializes 6 caches.
    pub const fn new() -> Self {
        Self {
            caches: [ // [SA1] 6 caches initialized
                SlabCache::new(0),
                SlabCache::new(1),
                SlabCache::new(2),
                SlabCache::new(3),
                SlabCache::new(4),
                SlabCache::new(5),
            ],
        }
    }

    /// Allocate memory for the given layout.
    ///
    /// # Returns
    /// - `Some(ptr)` if allocation succeeds
    /// - `None` if:
    ///   - Size is 0
    ///   - Size > 2048 (use BuddyAllocator directly)
    ///   - Alignment > object_size (unsupported)
    ///   - OOM (BuddyAllocator exhausted)
    pub fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        // Reject invalid or unsupported requests
        if layout.size() == 0 {
            return None;
        }

        let class = Self::size_to_class(layout.size())?;
        let config = &SIZE_CLASSES[class];

        // Check alignment requirement
        if layout.align() > config.object_size {
            return None;
        }

        self.caches[class].alloc()
    }

    /// Free memory previously allocated with the same layout.
    ///
    /// # Safety
    /// - `ptr` must have been allocated by this allocator
    /// - `layout` must be the same as used for allocation
    pub unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if let Some(class) = Self::size_to_class(layout.size()) {
            // SAFETY: Caller guarantees ptr was allocated with the same layout
            unsafe {
                self.caches[class].free(ptr);
            }
        }
    }

    /// [SA2] Map allocation size to size class index.
    /// [SA3] Returns None for size 0 (invalid).
    /// [SA4] Returns None for size > 2048.
    ///
    /// Returns None for size 0 or size > 2048.
    fn size_to_class(size: usize) -> Option<usize> {
        match size {
            0 => None,           // [SA3] invalid size
            1..=64 => Some(0),   // [SA2] maps correctly
            65..=128 => Some(1),
            129..=256 => Some(2),
            257..=512 => Some(3),
            513..=1024 => Some(4),
            1025..=2048 => Some(5),
            _ => None,           // [SA4] size > 2048
        }
    }
}

/// Global slab allocator instance.
///
/// Protected by Mutex for thread-safety.
///
/// # Usage
/// ```rust
/// use los_hal::allocator::slab::SLAB_ALLOCATOR;
/// use core::alloc::Layout;
///
/// let layout = Layout::from_size_align(128, 8).unwrap();
/// let ptr = SLAB_ALLOCATOR.lock().alloc(layout);
/// ```
pub static SLAB_ALLOCATOR: Mutex<SlabAllocator> = Mutex::new(SlabAllocator::new());

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests: [SA2] size_to_class maps sizes correctly
    #[test]
    fn test_size_to_class_mapping() {
        // [SA2] Class 0: 1-64 bytes
        assert_eq!(SlabAllocator::size_to_class(1), Some(0));
        assert_eq!(SlabAllocator::size_to_class(64), Some(0));

        // [SA2] Class 1: 65-128 bytes
        assert_eq!(SlabAllocator::size_to_class(65), Some(1));
        assert_eq!(SlabAllocator::size_to_class(128), Some(1));

        // [SA2] Class 2: 129-256 bytes
        assert_eq!(SlabAllocator::size_to_class(129), Some(2));
        assert_eq!(SlabAllocator::size_to_class(256), Some(2));

        // [SA2] Class 3: 257-512 bytes
        assert_eq!(SlabAllocator::size_to_class(257), Some(3));
        assert_eq!(SlabAllocator::size_to_class(512), Some(3));

        // [SA2] Class 4: 513-1024 bytes
        assert_eq!(SlabAllocator::size_to_class(513), Some(4));
        assert_eq!(SlabAllocator::size_to_class(1024), Some(4));

        // [SA2] Class 5: 1025-2048 bytes
        assert_eq!(SlabAllocator::size_to_class(1025), Some(5));
        assert_eq!(SlabAllocator::size_to_class(2048), Some(5));

        // Out of range
        assert_eq!(SlabAllocator::size_to_class(0), None);
        assert_eq!(SlabAllocator::size_to_class(2049), None);
    }

    /// Tests: [SA1] New allocator initializes 6 caches
    #[test]
    fn test_new_allocator() {
        let allocator = SlabAllocator::new();

        // [SA1] Verify all 6 caches are initialized
        for (i, cache) in allocator.caches.iter().enumerate() {
            assert_eq!(cache.class_index, i); // [SA1]
        }
    }

    /// Tests: [SA3] Returns None for size 0, [SA4] Returns None for size > 2048
    #[test]
    fn test_invalid_allocation_requests() {
        let mut allocator = SlabAllocator::new();

        // [SA3] Size 0
        let layout = Layout::from_size_align(0, 1).unwrap();
        assert!(allocator.alloc(layout).is_none()); // [SA3]

        // [SA4] Size too large
        let layout = Layout::from_size_align(4096, 8).unwrap();
        assert!(allocator.alloc(layout).is_none()); // [SA4]
    }
}
