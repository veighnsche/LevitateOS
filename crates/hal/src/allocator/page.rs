use bitflags::bitflags;
use core::ptr::NonNull;

use super::intrusive_list::ListNode;

// TEAM_047: Page descriptor for physical frame tracking
// Part of Buddy Allocator implementation (Phase 5)

bitflags! {
    /// Flags for a physical page frame.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PhysPageFlags: u8 {
        /// Page is currently allocated
        const ALLOCATED = 1 << 0;
        /// Page is the head of a multi-page allocation
        const HEAD      = 1 << 1;
        /// Page is a tail (non-head) of a multi-page allocation
        const TAIL      = 1 << 2;
        /// Page is free and managed by buddy allocator
        const FREE      = 1 << 3;
    }
}

/// A descriptor for a physical page frame.
///
/// Each physical frame has one `Page` descriptor in a global array.
/// This allows tracking ownership and buddy state without touching
/// the physical memory itself (safe for device memory or uncached regions).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Page {
    pub flags: PhysPageFlags,
    /// Order of allocation (only valid if HEAD is set)
    pub order: u8,
    /// Reference count (for future CoW/shared memory support)
    pub refcount: u16,
    /// Intrusive list pointers for the buddy allocator's free lists.
    /// Only used when PageFlags::FREE is set.
    pub next: Option<NonNull<Page>>,
    pub prev: Option<NonNull<Page>>,
}

// SAFETY: Page is just a descriptor, we manage access via Mutex in BuddyAllocator
unsafe impl Send for Page {}
unsafe impl Sync for Page {}

impl Page {
    /// Create a new, zeroed page descriptor.
    pub const fn new() -> Self {
        Self {
            flags: PhysPageFlags::empty(),
            order: 0,
            refcount: 0,
            next: None,
            prev: None,
        }
    }

    /// Reset page state to defaults.
    pub fn reset(&mut self) {
        self.flags = PhysPageFlags::empty();
        self.order = 0;
        self.refcount = 0;
        self.next = None;
        self.prev = None;
    }

    /// Check if page is currently free.
    pub fn is_free(&self) -> bool {
        self.flags.contains(PhysPageFlags::FREE)
    }

    /// Mark page as free.
    pub fn mark_free(&mut self) {
        self.flags.insert(PhysPageFlags::FREE);
        self.flags.remove(PhysPageFlags::ALLOCATED);
    }

    /// Mark page as allocated.
    pub fn mark_allocated(&mut self) {
        self.flags.insert(PhysPageFlags::ALLOCATED);
        self.flags.remove(PhysPageFlags::FREE);
    }
}

// TEAM_135: Implement ListNode trait for Page to enable use with IntrusiveList
impl ListNode for Page {
    #[inline]
    fn next(&self) -> Option<NonNull<Self>> {
        self.next
    }

    #[inline]
    fn prev(&self) -> Option<NonNull<Self>> {
        self.prev
    }

    #[inline]
    fn set_next(&mut self, next: Option<NonNull<Self>>) {
        self.next = next;
    }

    #[inline]
    fn set_prev(&mut self, prev: Option<NonNull<Self>>) {
        self.prev = prev;
    }
}
