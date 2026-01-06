//! TEAM_073: User Address Space Management for LevitateOS.
//!
//! This module provides:
//! - Per-process TTBR0 page table creation
//! - User memory mapping functions
//! - Address space layout for user processes

use crate::memory::FRAME_ALLOCATOR;
use los_hal::mmu::{self, MmuError, PAGE_SIZE, PageAllocator, PageFlags, PageTable};

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
    los_hal::println!("[MMU] Creating user page table...");
    // Allocate a page for L0 table
    let l0_phys = FRAME_ALLOCATOR.alloc_page()?;

    los_hal::println!("[MMU] Allocated L0 table at phys [MASKED]");

    // Zero the table
    let l0_va = mmu::phys_to_virt(l0_phys);
    los_hal::println!("[MMU] Zeroing L0 table at va [MASKED]");
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
) -> Result<(), MmuError> {
    // TEAM_152: Updated to use MmuError
    // Validate user address
    if user_va >= layout::USER_SPACE_END {
        return Err(MmuError::InvalidVirtualAddress);
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
) -> Result<(), MmuError> {
    // TEAM_152: Updated to use MmuError
    // Validate user address
    if user_va_start >= layout::USER_SPACE_END {
        return Err(MmuError::InvalidVirtualAddress);
    }
    if user_va_start.saturating_add(len) > layout::USER_SPACE_END {
        return Err(MmuError::InvalidVirtualAddress);
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
) -> Result<usize, MmuError> {
    let stack_size = stack_pages * PAGE_SIZE;
    let stack_bottom = layout::STACK_TOP - stack_size;

    // Allocate physical pages for stack
    for i in 0..stack_pages {
        let page_va = stack_bottom + i * PAGE_SIZE;

        // Allocate physical page
        let phys = FRAME_ALLOCATOR
            .alloc_page()
            .ok_or(MmuError::AllocationFailed)?;

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

/// TEAM_169: Set up user stack with argc/argv/envp.
///
/// Per Phase 2 Q5 decision: Stack-based argument passing (Linux ABI compatible).
///
/// Stack layout (grows downward):
/// ```text
/// High addresses
///   +---------------+
///   | env strings   |  <- Environment variable strings
///   | arg strings   |  <- Argument strings  
///   | NULL          |  <- envp terminator
///   | envp[n-1]     |  <- Environment pointers
///   | ...           |
///   | envp[0]       |
///   | NULL          |  <- argv terminator
///   | argv[argc-1]  |  <- Argument pointers
///   | ...           |
///   | argv[0]       |
///   | argc          |  <- SP points here
///   +---------------+
/// Low addresses
/// ```
///
/// # Arguments
/// * `ttbr0_phys` - Physical address of user L0 page table
/// * `stack_top` - Top of the allocated stack
/// * `args` - Argument strings
/// * `envs` - Environment variable strings
///
/// # Returns
/// New stack pointer after setting up arguments.
pub fn setup_stack_args(
    ttbr0_phys: usize,
    stack_top: usize,
    args: &[&str],
    envs: &[&str],
) -> Result<usize, MmuError> {
    let mut sp = stack_top;

    // Helper to write a usize to user stack
    let write_usize = |sp: &mut usize, val: usize| -> Result<(), MmuError> {
        *sp -= core::mem::size_of::<usize>();
        let ptr = user_va_to_kernel_ptr(ttbr0_phys, *sp)
            .ok_or(MmuError::InvalidVirtualAddress)?;
        unsafe {
            *(ptr as *mut usize) = val;
        }
        Ok(())
    };

    // Helper to write a string to user stack (returns pointer to string)
    let write_string = |sp: &mut usize, s: &str| -> Result<usize, MmuError> {
        let len = s.len() + 1; // Include null terminator
        *sp -= len;
        *sp &= !7; // Align to 8 bytes
        let str_ptr = *sp;
        
        for (i, byte) in s.bytes().enumerate() {
            let ptr = user_va_to_kernel_ptr(ttbr0_phys, str_ptr + i)
                .ok_or(MmuError::InvalidVirtualAddress)?;
            unsafe { *ptr = byte; }
        }
        // Null terminator
        let ptr = user_va_to_kernel_ptr(ttbr0_phys, str_ptr + s.len())
            .ok_or(MmuError::InvalidVirtualAddress)?;
        unsafe { *ptr = 0; }
        
        Ok(str_ptr)
    };

    // 1. Write all strings to stack (env first, then args)
    let mut env_ptrs = alloc::vec::Vec::new();
    for env in envs.iter().rev() {
        let ptr = write_string(&mut sp, env)?;
        env_ptrs.push(ptr);
    }
    env_ptrs.reverse();

    let mut arg_ptrs = alloc::vec::Vec::new();
    for arg in args.iter().rev() {
        let ptr = write_string(&mut sp, arg)?;
        arg_ptrs.push(ptr);
    }
    arg_ptrs.reverse();

    // Align stack to 16 bytes before writing pointers
    sp &= !15;

    // 2. Write envp[] array (NULL terminated)
    write_usize(&mut sp, 0)?; // NULL terminator
    for ptr in env_ptrs.iter().rev() {
        write_usize(&mut sp, *ptr)?;
    }

    // 3. Write argv[] array (NULL terminated)
    write_usize(&mut sp, 0)?; // NULL terminator
    for ptr in arg_ptrs.iter().rev() {
        write_usize(&mut sp, *ptr)?;
    }

    // 4. Write argc
    write_usize(&mut sp, args.len())?;

    // Ensure final alignment
    sp &= !15;

    Ok(sp)
}

/// TEAM_073: Allocate physical pages and map them for user code/data.
#[allow(dead_code)]
pub unsafe fn alloc_and_map_user_range(
    ttbr0_phys: usize,
    user_va_start: usize,
    len: usize,
    flags: PageFlags,
) -> Result<usize, MmuError> {
    // TEAM_152: Updated to use MmuError
    if len == 0 {
        return Err(MmuError::InvalidVirtualAddress);
    }

    let va_start = user_va_start & !0xFFF;
    let pages_needed = (len + (user_va_start - va_start) + PAGE_SIZE - 1) / PAGE_SIZE;

    let mut first_phys = 0;

    for i in 0..pages_needed {
        let page_va = va_start + i * PAGE_SIZE;

        // Allocate physical page
        let phys = FRAME_ALLOCATOR
            .alloc_page()
            .ok_or(MmuError::AllocationFailed)?;

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
pub unsafe fn destroy_user_page_table(_ttbr0_phys: usize) -> Result<(), MmuError> {
    // TODO(TEAM_073): Implement full page table teardown
    // For now, we leak the pages - will be fixed when process cleanup is added
    Ok(())
}

/// TEAM_166: Allocate and map a single heap page for sbrk.
///
/// # Arguments
/// * `ttbr0_phys` - Physical address of user L0 page table
/// * `user_va` - Virtual address to map (page-aligned)
///
/// # Returns
/// Ok(()) on success, Err on allocation or mapping failure.
pub fn alloc_and_map_heap_page(ttbr0_phys: usize, user_va: usize) -> Result<(), MmuError> {
    // Allocate physical page
    let phys = FRAME_ALLOCATOR
        .alloc_page()
        .ok_or(MmuError::AllocationFailed)?;

    // Zero the page for security
    let page_ptr = mmu::phys_to_virt(phys) as *mut u8;
    unsafe {
        core::ptr::write_bytes(page_ptr, 0, PAGE_SIZE);
    }

    // Map into user address space with heap flags (RW, user accessible)
    unsafe {
        map_user_page_at(ttbr0_phys, user_va, phys, PageFlags::USER_DATA)?;
    }

    Ok(())
}

/// TEAM_166: Internal helper - map a page at a specific physical address.
/// Renamed from the original map_user_page to avoid confusion.
unsafe fn map_user_page_at(
    ttbr0_phys: usize,
    user_va: usize,
    phys: usize,
    flags: PageFlags,
) -> Result<(), MmuError> {
    // Validate user address
    if user_va >= layout::USER_SPACE_END {
        return Err(MmuError::InvalidVirtualAddress);
    }

    // Get the L0 table
    let l0_va = mmu::phys_to_virt(ttbr0_phys);
    let l0 = unsafe { &mut *(l0_va as *mut PageTable) };

    // Use MMU's map_page function
    mmu::map_page(l0, user_va, phys, flags)
}

/// TEAM_156: Translate a user virtual address to a kernel-accessible pointer.
///
/// This walks the user's page table to find the physical address,
/// then converts it to a kernel VA that can be safely accessed.
///
/// # Safety
/// - `ttbr0_phys` must be a valid user page table
/// - The user VA must be mapped
/// - Caller must ensure proper synchronization
pub fn user_va_to_kernel_ptr(ttbr0_phys: usize, user_va: usize) -> Option<*mut u8> {
    // Get L0 table
    let l0_va = mmu::phys_to_virt(ttbr0_phys);
    let l0 = unsafe { &mut *(l0_va as *mut PageTable) };

    // Walk page tables to find physical address
    let page_va = user_va & !0xFFF;
    let page_offset = user_va & 0xFFF;

    if let Ok(walk) = mmu::walk_to_entry(l0, page_va, 3, false) {
        let entry = walk.table.entry(walk.index);
        if entry.is_valid() {
            let entry_phys = entry.address();
            let dst_phys = entry_phys + page_offset;
            let kernel_va = mmu::phys_to_virt(dst_phys);
            return Some(kernel_va as *mut u8);
        }
    }
    None
}

/// TEAM_137: Validate a user buffer range.
/// Checks that all pages in the range are mapped and have correct permissions for EL0.
pub fn validate_user_buffer(
    ttbr0_phys: usize,
    ptr: usize,
    len: usize,
    writable: bool,
) -> Result<(), MmuError> {
    // TEAM_152: Updated to use MmuError
    // 1. Check user address space bounds
    if ptr >= layout::USER_SPACE_END {
        return Err(MmuError::InvalidVirtualAddress);
    }
    // Check for overflow or exceeding user space
    if let Some(end) = ptr.checked_add(len) {
        if end > layout::USER_SPACE_END {
            return Err(MmuError::InvalidVirtualAddress);
        }
    } else {
        return Err(MmuError::InvalidVirtualAddress);
    }

    if len == 0 {
        return Ok(());
    }

    // 2. Get L0 table (Read-Only access pattern)
    let l0_va = mmu::phys_to_virt(ttbr0_phys);
    // SAFETY: ttbr0_phys is guaranteed to be a valid page table by caller (process struct)
    let l0 = unsafe { &*(l0_va as *const PageTable) };

    // 3. Iterate over every page touched by the buffer
    let mut current = ptr;
    let end = ptr + len;

    while current < end {
        // Translate VA -> PA + Flags
        match mmu::translate(l0, current) {
            Some((_pa, flags)) => {
                // Check VALID bit (implicit in translate, but good to be explicit)
                if !flags.contains(PageFlags::VALID) {
                    return Err(MmuError::NotMapped);
                }

                // Check User Accessibility (AP bit 6 must be set)
                // 00 (RW_EL1) -> Bit 6=0
                // 10 (RO_EL1) -> Bit 6=0
                // 01 (RW_ALL) -> Bit 6=1
                // 11 (RO_ALL) -> Bit 6=1
                let ap_bits = (flags.bits() >> 6) & 0b11;
                let is_user = (ap_bits & 0b01) != 0;

                if !is_user {
                    return Err(MmuError::NotMapped);
                }

                // Check Write Permission if requested
                if writable {
                    // Must be RW_ALL (01)
                    // If it is RO_ALL (11), then write is denied.
                    // 01 & 11 -> 01 (Bit 7 is the difference)
                    // RW_ALL: 01 (Bit 6=1, Bit 7=0)
                    // RO_ALL: 11 (Bit 6=1, Bit 7=1)
                    // So we must ensure Bit 7 is 0.
                    let is_readonly = (ap_bits & 0b10) != 0;
                    if is_readonly {
                        return Err(MmuError::NotMapped);
                    }
                }
            }
            None => return Err(MmuError::NotMapped),
        }

        // Move to next page boundary
        // If current is 0x1000, next is 0x2000.
        // If current is 0x1005, next is 0x2000.
        // Formula: (current & !0xFFF) + 0x1000
        let next_page = (current & !0xFFF) + PAGE_SIZE;
        current = next_page;
    }

    Ok(())
}
