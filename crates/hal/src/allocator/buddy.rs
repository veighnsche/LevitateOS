use super::intrusive_list::IntrusiveList;
use super::page::Page;

// TEAM_047: Buddy Allocator implementation
// Handles physical frame allocation and freeing with coalescing.
//
// TEAM_130: This allocator uses expect() in critical paths where failure
// indicates corrupted internal state (invariant violations). Per Rule 14
// (Fail Loud, Fail Fast), panicking is correct for broken invariants.
// TEAM_158: Added behavior ID traceability [B1]-[B11]

pub const MAX_ORDER: usize = 21; // Up to 8GB (2^21 * 4KB)
pub const PAGE_SIZE: usize = 4096;

/// [B1] Buddy Allocator for physical frame management.
/// Allocator starts with empty free lists.
pub struct BuddyAllocator {
    // TEAM_135: Use IntrusiveList instead of raw NonNull pointers
    /// [B1] Free lists for each order (initially empty).
    /// free_lists[i] stores a doubly-linked list of free blocks of order i.
    free_lists: [IntrusiveList<Page>; MAX_ORDER],

    /// Pointer to the global memory map (array of Page structs).
    mem_map: Option<&'static mut [Page]>,

    /// Physical address corresponding to the first entry in mem_map.
    phys_base: usize,
}

// SAFETY: BuddyAllocator is managed via Mutex in the FrameAllocator global
unsafe impl Send for BuddyAllocator {}
unsafe impl Sync for BuddyAllocator {}

impl BuddyAllocator {
    /// Create a new, uninitialized Buddy Allocator.
    // TEAM_135: IntrusiveList::new() is const, so this remains const-compatible
    pub const fn new() -> Self {
        Self {
            free_lists: [const { IntrusiveList::new() }; MAX_ORDER],
            mem_map: None,
            phys_base: 0,
        }
    }

    /// Initialize the allocator with a memory map and physical base address.
    pub unsafe fn init(&mut self, mem_map: &'static mut [Page], phys_base: usize) {
        self.mem_map = Some(mem_map);
        self.phys_base = phys_base;
    }

    /// Add a range of physical memory to the allocator.
    ///
    /// The range must be page-aligned.
    pub unsafe fn add_range(&mut self, start_pa: usize, end_pa: usize) {
        let mut curr_pa = start_pa;
        while curr_pa < end_pa {
            // Find the largest order that fits and is aligned
            let mut order = MAX_ORDER - 1;
            while order > 0 {
                let size = (1 << order) * PAGE_SIZE;
                if curr_pa + size <= end_pa && (curr_pa % size) == 0 {
                    break;
                }
                order -= 1;
            }

            self.free_block(curr_pa, order);
            curr_pa += (1 << order) * PAGE_SIZE;
        }
    }

    /// [B2] Allocate a block of memory of the given order.
    /// [B3] Returns None (OOM) when pool exhausted.
    /// [B4] alloc(order=N) allocates 2^N contiguous pages.
    /// [B6] Block splitting creates buddy pairs.
    // TEAM_135: Refactored to use IntrusiveList API - eliminates unsafe in alloc path
    pub fn alloc(&mut self, order: usize) -> Option<usize> {
        if order >= MAX_ORDER {
            return None;
        }

        // [B2][B4] Find the smallest free block of order >= requested
        for i in order..MAX_ORDER {
            if !self.free_lists[i].is_empty() {
                // Found a block! Pop it from the list.
                let page_ptr = self.free_lists[i].pop_front()
                    .expect("TEAM_135: List was not empty but pop_front failed");
                
                // SAFETY: page_ptr comes from our mem_map via add_to_list
                let page = unsafe { &mut *page_ptr.as_ptr() };

                // [B6] Split the block if it's larger than needed
                for j in (order..i).rev() {
                    let buddy_pa = self.page_to_pa(page) + (1 << j) * PAGE_SIZE;
                    // TEAM_130: Buddy page must exist - this is an invariant of the allocator.
                    let buddy_page = self
                        .pa_to_page_mut(buddy_pa)
                        .expect("TEAM_130: Buddy page must exist - corrupted allocator state");

                    buddy_page.reset();
                    buddy_page.order = j as u8;
                    buddy_page.mark_free();
                    self.add_to_list(j, buddy_page);
                }

                page.mark_allocated();
                page.order = order as u8;
                return Some(self.page_to_pa(page)); // [B2][B7] sequential addresses
            }
        }

        None // [B3] OOM
    }

    /// Free a block of memory.
    pub fn free(&mut self, pa: usize, order: usize) {
        self.free_block(pa, order);
    }

    /// [B8] Free blocks are coalesced with buddies.
    fn free_block(&mut self, pa: usize, order: usize) {
        let mut curr_pa = pa;
        let mut curr_order = order;

        // [B8] Coalesce with buddy if possible
        while curr_order < MAX_ORDER - 1 {
            let buddy_pa = curr_pa ^ ((1 << curr_order) * PAGE_SIZE);

            if let Some(buddy_page) = self.pa_to_page_mut(buddy_pa) {
                // Buddy must be free and have the same order
                if buddy_page.is_free() && buddy_page.order as usize == curr_order {
                    // Pull buddy out of its list
                    self.remove_from_list(curr_order, buddy_page);

                    // [B8] Coalesce
                    if buddy_pa < curr_pa {
                        curr_pa = buddy_pa;
                    }
                    curr_order += 1; // [B8] merged into larger block
                    continue;
                }
            }
            break;
        }

        // Add the (possibly coalesced) block to the free list
        // TEAM_130: Page must exist - caller passed valid PA from prior allocation
        let page = self.pa_to_page_mut(curr_pa).expect("TEAM_130: Page must exist - invalid PA passed to free");
        page.reset();
        page.mark_free();
        page.order = curr_order as u8;
        self.add_to_list(curr_order, page);
    }

    // Helper: Convert Physical Address to Page descriptor
    pub(crate) fn pa_to_page_mut(&mut self, pa: usize) -> Option<&'static mut Page> {
        let mem_map = self.mem_map.as_mut()?;
        let index = (pa - self.phys_base) / PAGE_SIZE;
        if index < mem_map.len() {
            // SAFETY: index is bounds-checked above. mem_map is a valid slice.
            // The returned reference has 'static lifetime as mem_map is 'static.
            unsafe {
                let ptr = mem_map.as_mut_ptr();
                Some(&mut *ptr.add(index))
            }
        } else {
            None
        }
    }

    // Helper: Convert Page descriptor to Physical Address
    fn page_to_pa(&self, page: &Page) -> usize {
        // TEAM_130: mem_map must be set - allocator is unusable if not initialized
        let mem_map = self.mem_map.as_ref().expect("TEAM_130: mem_map must be set - allocator not initialized");
        let offset = page as *const Page as usize - mem_map.as_ptr() as usize;
        let index = offset / core::mem::size_of::<Page>();
        self.phys_base + index * PAGE_SIZE
    }

    // TEAM_135: Simplified using IntrusiveList - no unsafe needed
    fn add_to_list(&mut self, order: usize, page: &'static mut Page) {
        self.free_lists[order].push_front(page);
    }

    // TEAM_135: Simplified using IntrusiveList - no unsafe needed
    fn remove_from_list(&mut self, order: usize, page: &mut Page) {
        self.free_lists[order].remove(page);
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    extern crate std;
    use std::boxed::Box;
    use std::vec;

    fn create_allocator(pages: usize) -> BuddyAllocator {
        let mem_map_storage = vec![Page::new(); pages].into_boxed_slice();
        let mem_map = Box::leak(mem_map_storage);
        let mut allocator = BuddyAllocator::new();
        unsafe {
            allocator.init(mem_map, 0);
            allocator.add_range(0, pages * PAGE_SIZE);
        }
        allocator
    }

    /// Tests: [B1] Allocator starts empty, [B2] alloc(order=0) returns single page, [B3] OOM returns None
    #[test]
    fn test_alloc_order_0() {
        let mut allocator = create_allocator(1); // 1 page

        let addr = allocator.alloc(0); // [B2]
        assert!(addr.is_some());
        assert_eq!(addr.unwrap(), 0); // [B2] single page address

        let addr2 = allocator.alloc(0);
        assert!(addr2.is_none()); // [B3] OOM
    }

    /// Tests: [B4] alloc(order=N) allocates 2^N contiguous pages, [B5] Large allocation consumes entire pool
    #[test]
    fn test_alloc_large() {
        let mut allocator = create_allocator(4); // 4 pages

        // [B4] alloc order 2 (4 pages)
        let addr = allocator.alloc(2);
        assert!(addr.is_some());
        assert_eq!(addr.unwrap(), 0); // [B4] 2^2 = 4 contiguous pages

        // [B5] alloc order 0 should fail - pool consumed
        assert!(allocator.alloc(0).is_none()); // [B5]
    }

    /// Tests: [B6] Block splitting creates buddy pairs, [B7] Sequential allocs get sequential addresses
    #[test]
    fn test_splitting() {
        let mut allocator = create_allocator(4); // 4 pages

        // [B6] Request order 0 (1 page). Should split order 2 -> order 1 -> order 0
        let addr1 = allocator.alloc(0);
        assert!(addr1.is_some());
        assert_eq!(addr1.unwrap(), 0); // [B7] sequential

        // [B6] Remaining: Order 0 (at 4K), Order 1 (at 8K)
        let addr2 = allocator.alloc(0);
        assert!(addr2.is_some());
        assert_eq!(addr2.unwrap(), 4096); // [B7] sequential

        let addr3 = allocator.alloc(1);
        assert!(addr3.is_some());
        assert_eq!(addr3.unwrap(), 8192); // [B7] sequential

        assert!(allocator.alloc(0).is_none());
    }

    /// Tests: [B8] Free blocks are coalesced with buddies, [B9] Coalesced blocks can be reallocated
    #[test]
    fn test_coalescing() {
        let mut allocator = create_allocator(4);

        let addr1 = allocator.alloc(0).unwrap(); // 0
        let addr2 = allocator.alloc(0).unwrap(); // 4096
        let addr3 = allocator.alloc(1).unwrap(); // 8192

        // [B8] Free in reverse order to test coalescing
        allocator.free(addr1, 0);
        allocator.free(addr2, 0);
        // [B8] Should have coalesced into Order 1 at 0

        allocator.free(addr3, 1);
        // [B8] Should have coalesced into Order 2 at 0

        // [B9] Attempt to allocate Order 2 again - coalesced block available
        let addr_big = allocator.alloc(2);
        assert!(addr_big.is_some());
        assert_eq!(addr_big.unwrap(), 0); // [B9] reallocated
    }

    /// Tests: [B10] Non-power-of-two ranges are handled, [B11] Leftover pages added to appropriate order
    #[test]
    fn test_alloc_unaligned_range() {
        // [B10] Test adding a range that isn't power-of-two aligned
        // 5 pages. 0-16K (Order 2), 16K-20K (Order 0)
        let mut allocator = create_allocator(5);

        let addr1 = allocator.alloc(2); // Should get the 0-16K block
        assert!(addr1.is_some());

        // [B11] The remaining 4KB (page 4) should be free as Order 0
        let addr2 = allocator.alloc(0);
        assert!(addr2.is_some());
        assert_eq!(addr2.unwrap(), 16384); // [B11] leftover page at correct address
    }
}
