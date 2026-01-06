//! TEAM_166: Global Allocator for LevitateOS userspace.
//!
//! Implements a simple bump allocator backed by the `sbrk` syscall.
//! Per Phase 2 decisions:
//! - Q1: Start with 0, grow by 4KB on first allocation
//! - Q3: Return null on OOM (allocator will panic)

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::null_mut;

/// TEAM_166: Page size for heap growth (4KB).
const PAGE_SIZE: usize = 4096;

/// TEAM_166: Simple bump allocator backed by sbrk.
///
/// This allocator grows the heap on demand using the sbrk syscall.
/// It uses a simple bump allocation strategy - allocation is O(1),
/// but deallocation is a no-op (memory is not reclaimed until process exit).
///
/// For more sophisticated memory management, consider using
/// `linked_list_allocator` or `dlmalloc` in the future.
pub struct LosAllocator {
    /// Current allocation pointer (bumps upward)
    head: UnsafeCell<usize>,
    /// End of currently allocated heap
    end: UnsafeCell<usize>,
}

impl LosAllocator {
    /// TEAM_166: Create a new allocator.
    ///
    /// The allocator starts with no heap - it will call sbrk on first allocation.
    pub const fn new() -> Self {
        Self {
            head: UnsafeCell::new(0),
            end: UnsafeCell::new(0),
        }
    }

    /// TEAM_166: Grow the heap by at least `min_size` bytes.
    ///
    /// Rounds up to page boundaries and calls sbrk.
    ///
    /// # Safety
    /// Must be called with proper synchronization (single-threaded or locked).
    unsafe fn grow(&self, min_size: usize) -> bool {
        // TEAM_181: Use checked arithmetic to prevent overflow
        let pages_needed = match min_size.checked_add(PAGE_SIZE - 1) {
            Some(n) => n / PAGE_SIZE,
            None => return false, // Overflow, can't allocate
        };
        let grow_size = match pages_needed.checked_mul(PAGE_SIZE) {
            Some(n) => n,
            None => return false, // Overflow
        };

        // Call sbrk to grow heap
        let old_break = libsyscall::sbrk(grow_size as isize);

        if old_break == 0 {
            // sbrk failed (OOM)
            return false;
        }

        // Update our tracking
        let head = unsafe { &mut *self.head.get() };
        let end = unsafe { &mut *self.end.get() };

        if *head == 0 {
            // First allocation - initialize head
            *head = old_break as usize;
        }

        *end = old_break as usize + grow_size;
        true
    }
}

// SAFETY: LosAllocator uses UnsafeCell but is only safe in single-threaded context.
// LevitateOS userspace is currently single-threaded per process.
unsafe impl Sync for LosAllocator {}

unsafe impl GlobalAlloc for LosAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // TEAM_181: Handle zero-size allocations per GlobalAlloc contract.
        // Zero-size allocations must return a non-null, well-aligned dangling pointer.
        if layout.size() == 0 {
            return layout.align() as *mut u8;
        }

        let head = unsafe { &mut *self.head.get() };
        let end = unsafe { &mut *self.end.get() };

        // Align the head pointer
        let align = layout.align();
        // TEAM_181: Use checked arithmetic to prevent overflow
        let aligned = match (*head).checked_add(align - 1) {
            Some(n) => n & !(align - 1),
            None => return null_mut(), // Overflow
        };
        let new_head = match aligned.checked_add(layout.size()) {
            Some(n) => n,
            None => return null_mut(), // Overflow
        };

        // Check if we need to grow
        if new_head > *end {
            let needed = new_head.saturating_sub(*end);
            // SAFETY: We are in an unsafe fn, grow is safe to call here
            if unsafe { !self.grow(needed) } {
                return null_mut(); // OOM - per Q3 decision
            }
            // TEAM_174: After grow(), head may have been initialized from 0 to old_break.
            // Re-compute aligned pointer with the updated head value.
            let aligned = match (*head).checked_add(align - 1) {
                Some(n) => n & !(align - 1),
                None => return null_mut(),
            };
            let new_head = match aligned.checked_add(layout.size()) {
                Some(n) => n,
                None => return null_mut(),
            };

            // TEAM_181: Verify we have enough space after grow
            if new_head > *end {
                return null_mut(); // Still not enough space
            }

            // Bump allocate
            let ptr = aligned as *mut u8;
            *head = new_head;
            return ptr;
        }

        // Bump allocate
        let ptr = aligned as *mut u8;
        *head = new_head;
        ptr
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // TEAM_166: Bump allocator does not reclaim memory.
        // Memory is reclaimed when the process exits.
        // A more sophisticated allocator (linked-list or slab) would
        // track free blocks here.
    }
}

/// TEAM_166: Global allocator instance.
///
/// Add this to your binary crate to enable heap allocation:
/// ```rust
/// #[global_allocator]
/// static ALLOCATOR: ulib::LosAllocator = ulib::LosAllocator::new();
/// ```
#[global_allocator]
pub static ALLOCATOR: LosAllocator = LosAllocator::new();

// TEAM_166: Allocation error handler.
// Required when using alloc crate in no_std.
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    libsyscall::write(2, b"ALLOC ERROR: out of memory\n");
    let _ = layout; // Suppress unused warning
    libsyscall::exit(1);
}
