//! AArch64 Memory Management Unit (MMU) support.
//!
//! TEAM_018: Implements page table structures and MMU configuration
//! for AArch64 with 4KB granule, 48-bit virtual addresses.
//!
//! Reference implementations studied:
//! - Theseus: MAIR/TCR config, TLB flush patterns
//! - Redox: RMM abstraction

use bitflags::bitflags;

// ============================================================================
// Constants
// ============================================================================

/// Page size: 4KB
pub const PAGE_SIZE: usize = 4096;
/// Page shift (log2 of page size)
pub const PAGE_SHIFT: usize = 12;
/// Entries per page table (512 for 4KB pages with 8-byte entries)
pub const ENTRIES_PER_TABLE: usize = 512;

/// Kernel physical start address (from linker.ld)
pub const KERNEL_PHYS_START: usize = 0x4008_0000;
/// Kernel physical end address (matches __heap_end in linker.ld)
/// Note: linker.ld sets __heap_end = _kernel_virt_base + 0x41F00000
pub const KERNEL_PHYS_END: usize = 0x41F0_0000;

/// Kernel virtual start address (Higher-half base)
pub const KERNEL_VIRT_START: usize = 0xFFFF_8000_0000_0000;

/// [M19] Converts high VA to PA, [M21] identity for low addresses
#[inline]
pub fn virt_to_phys(va: usize) -> usize {
    if va >= KERNEL_VIRT_START {
        va - KERNEL_VIRT_START  // [M19] high VA to PA
    } else {
        va // [M21] identity for low addresses
    }
}

/// [M20] Converts PA to high VA, [M22] identity for device addresses
#[inline]
pub fn phys_to_virt(pa: usize) -> usize {
    if pa >= 0x4000_0000 {
        pa + KERNEL_VIRT_START  // [M20] PA to high VA
    } else {
        pa // [M22] identity for device addresses
    }
}

// TEAM_019: 2MB block mapping constants
/// 2MB block size (for L2 block mappings)
pub const BLOCK_2MB_SIZE: usize = 2 * 1024 * 1024;
/// 2MB block alignment mask
pub const BLOCK_2MB_MASK: usize = BLOCK_2MB_SIZE - 1;
/// 1GB block size (for L1 block mappings, future use)
pub const BLOCK_1GB_SIZE: usize = 1024 * 1024 * 1024;

// MAIR_EL1 configuration (from Theseus)
// Attr0: Normal memory (WriteBack, Non-Transient, ReadWriteAlloc)
// Attr1: Device memory (nGnRE)
#[allow(dead_code)]
const MAIR_ATTR_NORMAL: u64 = 0xFF; // Inner/Outer WriteBack
#[allow(dead_code)]
const MAIR_ATTR_DEVICE: u64 = 0x04; // Device-nGnRE
const MAIR_VALUE: u64 = MAIR_ATTR_NORMAL | (MAIR_ATTR_DEVICE << 8);

// TCR_EL1 configuration
// T0SZ = 16  => 48-bit VA (User/Identity)
// T1SZ = 16  => 48-bit VA (Kernel High)
// TG0 = 0b00 => 4KB granule
// TG1 = 0b10 => 4KB granule
// IPS = 0b101 => 48-bit PA
// SH0 = 0b11 => Inner Shareable (TTBR0)
// SH1 = 0b11 => Inner Shareable (TTBR1)
// ORGN0/IRGN0 = 0b01 => Write-Back Read-Allocate Write-Allocate Cacheable
// ORGN1/IRGN1 = 0b01 => Write-Back Read-Allocate Write-Allocate Cacheable
#[allow(dead_code)]
const TCR_T0SZ: u64 = 16;
#[allow(dead_code)]
const TCR_T1SZ: u64 = 16 << 16;
#[allow(dead_code)]
const TCR_TG0_4KB: u64 = 0b00 << 14;
#[allow(dead_code)]
const TCR_TG1_4KB: u64 = 0b10 << 30;
#[allow(dead_code)]
const TCR_IPS_48BIT: u64 = 0b101 << 32;
#[allow(dead_code)]
const TCR_SH0_INNER: u64 = 0b11 << 12;
#[allow(dead_code)]
const TCR_SH1_INNER: u64 = 0b11 << 28;
#[allow(dead_code)]
const TCR_ORGN0_WB_WA: u64 = 0b01 << 10;
#[allow(dead_code)]
const TCR_IRGN0_WB_WA: u64 = 0b01 << 8;
#[allow(dead_code)]
const TCR_ORGN1_WB_WA: u64 = 0b01 << 26;
#[allow(dead_code)]
const TCR_IRGN1_WB_WA: u64 = 0b01 << 24;

const TCR_VALUE: u64 = TCR_T0SZ
    | TCR_T1SZ
    | TCR_TG0_4KB
    | TCR_TG1_4KB
    | TCR_IPS_48BIT
    | TCR_SH0_INNER
    | TCR_SH1_INNER
    | TCR_ORGN0_WB_WA
    | TCR_IRGN0_WB_WA
    | TCR_ORGN1_WB_WA
    | TCR_IRGN1_WB_WA;

// ============================================================================
// Page Table Entry
// ============================================================================

/// A 64-bit page table entry.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Create an empty (invalid) entry.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Check if entry is valid.
    #[inline]
    pub fn is_valid(&self) -> bool {
        (self.0 & PageFlags::VALID.bits()) != 0
    }

    /// Check if entry is a table descriptor (vs block).
    #[inline]
    pub fn is_table(&self) -> bool {
        self.is_valid() && (self.0 & PageFlags::TABLE.bits()) != 0
    }

    /// Get the physical address from this entry.
    #[inline]
    pub fn address(&self) -> usize {
        (self.0 & 0x0000_FFFF_FFFF_F000) as usize
    }

    /// Set the entry with address and flags.
    #[inline]
    pub fn set(&mut self, addr: usize, flags: PageFlags) {
        self.0 = ((addr as u64) & 0x0000_FFFF_FFFF_F000) | flags.bits();
    }

    /// Clear the entry.
    #[inline]
    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

// ============================================================================
// Page Flags
// ============================================================================

bitflags! {
    /// AArch64 Stage 1 page table entry flags.
    /// Behaviors: [M1] VALID bit 0, [M2] TABLE bit 1, [M3] block has TABLE=0, [M4] table has TABLE=1
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PageFlags: u64 {
        /// [M1] Entry is valid (bit 0)
        const VALID       = 1 << 0;
        /// [M2] Table descriptor (bit 1) - [M3] blocks have this unset, [M4] tables have this set
        const TABLE       = 1 << 1;

        // MAIR index (AttrIndx[2:0] at bits [4:2])
        /// Normal memory (MAIR index 0)
        const ATTR_NORMAL = 0b000 << 2;
        /// Device memory (MAIR index 1)
        const ATTR_DEVICE = 0b001 << 2;

        /// Non-secure
        const NS          = 1 << 5;

        // Access Permissions (AP[2:1] at bits [7:6])
        /// R/W at EL1, none at EL0
        const AP_RW_EL1   = 0b00 << 6;
        /// R/W at all ELs
        const AP_RW_ALL   = 0b01 << 6;
        /// RO at EL1, none at EL0
        const AP_RO_EL1   = 0b10 << 6;
        /// RO at all ELs
        const AP_RO_ALL   = 0b11 << 6;

        // Shareability (SH[1:0] at bits [9:8])
        /// Inner Shareable
        const SH_INNER    = 0b11 << 8;

        /// Access Flag - must be set or HW will fault
        const AF          = 1 << 10;
        /// Not Global
        const NG          = 1 << 11;

        // Upper attributes
        /// Privileged Execute Never
        const PXN         = 1 << 53;
        /// User Execute Never
        const UXN         = 1 << 54;
    }
}

impl PageFlags {
    /// Standard flags for kernel code (executable, read-only)
    pub const KERNEL_CODE: PageFlags = PageFlags::VALID
        .union(PageFlags::AF)
        .union(PageFlags::SH_INNER)
        .union(PageFlags::AP_RO_EL1)
        .union(PageFlags::UXN);

    /// Standard flags for kernel data (read-write, not executable)
    pub const KERNEL_DATA: PageFlags = PageFlags::VALID
        .union(PageFlags::AF)
        .union(PageFlags::SH_INNER)
        .union(PageFlags::AP_RW_EL1)
        .union(PageFlags::PXN)
        .union(PageFlags::UXN);

    /// Standard flags for device memory (read-write, not executable, not cached)
    pub const DEVICE: PageFlags = PageFlags::VALID
        .union(PageFlags::AF)
        .union(PageFlags::ATTR_DEVICE)
        .union(PageFlags::AP_RW_EL1)
        .union(PageFlags::PXN)
        .union(PageFlags::UXN);

    // TEAM_019: Block descriptor flags (bits[1:0] = 0b01, no TABLE bit)
    /// Kernel data as 2MB block (VALID but NOT TABLE)
    pub const KERNEL_DATA_BLOCK: PageFlags = PageFlags::VALID
        .union(PageFlags::AF)
        .union(PageFlags::SH_INNER)
        .union(PageFlags::AP_RW_EL1)
        .union(PageFlags::PXN)
        .union(PageFlags::UXN);

    /// Kernel code as 2MB block (VALID but NOT TABLE, executable)
    pub const KERNEL_CODE_BLOCK: PageFlags = PageFlags::VALID
        .union(PageFlags::AF)
        .union(PageFlags::SH_INNER)
        .union(PageFlags::AP_RO_EL1)
        .union(PageFlags::UXN);

    /// Device memory as 2MB block (VALID but NOT TABLE)
    pub const DEVICE_BLOCK: PageFlags = PageFlags::VALID
        .union(PageFlags::AF)
        .union(PageFlags::ATTR_DEVICE)
        .union(PageFlags::AP_RW_EL1)
        .union(PageFlags::PXN)
        .union(PageFlags::UXN);
}

// ============================================================================
// Page Table
// ============================================================================

/// A 4KB-aligned page table with 512 entries.
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; ENTRIES_PER_TABLE],
}

impl PageTable {
    /// Create a new empty page table.
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::empty(); ENTRIES_PER_TABLE],
        }
    }

    /// Zero all entries.
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.clear();
        }
    }

    /// Get entry at index.
    #[inline]
    pub fn entry(&self, index: usize) -> &PageTableEntry {
        &self.entries[index]
    }

    /// Get mutable entry at index.
    #[inline]
    pub fn entry_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
}

// ============================================================================
// Virtual Address Indexing
// Behaviors: [M7]-[M10] VA index extraction
// ============================================================================

/// [M7] Extract L0 index from virtual address (bits [47:39])
#[inline]
pub fn va_l0_index(va: usize) -> usize {
    (va >> 39) & 0x1FF  // [M7]
}

/// [M8] Extract L1 index from virtual address (bits [38:30])
#[inline]
pub fn va_l1_index(va: usize) -> usize {
    (va >> 30) & 0x1FF  // [M8]
}

/// [M9] Extract L2 index from virtual address (bits [29:21])
#[inline]
pub fn va_l2_index(va: usize) -> usize {
    (va >> 21) & 0x1FF  // [M9]
}

/// [M10] Extract L3 index from virtual address (bits [20:12])
#[inline]
pub fn va_l3_index(va: usize) -> usize {
    (va >> 12) & 0x1FF  // [M10]
}

// ============================================================================
// TLB Flush (from Theseus patterns)
// ============================================================================

/// Flush all TLB entries.
#[cfg(target_arch = "aarch64")]
pub fn tlb_flush_all() {
    unsafe {
        core::arch::asm!("tlbi vmalle1", "dsb sy", "isb", options(nostack));
    }
}

#[cfg(not(target_arch = "aarch64"))]
pub fn tlb_flush_all() {
    // Stub for non-aarch64 builds (test builds on host)
}

/// Flush TLB entry for a specific virtual address.
#[cfg(target_arch = "aarch64")]
pub fn tlb_flush_page(va: usize) {
    unsafe {
        let value = va >> 12;
        core::arch::asm!(
            "tlbi vae1, {}",
            "dsb sy",
            "isb",
            in(reg) value,
            options(nostack)
        );
    }
}

#[cfg(not(target_arch = "aarch64"))]
pub fn tlb_flush_page(_va: usize) {
    // Stub for non-aarch64 builds (test builds on host)
}

// ============================================================================
// MMU Initialization
// ============================================================================

/// Initialize MMU registers (MAIR, TCR). Does NOT enable MMU.
#[cfg(target_arch = "aarch64")]
pub fn init() {
    unsafe {
        // Configure MAIR_EL1
        core::arch::asm!(
            "msr mair_el1, {}",
            in(reg) MAIR_VALUE,
            options(nostack)
        );

        // Configure TCR_EL1
        core::arch::asm!(
            "msr tcr_el1, {}",
            in(reg) TCR_VALUE,
            options(nostack)
        );

        // Barrier
        core::arch::asm!("isb", options(nostack));
    }
}

#[cfg(not(target_arch = "aarch64"))]
pub fn init() {
    // Stub for non-aarch64 builds (test builds on host)
}

/// Enable the MMU with both TTBR0 and TTBR1 root physical addresses.
///
/// # Safety
/// - `ttbr0_phys` and `ttbr1_phys` must point to valid page tables.
#[cfg(target_arch = "aarch64")]
pub unsafe fn enable_mmu(ttbr0_phys: usize, ttbr1_phys: usize) {
    unsafe {
        // Load TTBR0_EL1 and TTBR1_EL1
        core::arch::asm!(
            "msr ttbr0_el1, {}",
            "msr ttbr1_el1, {}",
            "isb",
            in(reg) ttbr0_phys,
            in(reg) ttbr1_phys,
            options(nostack)
        );

        // Read SCTLR_EL1
        let mut sctlr: u64;
        core::arch::asm!(
            "mrs {}, sctlr_el1",
            out(reg) sctlr,
            options(nostack)
        );

        // Set M bit (enable MMU)
        sctlr |= 1;

        // Write SCTLR_EL1
        core::arch::asm!(
            "msr sctlr_el1, {}",
            "isb",
            in(reg) sctlr,
            options(nostack)
        );
    }
}

#[cfg(not(target_arch = "aarch64"))]
pub unsafe fn enable_mmu(_ttbr0_phys: usize, _ttbr1_phys: usize) {
    // Stub for non-aarch64 builds (test builds on host)
}

/// Disable the MMU.
#[cfg(target_arch = "aarch64")]
pub unsafe fn disable_mmu() {
    unsafe {
        let mut sctlr: u64;
        core::arch::asm!(
            "mrs {}, sctlr_el1",
            out(reg) sctlr,
            options(nostack)
        );

        sctlr &= !1; // Clear M bit

        core::arch::asm!(
            "msr sctlr_el1, {}",
            "isb",
            in(reg) sctlr,
            options(nostack)
        );
    }
}

#[cfg(not(target_arch = "aarch64"))]
pub unsafe fn disable_mmu() {
    // Stub for non-aarch64 builds (test builds on host)
}

// ============================================================================
// Static Page Table Pool (for early boot before heap is available)
// ============================================================================

/// Static pool of page tables for early boot.
/// TEAM_019: With 2MB block mappings, we need far fewer tables:
/// - 1 L0 + 1-2 L1 + 1-2 L2 = ~4 tables for blocks
/// - Plus a few L3 tables for unaligned boundaries
/// - 16 tables provides ample safety margin
static mut PT_POOL: [PageTable; 16] = [const { PageTable::new() }; 16];
static mut PT_POOL_NEXT: usize = 0;

/// Allocate a page table from the static pool.
/// Returns None if pool is exhausted.
fn alloc_page_table() -> Option<&'static mut PageTable> {
    // SAFETY: Single-threaded boot context, no concurrent access
    unsafe {
        let pool_ptr = core::ptr::addr_of_mut!(PT_POOL);
        let next_ptr = core::ptr::addr_of_mut!(PT_POOL_NEXT);
        let next = *next_ptr;
        let pool_len = (*pool_ptr).len();

        if next >= pool_len {
            return None;
        }

        let pt = &mut (*pool_ptr)[next];
        *next_ptr = next + 1;
        pt.zero();
        Some(pt)
    }
}

// ============================================================================
// Page Table Mapping
// ============================================================================

/// Map a single 4KB page.
///
/// Creates intermediate table entries as needed.
/// Returns Err if page table allocation fails.
pub fn map_page(
    root: &mut PageTable,
    va: usize,
    pa: usize,
    flags: PageFlags,
) -> Result<(), &'static str> {
    // Get indices at each level
    let l0_idx = va_l0_index(va);
    let l1_idx = va_l1_index(va);
    let l2_idx = va_l2_index(va);
    let l3_idx = va_l3_index(va);

    // Walk L0 -> L1
    let l1_table = get_or_create_table(root, l0_idx)?;

    // Walk L1 -> L2
    let l2_table = get_or_create_table(l1_table, l1_idx)?;

    // Walk L2 -> L3
    let l3_table = get_or_create_table(l2_table, l2_idx)?;

    // Set L3 entry (4KB page)
    let entry = l3_table.entry_mut(l3_idx);
    entry.set(pa, flags | PageFlags::TABLE); // L3 entries use TABLE bit = 1 for pages

    Ok(())
}

/// Get or create a child table at the given index.
fn get_or_create_table(
    parent: &mut PageTable,
    index: usize,
) -> Result<&'static mut PageTable, &'static str> {
    let entry = parent.entry(index);

    if entry.is_table() {
        // Entry exists, get the child table address (Physical)
        let child_pa = entry.address();
        // Convert PA to VA for Rust access
        let child_va = phys_to_virt(child_pa);
        unsafe { Ok(&mut *(child_va as *mut PageTable)) }
    } else {
        // Need to allocate a new table
        let new_table = alloc_page_table().ok_or("Page table pool exhausted")?;
        let new_va = new_table as *mut PageTable as usize;
        let new_pa = virt_to_phys(new_va);

        // Set parent entry to point to new table (Physical Address)
        parent
            .entry_mut(index)
            .set(new_pa, PageFlags::VALID | PageFlags::TABLE);

        Ok(new_table)
    }
}

/// Identity map a range of physical addresses (VA == PA).
pub fn identity_map_range(
    root: &mut PageTable,
    start: usize,
    end: usize,
    flags: PageFlags,
) -> Result<(), &'static str> {
    let start_page = start & !0xFFF;
    let end_page = (end + 0xFFF) & !0xFFF;

    let mut addr = start_page;
    while addr < end_page {
        map_page(root, addr, addr, flags)?;
        addr += PAGE_SIZE;
    }

    Ok(())
}

// TEAM_019: 2MB Block Mapping Support
// ============================================================================

/// Map a single 2MB block at L2 level.
///
/// # Arguments
/// - `root`: L0 page table
/// - `va`: Virtual address (must be 2MB aligned)
/// - `pa`: Physical address (must be 2MB aligned)
/// - `flags`: Page flags (should use KERNEL_DATA_BLOCK or DEVICE_BLOCK)
///
/// # Returns
/// Ok(()) on success, Err if allocation fails or misaligned
pub fn map_block_2mb(
    root: &mut PageTable,
    va: usize,
    pa: usize,
    flags: PageFlags,
) -> Result<(), &'static str> {
    // Verify 2MB alignment
    if (va & BLOCK_2MB_MASK) != 0 {
        return Err("VA not 2MB aligned for block mapping");
    }
    if (pa & BLOCK_2MB_MASK) != 0 {
        return Err("PA not 2MB aligned for block mapping");
    }

    // Get indices
    let l0_idx = va_l0_index(va);
    let l1_idx = va_l1_index(va);
    let l2_idx = va_l2_index(va);

    // Walk L0 -> L1
    let l1_table = get_or_create_table(root, l0_idx)?;

    // Walk L1 -> L2
    let l2_table = get_or_create_table(l1_table, l1_idx)?;

    // Set L2 entry as BLOCK (not TABLE)
    // Block descriptor: bits[1:0] = 0b01 (VALID, not TABLE)
    let entry = l2_table.entry_mut(l2_idx);
    entry.set(pa, flags);

    Ok(())
}

/// Map a range using 2MB blocks where possible, otherwise 4KB pages.
pub fn map_range(
    root: &mut PageTable,
    va_start: usize,
    pa_start: usize,
    len: usize,
    flags: PageFlags,
) -> Result<MappingStats, &'static str> {
    let mut va = va_start & !0xFFF;
    let mut pa = pa_start & !0xFFF;
    let end_va = (va_start + len + 0xFFF) & !0xFFF;

    let mut stats = MappingStats {
        blocks_2mb: 0,
        pages_4kb: 0,
    };

    while va < end_va {
        let remaining = end_va - va;

        // Check if we can use 2MB block:
        // 1. Both VA and PA are 2MB aligned
        // 2. At least 2MB remaining
        if (va & BLOCK_2MB_MASK) == 0 && (pa & BLOCK_2MB_MASK) == 0 && remaining >= BLOCK_2MB_SIZE {
            let block_flags = flags.difference(PageFlags::TABLE);
            map_block_2mb(root, va, pa, block_flags)?;
            stats.blocks_2mb += 1;
            va += BLOCK_2MB_SIZE;
            pa += BLOCK_2MB_SIZE;
        } else {
            map_page(root, va, pa, flags)?;
            stats.pages_4kb += 1;
            va += PAGE_SIZE;
            pa += PAGE_SIZE;
        }
    }

    Ok(stats)
}

/// Identity map a range using 2MB blocks where possible, otherwise 4KB pages.
pub fn identity_map_range_optimized(
    root: &mut PageTable,
    start: usize,
    end: usize,
    flags: PageFlags,
) -> Result<MappingStats, &'static str> {
    map_range(root, start, start, end - start, flags)
}

/// Statistics from an optimized identity mapping operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappingStats {
    /// Number of 2MB blocks mapped
    pub blocks_2mb: usize,
    /// Number of 4KB pages mapped
    pub pages_4kb: usize,
}

impl MappingStats {
    /// Total bytes mapped
    pub fn total_bytes(&self) -> usize {
        self.blocks_2mb * BLOCK_2MB_SIZE + self.pages_4kb * PAGE_SIZE
    }
}

// ============================================================================
// Unit Tests (TEAM_019)
// Gated on `std` feature because this is a no_std crate.
// Run with: cargo test -p levitate-hal --features std
// ============================================================================

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    // === Flag Construction Tests ===

    #[test]
    fn test_page_flags_block_vs_table() {
        // Block descriptor: bits[1:0] = 0b01 (VALID only)
        let block = PageFlags::VALID;
        assert_eq!(block.bits() & 0b11, 0b01);

        // Table descriptor: bits[1:0] = 0b11 (VALID | TABLE)
        let table = PageFlags::VALID.union(PageFlags::TABLE);
        assert_eq!(table.bits() & 0b11, 0b11);
    }

    #[test]
    fn test_block_flags_no_table_bit() {
        let block = PageFlags::KERNEL_DATA_BLOCK;
        assert!(!block.contains(PageFlags::TABLE));
        assert!(block.contains(PageFlags::VALID));
        assert!(block.contains(PageFlags::AF));
        assert!(block.contains(PageFlags::PXN));
        assert!(block.contains(PageFlags::UXN));
    }

    #[test]
    fn test_device_block_flags() {
        let block = PageFlags::DEVICE_BLOCK;
        assert!(!block.contains(PageFlags::TABLE));
        assert!(block.contains(PageFlags::VALID));
        assert!(block.contains(PageFlags::ATTR_DEVICE));
    }

    // === Address Index Extraction Tests ===

    #[test]
    fn test_va_l0_index() {
        assert_eq!(va_l0_index(0x0000_0000_0000_0000), 0);
        assert_eq!(va_l0_index(0x0000_0080_0000_0000), 1); // 512GB boundary
        assert_eq!(va_l0_index(0x0000_FF80_0000_0000), 511);
    }

    #[test]
    fn test_va_l1_index() {
        assert_eq!(va_l1_index(0x0000_0000_0000_0000), 0);
        assert_eq!(va_l1_index(0x0000_0000_4000_0000), 1); // 1GB boundary
        assert_eq!(va_l1_index(0x0000_0000_8000_0000), 2);
    }

    #[test]
    fn test_va_l2_index() {
        assert_eq!(va_l2_index(0x0000_0000_0000_0000), 0);
        assert_eq!(va_l2_index(0x0000_0000_0020_0000), 1); // 2MB boundary
        assert_eq!(va_l2_index(0x0000_0000_0040_0000), 2);
    }

    #[test]
    fn test_va_l3_index() {
        assert_eq!(va_l3_index(0x0000_0000_0000_0000), 0);
        assert_eq!(va_l3_index(0x0000_0000_0000_1000), 1); // 4KB boundary
        assert_eq!(va_l3_index(0x0000_0000_0000_2000), 2);
    }

    #[test]
    fn test_kernel_address_indices() {
        // Kernel start: 0x4008_0000
        let va = 0x4008_0000usize;
        assert_eq!(va_l0_index(va), 0); // Within first 512GB
        assert_eq!(va_l1_index(va), 1); // Second 1GB region
        assert_eq!(va_l2_index(va), 0); // First 2MB within that 1GB
        // Note: 0x4008_0000 is NOT 2MB aligned (0x0008_0000 = 512KB offset)
    }

    // === Alignment Tests ===

    #[test]
    fn test_block_alignment() {
        // 2MB aligned addresses
        assert_eq!(0x4000_0000 & BLOCK_2MB_MASK, 0); // 1GB is 2MB aligned
        assert_eq!(0x4020_0000 & BLOCK_2MB_MASK, 0); // 2MB aligned
        assert_eq!(0x4040_0000 & BLOCK_2MB_MASK, 0); // 4MB aligned

        // NOT 2MB aligned
        assert_ne!(0x4010_0000 & BLOCK_2MB_MASK, 0); // 1MB offset
        assert_ne!(0x4008_0000 & BLOCK_2MB_MASK, 0); // 512KB offset (kernel start)
        assert_ne!(0x4001_0000 & BLOCK_2MB_MASK, 0); // 64KB offset
    }

    #[test]
    fn test_constants() {
        assert_eq!(BLOCK_2MB_SIZE, 0x0020_0000);
        assert_eq!(BLOCK_2MB_MASK, 0x001F_FFFF);
        assert_eq!(PAGE_SIZE, 0x1000);
    }

    // === Page Table Entry Tests ===

    #[test]
    fn test_page_table_entry_empty() {
        let entry = PageTableEntry::empty();
        assert!(!entry.is_valid());
        assert!(!entry.is_table());
        assert_eq!(entry.address(), 0);
    }

    #[test]
    fn test_page_table_entry_set_block() {
        let mut entry = PageTableEntry::empty();
        entry.set(0x4000_0000, PageFlags::KERNEL_DATA_BLOCK);
        assert!(entry.is_valid());
        assert!(!entry.is_table()); // Block, not table
        assert_eq!(entry.address(), 0x4000_0000);
    }

    #[test]
    fn test_page_table_entry_set_table() {
        let mut entry = PageTableEntry::empty();
        entry.set(0x4000_0000, PageFlags::VALID.union(PageFlags::TABLE));
        assert!(entry.is_valid());
        assert!(entry.is_table());
        assert_eq!(entry.address(), 0x4000_0000);
    }

    // === Mapping Calculation Tests ===

    #[test]
    fn test_table_count_for_block_mapping() {
        // Calculate expected blocks for 128MB kernel with block mapping
        // Start: 0x4008_0000 (not 2MB aligned)
        // End:   0x4800_0000

        let start = 0x4008_0000usize;
        let end = 0x4800_0000usize;

        // First 2MB-aligned address at or after start
        let first_block_aligned = (start + BLOCK_2MB_SIZE - 1) & !BLOCK_2MB_MASK;
        assert_eq!(first_block_aligned, 0x4020_0000);

        // Last 2MB-aligned address at or before end
        let last_block_aligned = end & !BLOCK_2MB_MASK;
        assert_eq!(last_block_aligned, 0x4800_0000);

        // Number of 2MB blocks
        let num_blocks = (last_block_aligned - first_block_aligned) / BLOCK_2MB_SIZE;
        assert_eq!(num_blocks, 63); // 63 blocks of 2MB = 126MB

        // Leading edge: 0x4008_0000 to 0x4020_0000 = 0x18_0000 = 1.5MB
        let leading_bytes = first_block_aligned - start;
        assert_eq!(leading_bytes, 0x18_0000); // 1.5MB
        let leading_pages = leading_bytes / PAGE_SIZE;
        assert_eq!(leading_pages, 384); // 384 x 4KB pages
    }

    #[test]
    fn test_mapping_stats() {
        let stats = MappingStats {
            blocks_2mb: 63,
            pages_4kb: 384,
        };

        // 63 * 2MB + 384 * 4KB = 126MB + 1.5MB = 127.5MB
        let expected_bytes = 63 * BLOCK_2MB_SIZE + 384 * PAGE_SIZE;
        assert_eq!(stats.total_bytes(), expected_bytes);
        assert_eq!(expected_bytes, 0x7F8_0000); // ~127.5MB
    }

    // === TEAM_030: Address Translation Tests (M19-M22) ===

    // M19: virt_to_phys converts high VA to PA
    #[test]
    fn test_virt_to_phys_high_address() {
        // Kernel virtual address in higher half
        let va = KERNEL_VIRT_START + 0x4008_0000;
        let pa = virt_to_phys(va);
        assert_eq!(pa, 0x4008_0000);
    }

    // M20: phys_to_virt converts PA to high VA
    #[test]
    fn test_phys_to_virt_kernel_region() {
        // Physical address in kernel region (>= 0x4000_0000)
        let pa = 0x4008_0000;
        let va = phys_to_virt(pa);
        assert_eq!(va, KERNEL_VIRT_START + 0x4008_0000);
    }

    // M21: virt_to_phys identity for low addresses
    #[test]
    fn test_virt_to_phys_low_address_identity() {
        // Low addresses (below KERNEL_VIRT_START) pass through unchanged
        let va = 0x4008_0000;
        let pa = virt_to_phys(va);
        assert_eq!(pa, va); // Identity: already physical
    }

    // M22: phys_to_virt identity for device addresses
    #[test]
    fn test_phys_to_virt_device_identity() {
        // Device addresses (< 0x4000_0000) use identity mapping
        let pa = 0x0900_0000; // UART address
        let va = phys_to_virt(pa);
        assert_eq!(va, pa); // Identity: device region
    }
}
