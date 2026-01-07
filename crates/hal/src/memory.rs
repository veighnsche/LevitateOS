// TEAM_051: Global frame allocator for HAL
// Moved from kernel/src/memory/mod.rs to enable slab allocator integration
// See docs/planning/slab-allocator/phase-3.md

use crate::allocator::BuddyAllocator;
use crate::traits::PageAllocator;
use los_utils::Mutex;

/// Global Frame Allocator wrapper around BuddyAllocator
pub struct FrameAllocator(pub Mutex<BuddyAllocator>);

// TEAM_051: SAFETY: Protected by Mutex, all access is synchronized
unsafe impl Send for FrameAllocator {}
unsafe impl Sync for FrameAllocator {}

impl PageAllocator for FrameAllocator {
    fn alloc_page(&self) -> Option<usize> {
        self.0.lock().alloc(0)
    }

    fn free_page(&self, pa: usize) {
        self.0.lock().free(pa, 0)
    }
}

/// Global frame allocator instance
pub static FRAME_ALLOCATOR: FrameAllocator = FrameAllocator(Mutex::new(BuddyAllocator::new()));
