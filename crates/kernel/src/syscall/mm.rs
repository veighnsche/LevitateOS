//! TEAM_228: Memory management syscalls.
//! TEAM_415: Refactored with helper functions.
//! TEAM_419: Use linux-raw-sys for mmap constants.
//! TEAM_420: Direct linux_raw_sys imports, no shims
//! TEAM_421: Return SyscallResult, no scattered casts

use crate::memory::FRAME_ALLOCATOR;
use crate::memory::user as mm_user;
use crate::memory::vma::VmaFlags;
use crate::syscall::SyscallResult;
use linux_raw_sys::errno::{EINVAL, ENOSYS, ENOMEM};
use linux_raw_sys::general::{
    PROT_NONE, PROT_READ, PROT_WRITE, PROT_EXEC,
    MAP_SHARED, MAP_PRIVATE, MAP_FIXED, MAP_ANONYMOUS,
};
use los_hal::mmu::{self, PAGE_SIZE, PageAllocator, PageFlags, PageTable, phys_to_virt, tlb_flush_page};

// ============================================================================
// TEAM_415: Helper Functions
// ============================================================================

/// TEAM_415: Round up a length to the next page boundary.
#[inline]
fn page_align_up(len: usize) -> usize {
    (len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

/// TEAM_415: Get mutable reference to user page table from TTBR0.
///
/// # Safety
/// Caller must ensure ttbr0 is a valid page table physical address.
#[inline]
unsafe fn get_user_page_table(ttbr0: usize) -> &'static mut PageTable {
    let l0_va = phys_to_virt(ttbr0);
    &mut *(l0_va as *mut PageTable)
}

/// TEAM_415: Convert PROT_* flags to VmaFlags.
fn prot_to_vma_flags(prot: u32) -> VmaFlags {
    let mut flags = VmaFlags::empty();
    if prot & PROT_READ != 0 {
        flags |= VmaFlags::READ;
    }
    if prot & PROT_WRITE != 0 {
        flags |= VmaFlags::WRITE;
    }
    if prot & PROT_EXEC != 0 {
        flags |= VmaFlags::EXEC;
    }
    flags
}

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
        // SAFETY: ttbr0 was validated when MmapGuard was created
        let l0 = unsafe { get_user_page_table(self.ttbr0) };

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
/// TEAM_421: Returns SyscallResult
pub fn sys_sbrk(increment: isize) -> SyscallResult {
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
                        return Ok(0); // Overflow - return current break
                    }
                };

                for page in old_page..new_page {
                    let va = page * los_hal::mmu::PAGE_SIZE;
                    if mm_user::user_va_to_kernel_ptr(task.ttbr0, va).is_none() {
                        if mm_user::alloc_and_map_heap_page(task.ttbr0, va).is_err() {
                            // TEAM_389: Log OOM for debugging (Rule 4: Silence is Golden - use debug level)
                            log::debug!("[OOM] sys_sbrk: failed to allocate page at VA 0x{:x}", va);
                            heap.current = old_break;
                            return Err(ENOMEM); // TEAM_389: Return error, not null
                        }
                    }
                }
            }
            Ok(old_break as i64)
        }
        Err(()) => {
            // TEAM_389: Log heap bounds exceeded for debugging
            log::debug!("[OOM] sys_sbrk: heap bounds exceeded (increment={})", increment);
            Err(ENOMEM) // TEAM_389: Return error on heap bounds exceeded
        }
    }
}

/// TEAM_228: sys_mmap - Map memory into process address space.
/// TEAM_421: Returns SyscallResult
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
/// Ok(virtual_address) on success, Err(errno) on failure.
pub fn sys_mmap(addr: usize, len: usize, prot: u32, flags: u32, fd: i32, offset: usize) -> SyscallResult {
    // TEAM_228: Validate arguments
    if len == 0 {
        return Err(EINVAL);
    }

    // For MVP, only support MAP_ANONYMOUS | MAP_PRIVATE
    if flags & MAP_ANONYMOUS == 0 {
        log::warn!(
            "[MMAP] Only MAP_ANONYMOUS supported, got flags=0x{:x}",
            flags
        );
        return Err(EINVAL);
    }
    if fd != -1 || offset != 0 {
        log::warn!("[MMAP] File-backed mappings not supported");
        return Err(EINVAL);
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
            return Err(EINVAL);
        }
        addr
    } else {
        // Find a free region - start at a safe mmap area
        // TEAM_228: Use a simple linear search for free space
        find_free_mmap_region(ttbr0, alloc_len).unwrap_or(0)
    };

    if base_addr == 0 {
        log::debug!("[OOM] sys_mmap: no free region for {} bytes", alloc_len);
        return Err(ENOMEM);
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
                // TEAM_389: Log OOM for debugging
                log::debug!("[OOM] sys_mmap: failed to allocate frame for page {}/{}", i + 1, pages_needed);
                // TEAM_238: Guard will clean up on drop
                return Err(ENOMEM);
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
            return Err(ENOMEM);
        }

        // TEAM_238: Track successful allocation
        guard.track(va, phys);
    }

    // TEAM_238: Success - commit the guard (pages kept)
    guard.commit();

    // TEAM_238: Record the VMA
    // TEAM_415: Use prot_to_vma_flags helper
    {
        use crate::memory::vma::Vma;
        let vma = Vma::new(base_addr, base_addr + alloc_len, prot_to_vma_flags(prot));
        let mut vmas = task.vmas.lock();
        let _ = vmas.insert(vma); // Ignore error if overlapping
    }

    log::trace!(
        "[MMAP] Mapped {} pages at 0x{:x} with prot=0x{:x}",
        pages_needed,
        base_addr,
        prot
    );

    Ok(base_addr as i64)
}

/// TEAM_228: sys_munmap - Unmap memory from process address space.
/// TEAM_238: Implemented proper VMA tracking and page unmapping.
/// TEAM_415: Refactored to use helper functions.
/// TEAM_421: Returns SyscallResult
pub fn sys_munmap(addr: usize, len: usize) -> SyscallResult {
    // Validate alignment
    if addr & 0xFFF != 0 || len == 0 {
        return Err(EINVAL);
    }

    let aligned_len = page_align_up(len);
    let end = match addr.checked_add(aligned_len) {
        Some(e) => e,
        None => return Err(EINVAL),
    };

    let task = crate::task::current_task();
    let ttbr0 = task.ttbr0;

    // 1. Remove VMA(s) from tracking
    {
        let mut vmas = task.vmas.lock();
        if vmas.remove(addr, end).is_err() {
            log::trace!("[MUNMAP] No VMA found for 0x{:x}-0x{:x}", addr, end);
        }
    }

    // 2. Unmap and free pages
    // SAFETY: ttbr0 from task is always valid
    let l0 = unsafe { get_user_page_table(ttbr0) };

    let mut current = addr;
    while current < end {
        if let Ok(walk) = mmu::walk_to_entry(l0, current, 3, false) {
            let entry = walk.table.entry(walk.index);
            if entry.is_valid() {
                let phys = entry.address();
                walk.table.entry_mut(walk.index).clear();
                tlb_flush_page(current);
                FRAME_ALLOCATOR.free_page(phys);
            }
        }
        current += PAGE_SIZE;
    }

    log::trace!("[MUNMAP] Unmapped 0x{:x}-0x{:x}", addr, end);
    Ok(0)
}

/// TEAM_239: sys_mprotect - Change protection on memory region.
/// TEAM_415: Refactored to use helper functions.
/// TEAM_421: Returns SyscallResult
pub fn sys_mprotect(addr: usize, len: usize, prot: u32) -> SyscallResult {
    // Validate alignment
    if addr & (PAGE_SIZE - 1) != 0 || len == 0 {
        return Err(EINVAL);
    }

    let aligned_len = page_align_up(len);
    let end = match addr.checked_add(aligned_len) {
        Some(e) => e,
        None => return Err(EINVAL),
    };

    let task = crate::task::current_task();
    let new_flags = prot_to_page_flags(prot);

    // SAFETY: ttbr0 from task is always valid
    let l0 = unsafe { get_user_page_table(task.ttbr0) };

    // Walk each page and update protection
    let mut current = addr;
    let mut modified_count = 0usize;

    while current < end {
        if let Ok(walk) = mmu::walk_to_entry(l0, current, 3, false) {
            let entry = walk.table.entry(walk.index);
            if entry.is_valid() {
                let phys = entry.address();
                walk.table.entry_mut(walk.index).set(phys, new_flags | PageFlags::TABLE);
                tlb_flush_page(current);
                modified_count += 1;
            }
        }
        current += PAGE_SIZE;
    }

    // Update VMA tracking
    task.vmas.lock().update_protection(addr, end, prot_to_vma_flags(prot));

    log::trace!(
        "[MPROTECT] Changed protection for {} pages at 0x{:x}-0x{:x} prot=0x{:x}",
        modified_count, addr, end, prot
    );

    Ok(0)
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
/// TEAM_421: Returns SyscallResult
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
/// Ok(0) on success (we always succeed by ignoring the advice)
#[allow(unused_variables)]
pub fn sys_madvise(addr: usize, len: usize, advice: i32) -> SyscallResult {
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
    Ok(0)
}

/// TEAM_360: sys_pkey_alloc - Allocate a memory protection key.
/// TEAM_421: Returns SyscallResult
///
/// Memory protection keys (Intel MPK) are not supported.
/// This syscall returns Err(ENOSYS) to indicate the feature is unavailable.
///
/// # Arguments
/// * `flags` - Must be 0
/// * `access_rights` - PKEY_DISABLE_ACCESS or PKEY_DISABLE_WRITE
///
/// # Returns
/// Err(ENOSYS) (syscall not implemented)
#[allow(unused_variables)]
pub fn sys_pkey_alloc(flags: u32, access_rights: u32) -> SyscallResult {
    log::trace!(
        "[SYSCALL] pkey_alloc(flags={}, access_rights={}) -> ENOSYS",
        flags,
        access_rights
    );
    // TEAM_360: Memory protection keys not supported
    Err(ENOSYS)
}

/// TEAM_360: sys_pkey_mprotect - Set memory protection with protection key.
/// TEAM_421: Returns SyscallResult
///
/// Memory protection keys (Intel MPK) are not supported.
/// This syscall returns Err(ENOSYS) to indicate the feature is unavailable.
///
/// # Arguments
/// * `addr` - Start address of memory region
/// * `len` - Length of memory region
/// * `prot` - Protection flags
/// * `pkey` - Protection key
///
/// # Returns
/// Err(ENOSYS) (syscall not implemented)
#[allow(unused_variables)]
pub fn sys_pkey_mprotect(addr: usize, len: usize, prot: u32, pkey: i32) -> SyscallResult {
    log::trace!(
        "[SYSCALL] pkey_mprotect(addr=0x{:x}, len=0x{:x}, prot={}, pkey={}) -> ENOSYS",
        addr,
        len,
        prot,
        pkey
    );
    // TEAM_360: Memory protection keys not supported
    Err(ENOSYS)
}
