use crate::memory::FRAME_ALLOCATOR;
use crate::memory::user as mm_user;
use los_hal::mmu::{self, PAGE_SIZE, PageAllocator, PageFlags};

// Memory management system calls.

// TEAM_228: mmap protection flags (matching Linux)
pub const PROT_NONE: u32 = 0;
pub const PROT_READ: u32 = 1;
pub const PROT_WRITE: u32 = 2;
pub const PROT_EXEC: u32 = 4;

// TEAM_228: mmap flags (matching Linux)
pub const MAP_SHARED: u32 = 0x01;
pub const MAP_PRIVATE: u32 = 0x02;
pub const MAP_FIXED: u32 = 0x10;
pub const MAP_ANONYMOUS: u32 = 0x20;

// TEAM_228: Error codes
const ENOMEM: i64 = -12;
const EINVAL: i64 = -22;

/// TEAM_238: RAII guard for mmap allocation cleanup.
///
/// Tracks pages allocated during mmap. On drop, frees all unless committed.
/// This ensures partial allocations are cleaned up on failure.
struct MmapGuard {
    /// (virtual_address, physical_address) pairs
    allocated: alloc::vec::Vec<(usize, usize)>,
    /// User page table physical address
    ttbr0: usize,
    /// Set to true when allocation succeeds
    committed: bool,
}

impl MmapGuard {
    /// Create a new guard for the given user page table.
    fn new(ttbr0: usize) -> Self {
        Self {
            allocated: alloc::vec::Vec::new(),
            ttbr0,
            committed: false,
        }
    }

    /// Track an allocated page.
    fn track(&mut self, va: usize, phys: usize) {
        self.allocated.push((va, phys));
    }

    /// Commit the allocation - pages will NOT be freed on drop.
    fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for MmapGuard {
    fn drop(&mut self) {
        if self.committed {
            return; // Success path - keep pages
        }

        // Failure path - clean up all allocated pages
        use los_hal::mmu::{PageTable, phys_to_virt, tlb_flush_page};

        let l0_va = phys_to_virt(self.ttbr0);
        let l0 = unsafe { &mut *(l0_va as *mut PageTable) };

        for &(va, phys) in &self.allocated {
            // 1. Unmap the page (clear PTE)
            if let Ok(walk) = mmu::walk_to_entry(l0, va, 3, false) {
                walk.table.entry_mut(walk.index).clear();
                tlb_flush_page(va);
            }

            // 2. Free the physical page
            FRAME_ALLOCATOR.free_page(phys);
        }
    }
}

/// TEAM_166: sys_sbrk - Adjust program break (heap allocation).
pub fn sys_sbrk(increment: isize) -> i64 {
    let task = crate::task::current_task();
    let mut heap = task.heap.lock();

    match heap.grow(increment) {
        Ok(old_break) => {
            if increment > 0 {
                let new_break = heap.current;
                let old_page = old_break / los_hal::mmu::PAGE_SIZE;
                // TEAM_181: Use checked arithmetic to prevent overflow
                let new_page = match new_break.checked_add(los_hal::mmu::PAGE_SIZE - 1) {
                    Some(n) => n / los_hal::mmu::PAGE_SIZE,
                    None => {
                        heap.current = old_break;
                        return 0; // Overflow
                    }
                };

                for page in old_page..new_page {
                    let va = page * los_hal::mmu::PAGE_SIZE;
                    if mm_user::user_va_to_kernel_ptr(task.ttbr0, va).is_none() {
                        if mm_user::alloc_and_map_heap_page(task.ttbr0, va).is_err() {
                            heap.current = old_break;
                            return 0; // null
                        }
                    }
                }
            }
            old_break as i64
        }
        Err(()) => 0,
    }
}

/// TEAM_228: sys_mmap - Map memory into process address space.
///
/// For std allocator support, we implement anonymous private mappings.
/// File-backed mappings are not yet supported.
///
/// # Arguments
/// * `addr` - Hint address (ignored unless MAP_FIXED)
/// * `len` - Length of mapping
/// * `prot` - Protection flags (PROT_READ, PROT_WRITE, PROT_EXEC)
/// * `flags` - Mapping flags (MAP_PRIVATE, MAP_ANONYMOUS required for now)
/// * `fd` - File descriptor (must be -1 for MAP_ANONYMOUS)
/// * `offset` - File offset (must be 0 for MAP_ANONYMOUS)
///
/// # Returns
/// Virtual address of mapping, or negative error code.
pub fn sys_mmap(addr: usize, len: usize, prot: u32, flags: u32, fd: i32, offset: usize) -> i64 {
    // TEAM_228: Validate arguments
    if len == 0 {
        return EINVAL;
    }

    // For MVP, only support MAP_ANONYMOUS | MAP_PRIVATE
    if flags & MAP_ANONYMOUS == 0 {
        log::warn!(
            "[MMAP] Only MAP_ANONYMOUS supported, got flags=0x{:x}",
            flags
        );
        return EINVAL;
    }
    if fd != -1 || offset != 0 {
        log::warn!("[MMAP] File-backed mappings not supported");
        return EINVAL;
    }

    let task = crate::task::current_task();
    let ttbr0 = task.ttbr0;

    // TEAM_238: Create RAII guard for cleanup on failure
    let mut guard = MmapGuard::new(ttbr0);

    // Round up length to page boundary
    let pages_needed = (len + PAGE_SIZE - 1) / PAGE_SIZE;
    let alloc_len = pages_needed * PAGE_SIZE;

    // Find free region in user address space
    // Start searching from a reasonable base (0x1000_0000_0000) if no hint
    let base_addr = if addr != 0 && flags & MAP_FIXED != 0 {
        // MAP_FIXED: use exact address (must be page-aligned)
        if addr & (PAGE_SIZE - 1) != 0 {
            return EINVAL;
        }
        addr
    } else {
        // Find a free region - start at a safe mmap area
        // TEAM_228: Use a simple linear search for free space
        find_free_mmap_region(ttbr0, alloc_len).unwrap_or(0)
    };

    if base_addr == 0 {
        return ENOMEM;
    }

    // Convert prot to PageFlags
    let page_flags = prot_to_page_flags(prot);

    // Allocate and map pages
    for i in 0..pages_needed {
        let va = base_addr + i * PAGE_SIZE;

        // Allocate physical page
        let phys = match FRAME_ALLOCATOR.alloc_page() {
            Some(p) => p,
            None => {
                // TEAM_238: Guard will clean up on drop
                return ENOMEM;
            }
        };

        // Zero the page
        let page_ptr = mmu::phys_to_virt(phys) as *mut u8;
        unsafe {
            core::ptr::write_bytes(page_ptr, 0, PAGE_SIZE);
        }

        // Map into user address space
        if unsafe { mm_user::map_user_page(ttbr0, va, phys, page_flags) }.is_err() {
            // TEAM_238: Free this page (not tracked yet) and let guard clean up rest
            FRAME_ALLOCATOR.free_page(phys);
            return ENOMEM;
        }

        // TEAM_238: Track successful allocation
        guard.track(va, phys);
    }

    // TEAM_238: Success - commit the guard (pages kept)
    guard.commit();

    // TEAM_238: Record the VMA
    {
        use crate::memory::vma::{Vma, VmaFlags};
        let mut vma_flags = VmaFlags::empty();
        if prot & PROT_READ != 0 {
            vma_flags |= VmaFlags::READ;
        }
        if prot & PROT_WRITE != 0 {
            vma_flags |= VmaFlags::WRITE;
        }
        if prot & PROT_EXEC != 0 {
            vma_flags |= VmaFlags::EXEC;
        }
        let vma = Vma::new(base_addr, base_addr + alloc_len, vma_flags);
        let mut vmas = task.vmas.lock();
        // Ignore error if overlapping (shouldn't happen with proper mmap)
        let _ = vmas.insert(vma);
    }

    log::trace!(
        "[MMAP] Mapped {} pages at 0x{:x} with prot=0x{:x}",
        pages_needed,
        base_addr,
        prot
    );

    base_addr as i64
}

/// TEAM_228: sys_munmap - Unmap memory from process address space.
/// TEAM_238: Implemented proper VMA tracking and page unmapping.
///
/// # Arguments
/// * `addr` - Start address of mapping (must be page-aligned)
/// * `len` - Length to unmap
///
/// # Returns
/// 0 on success, negative error code on failure.
pub fn sys_munmap(addr: usize, len: usize) -> i64 {
    use los_hal::mmu::{PageTable, phys_to_virt, tlb_flush_page};

    // Validate alignment
    if addr & 0xFFF != 0 {
        return EINVAL;
    }

    if len == 0 {
        return EINVAL;
    }

    // Page-align length
    let aligned_len = (len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
    let end = match addr.checked_add(aligned_len) {
        Some(e) => e,
        None => return EINVAL, // Overflow
    };

    let task = crate::task::current_task();
    let ttbr0 = task.ttbr0;

    // 1. Remove VMA(s) from tracking
    {
        let mut vmas = task.vmas.lock();
        if vmas.remove(addr, end).is_err() {
            // No VMA found - per POSIX, this is not necessarily an error
            // but we'll return success anyway (Linux behavior)
            log::trace!("[MUNMAP] No VMA found for 0x{:x}-0x{:x}", addr, end);
        }
    }

    // 2. Unmap and free pages
    let l0_va = phys_to_virt(ttbr0);
    let l0 = unsafe { &mut *(l0_va as *mut PageTable) };

    let mut current = addr;
    while current < end {
        // Try to find the page mapping
        if let Ok(walk) = mmu::walk_to_entry(l0, current, 3, false) {
            let entry = walk.table.entry(walk.index);
            if entry.is_valid() {
                // Get physical address before clearing
                let phys = entry.address();

                // Clear the entry
                walk.table.entry_mut(walk.index).clear();

                // Flush TLB for this page
                tlb_flush_page(current);

                // Free the physical page
                FRAME_ALLOCATOR.free_page(phys);
            }
        }

        current += PAGE_SIZE;
    }

    log::trace!("[MUNMAP] Unmapped 0x{:x}-0x{:x}", addr, end);

    0 // Success
}

/// TEAM_239: sys_mprotect - Change protection on memory region.
///
/// # Arguments
/// * `addr` - Start address (must be page-aligned)
/// * `len` - Length of region
/// * `prot` - New protection flags
///
/// # Returns
/// 0 on success, negative error code on failure.
pub fn sys_mprotect(addr: usize, len: usize, prot: u32) -> i64 {
    use los_hal::mmu::{PageFlags, PageTable, phys_to_virt, tlb_flush_page};

    // Validate alignment
    if addr & (PAGE_SIZE - 1) != 0 {
        return EINVAL;
    }

    if len == 0 {
        return EINVAL;
    }

    // Page-align length
    let aligned_len = (len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
    let end = match addr.checked_add(aligned_len) {
        Some(e) => e,
        None => return EINVAL, // Overflow
    };

    let task = crate::task::current_task();
    let ttbr0 = task.ttbr0;

    // Convert prot to PageFlags
    let new_flags = prot_to_page_flags(prot);

    // Get access to root page table
    let l0_va = phys_to_virt(ttbr0);
    let l0 = unsafe { &mut *(l0_va as *mut PageTable) };

    // Walk each page and update protection
    let mut current = addr;
    let mut modified_count = 0usize;

    while current < end {
        // Walk to the L3 (leaf) entry for this page
        if let Ok(walk) = mmu::walk_to_entry(l0, current, 3, false) {
            let entry = walk.table.entry(walk.index);
            if entry.is_valid() {
                // Get the physical address (preserve it)
                let phys = entry.address();

                // Set new entry with same address but new flags
                // L3 entries use TABLE bit = 1 for pages
                walk.table
                    .entry_mut(walk.index)
                    .set(phys, new_flags | PageFlags::TABLE);

                // Flush TLB for this page
                tlb_flush_page(current);
                modified_count += 1;
            }
        }
        // If page not mapped, skip it (Linux behavior)

        current += PAGE_SIZE;
    }

    // Update VMA tracking
    {
        use crate::memory::vma::VmaFlags;
        let mut vma_flags = VmaFlags::empty();
        if prot & PROT_READ != 0 {
            vma_flags |= VmaFlags::READ;
        }
        if prot & PROT_WRITE != 0 {
            vma_flags |= VmaFlags::WRITE;
        }
        if prot & PROT_EXEC != 0 {
            vma_flags |= VmaFlags::EXEC;
        }

        let mut vmas = task.vmas.lock();
        // Update protection flags for overlapping VMAs
        vmas.update_protection(addr, end, vma_flags);
    }

    log::trace!(
        "[MPROTECT] Changed protection for {} pages at 0x{:x}-0x{:x} prot=0x{:x}",
        modified_count,
        addr,
        end,
        prot
    );

    0 // Success
}

/// TEAM_228: Find a free region in user address space for mmap.
///
/// This is a simple implementation that searches for unmapped pages.
/// A production implementation would use a proper VMA tree.
fn find_free_mmap_region(ttbr0: usize, len: usize) -> Option<usize> {
    // Start searching from mmap area (above typical heap, below stack)
    const MMAP_START: usize = 0x0000_1000_0000_0000; // 16 TiB
    const MMAP_END: usize = 0x0000_7000_0000_0000; // Well below stack

    let pages_needed = len / PAGE_SIZE;
    let mut search_addr = MMAP_START;

    while search_addr + len <= MMAP_END {
        // Check if this region is free
        let mut all_free = true;
        for i in 0..pages_needed {
            let test_addr = search_addr + i * PAGE_SIZE;
            if mm_user::user_va_to_kernel_ptr(ttbr0, test_addr).is_some() {
                // This page is already mapped
                all_free = false;
                // Skip past this mapped page
                search_addr = test_addr + PAGE_SIZE;
                break;
            }
        }

        if all_free {
            return Some(search_addr);
        }
    }

    None
}

/// TEAM_228: Convert prot flags to PageFlags.
fn prot_to_page_flags(prot: u32) -> PageFlags {
    // Start with user-accessible base
    let mut flags = PageFlags::USER_DATA;

    if prot & PROT_EXEC != 0 {
        flags = PageFlags::USER_CODE;
    }

    // Note: PROT_NONE would need a different approach - we'd need to
    // map the page but make it inaccessible. For now, treat as USER_DATA.

    flags
}

/// TEAM_350: sys_madvise - Give advice about use of memory.
///
/// This is a stub that ignores all advice and returns success.
/// Allocators call madvise but can tolerate it failing or being ignored.
///
/// # Arguments
/// * `addr` - Start address of memory region
/// * `len` - Length of memory region
/// * `advice` - Advice hint (MADV_DONTNEED, MADV_WILLNEED, etc.)
///
/// # Returns
/// 0 on success (we always succeed by ignoring the advice)
#[allow(unused_variables)]
pub fn sys_madvise(addr: usize, len: usize, advice: i32) -> i64 {
    // TEAM_350: Stub - ignore advice, return success
    // Common advice values:
    // MADV_NORMAL = 0, MADV_RANDOM = 1, MADV_SEQUENTIAL = 2
    // MADV_WILLNEED = 3, MADV_DONTNEED = 4
    log::trace!(
        "[SYSCALL] madvise(addr=0x{:x}, len=0x{:x}, advice={})",
        addr,
        len,
        advice
    );
    0
}
