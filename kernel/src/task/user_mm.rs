//! TEAM_073: User Address Space Management for LevitateOS.
//!
//! This module provides:
//! - Per-process TTBR0 page table creation
//! - User memory mapping functions
//! - Address space layout for user processes

use crate::memory::FRAME_ALLOCATOR;
use levitate_hal::mmu::{self, PAGE_SIZE, PageAllocator, PageFlags, PageTable};

/// TEAM_073: User address space layout constants.
pub mod layout {
    /// User stack top (grows down from here)
    /// Max user address for 48-bit VA with TTBR0
    pub const STACK_TOP: usize = 0x0000_7FFF_FFFF_0000;

    /// User stack size (64KB default)
    pub const STACK_SIZE: usize = 65536;

    /// End of user address space (bit 47 clear = TTBR0)
    pub const USER_SPACE_END: usize = 0x0000_8000_0000_0000;
}

/// TEAM_073: Create a new user page table.
///
/// Allocates an L0 page table for a user process's TTBR0.
/// The table is initially empty - caller must map user segments.
///
/// # Returns
/// Physical address of the new L0 page table, or None if allocation fails.
pub fn create_user_page_table() -> Option<usize> {
    levitate_hal::println!("[MMU] Creating user page table...");
    // Allocate a page for L0 table
    let l0_phys = FRAME_ALLOCATOR.alloc_page()?;

    levitate_hal::println!("[MMU] Allocated L0 table at phys [MASKED]");

    // Zero the table
    let l0_va = mmu::phys_to_virt(l0_phys);
    levitate_hal::println!("[MMU] Zeroing L0 table at va [MASKED]");
    let l0 = unsafe { &mut *(l0_va as *mut PageTable) };
    l0.zero();

    // Return physical address for TTBR0
    Some(l0_phys)
}

/// TEAM_073: Map a single user page.
///
/// Maps a page in the user's TTBR0 page table.
///
/// # Arguments
/// * `ttbr0_phys` - Physical address of user L0 page table
/// * `user_va` - Virtual address in user space (must be < 0x8000_0000_0000)
/// * `phys` - Physical address to map
/// * `flags` - Page flags (should use USER_CODE or USER_DATA)
///
/// # Safety
/// - `ttbr0_phys` must point to a valid L0 page table
/// - `user_va` must be in valid user address range
pub unsafe fn map_user_page(
    ttbr0_phys: usize,
    user_va: usize,
    phys: usize,
    flags: PageFlags,
) -> Result<(), &'static str> {
    // Validate user address
    if user_va >= layout::USER_SPACE_END {
        return Err("Virtual address not in user space");
    }

    // Get the L0 table
    let l0_va = mmu::phys_to_virt(ttbr0_phys);
    let l0 = unsafe { &mut *(l0_va as *mut PageTable) };

    // Use MMU's map_page function
    mmu::map_page(l0, user_va, phys, flags)
}

/// TEAM_073: Map a range of user pages.
///
/// # Arguments
/// * `ttbr0_phys` - Physical address of user L0 page table
/// * `user_va_start` - Starting virtual address (page-aligned)
/// * `phys_start` - Starting physical address (page-aligned)
/// * `len` - Length in bytes to map
/// * `flags` - Page flags
#[allow(dead_code)]
pub unsafe fn map_user_range(
    ttbr0_phys: usize,
    user_va_start: usize,
    phys_start: usize,
    len: usize,
    flags: PageFlags,
) -> Result<(), &'static str> {
    // Validate user address
    if user_va_start >= layout::USER_SPACE_END {
        return Err("Virtual address not in user space");
    }
    if user_va_start.saturating_add(len) > layout::USER_SPACE_END {
        return Err("Mapping extends beyond user space");
    }

    let l0_va = mmu::phys_to_virt(ttbr0_phys);
    let l0 = unsafe { &mut *(l0_va as *mut PageTable) };

    let mut va = user_va_start & !0xFFF; // Page align
    let mut pa = phys_start & !0xFFF;
    let end_va = (user_va_start + len + 0xFFF) & !0xFFF;

    while va < end_va {
        mmu::map_page(l0, va, pa, flags)?;
        va += PAGE_SIZE;
        pa += PAGE_SIZE;
    }

    Ok(())
}

/// TEAM_073: Allocate and map user stack pages.
///
/// Allocates physical pages for the user stack and maps them at the
/// standard stack location.
///
/// # Arguments
/// * `ttbr0_phys` - Physical address of user L0 page table
/// * `stack_pages` - Number of stack pages (e.g., 16 for 64KB)
///
/// # Returns
/// Initial stack pointer (top of stack) on success.
pub unsafe fn setup_user_stack(
    ttbr0_phys: usize,
    stack_pages: usize,
) -> Result<usize, &'static str> {
    let stack_size = stack_pages * PAGE_SIZE;
    let stack_bottom = layout::STACK_TOP - stack_size;

    // Allocate physical pages for stack
    for i in 0..stack_pages {
        let page_va = stack_bottom + i * PAGE_SIZE;

        // Allocate physical page
        let phys = FRAME_ALLOCATOR
            .alloc_page()
            .ok_or("Failed to allocate stack page")?;

        // Zero the stack page for security
        let page_ptr = mmu::phys_to_virt(phys) as *mut u8;
        unsafe {
            core::ptr::write_bytes(page_ptr, 0, PAGE_SIZE);
        }

        // Map into user address space
        unsafe {
            map_user_page(ttbr0_phys, page_va, phys, PageFlags::USER_STACK)?;
        }
    }

    // Return stack pointer (top of stack, stack grows down)
    Ok(layout::STACK_TOP)
}

/// TEAM_073: Allocate physical pages and map them for user code/data.
#[allow(dead_code)]
pub unsafe fn alloc_and_map_user_range(
    ttbr0_phys: usize,
    user_va_start: usize,
    len: usize,
    flags: PageFlags,
) -> Result<usize, &'static str> {
    if len == 0 {
        return Err("Cannot allocate zero bytes");
    }

    let va_start = user_va_start & !0xFFF;
    let pages_needed = (len + (user_va_start - va_start) + PAGE_SIZE - 1) / PAGE_SIZE;

    let mut first_phys = 0;

    for i in 0..pages_needed {
        let page_va = va_start + i * PAGE_SIZE;

        // Allocate physical page
        let phys = FRAME_ALLOCATOR
            .alloc_page()
            .ok_or("Failed to allocate user page")?;

        if i == 0 {
            first_phys = phys;
        }

        // Zero the page
        let page_ptr = mmu::phys_to_virt(phys) as *mut u8;
        unsafe {
            core::ptr::write_bytes(page_ptr, 0, PAGE_SIZE);
        }

        // Map into user address space
        unsafe {
            map_user_page(ttbr0_phys, page_va, phys, flags)?;
        }
    }

    Ok(first_phys)
}

/// TEAM_073: Free a user page table and all its mappings.
#[allow(dead_code)]
pub unsafe fn destroy_user_page_table(_ttbr0_phys: usize) -> Result<(), &'static str> {
    // TODO(TEAM_073): Implement full page table teardown
    // For now, we leak the pages - will be fixed when process cleanup is added
    Ok(())
}
