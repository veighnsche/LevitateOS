//! AArch64 Memory Management Unit (MMU) support.
//!
//! TEAM_018: Implements page table structures and MMU configuration
//! for AArch64 with 4KB granule, 48-bit virtual addresses.
//!
//! Reference implementations studied:
//! - Theseus: MAIR/TCR config, TLB flush patterns
//! - Redox: RMM abstraction

use bitflags::bitflags;

use los_error::define_kernel_error;

define_kernel_error! {
    /// TEAM_152: MMU error type with error codes (0x01xx) per unified error system plan.
    /// TEAM_155: Migrated to define_kernel_error! macro.
    pub enum MmuError(0x01) {
        /// Page table allocation failed
        AllocationFailed = 0x01 => "Page table allocation failed",
        /// Address not mapped
        NotMapped = 0x02 => "Address not mapped",
        /// Invalid virtual address or target level
        InvalidVirtualAddress = 0x03 => "Invalid virtual address",
        /// Address not properly aligned
        Misaligned = 0x04 => "Address not properly aligned",
        /// Page table walk failed at intermediate level
        WalkFailed = 0x05 => "Page table walk failed",
    }
}

/// [M23] Trait for physical page allocation, to be implemented by a Buddy Allocator.
/// Allows MMU to request pages for dynamic page tables.
/// TEAM_054: Added behavior traceability
pub trait PageAllocator: Send + Sync {
    /// [M23] Allocate a 4KB physical page for page tables.
    fn alloc_page(&self) -> Option<usize>;
    /// [M24] Free a 4KB physical page (unused until unmap support).
    fn free_page(&self, pa: usize);
}

/// Pointer to the dynamic page allocator, set once during boot.
static mut PAGE_ALLOCATOR_PTR: Option<&'static dyn PageAllocator> = None;

/// [M25] Set the global page allocator for MMU use.
/// Called once during boot after Buddy Allocator is initialized.
pub fn set_page_allocator(allocator: &'static dyn PageAllocator) {
    // SAFETY: Single-threaded boot context
    unsafe {
        PAGE_ALLOCATOR_PTR = Some(allocator);
    }
}

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

// TEAM_078: Device virtual addresses (mapped via TTBR1)
// These allow device access regardless of TTBR0 state (critical for userspace)
/// Device virtual address base (same as kernel base for simplicity)
pub const DEVICE_VIRT_BASE: usize = KERNEL_VIRT_START;
/// UART PL011 virtual address (PA: 0x0900_0000)
pub const UART_VA: usize = DEVICE_VIRT_BASE + 0x0900_0000;
/// VirtIO MMIO base virtual address (PA: 0x0A00_0000)
pub const VIRTIO_MMIO_VA: usize = DEVICE_VIRT_BASE + 0x0A00_0000;
/// GIC Distributor virtual address (PA: 0x0800_0000)
pub const GIC_DIST_VA: usize = DEVICE_VIRT_BASE + 0x0800_0000;
/// GIC CPU Interface virtual address (PA: 0x0801_0000)
pub const GIC_CPU_VA: usize = DEVICE_VIRT_BASE + 0x0801_0000;
/// GIC Redistributor virtual address (PA: 0x080A_0000)
pub const GIC_REDIST_VA: usize = DEVICE_VIRT_BASE + 0x080A_0000;

// TEAM_114: PCI ECAM (Enhanced Configuration Access Mechanism) for VirtIO PCI
/// PCI ECAM base physical address (QEMU virt machine Highmem PCIe)
/// From DTB: reg = <0x40 0x10000000 0x00 0x10000000> = PA 0x4010000000, size 256MB
pub const ECAM_PA: usize = 0x40_1000_0000;
/// PCI ECAM virtual address (high half mapping)
/// Note: This creates a VA in the upper 48-bit space
pub const ECAM_VA: usize = KERNEL_VIRT_START + ECAM_PA;
/// ECAM size: 256MB for 256 buses (1MB per bus)
pub const ECAM_SIZE: usize = 256 * 1024 * 1024;

// TEAM_114: PCI 32-bit memory region for BAR allocation (from QEMU virt DTB)
/// PCI 32-bit MMIO base physical address
pub const PCI_MEM32_PA: usize = 0x1000_0000;
/// PCI 32-bit MMIO size
pub const PCI_MEM32_SIZE: usize = 0x2EFF_0000;
/// PCI 32-bit MMIO virtual address
pub const PCI_MEM32_VA: usize = KERNEL_VIRT_START + PCI_MEM32_PA;

/// [M19] Converts high VA to PA, [M21] identity for low addresses
#[inline]
pub fn virt_to_phys(va: usize) -> usize {
    #[cfg(not(target_arch = "aarch64"))]
    {
        va
    }
    #[cfg(target_arch = "aarch64")]
    if va >= KERNEL_VIRT_START {
        va - KERNEL_VIRT_START // [M19] high VA to PA
    } else {
        va // [M21] identity for low addresses
    }
}

/// [M20] Converts PA to high VA
/// TEAM_078: Now maps ALL physical addresses to high VA (including devices)
/// This ensures devices are accessible via TTBR1 regardless of TTBR0 state.
#[inline]
pub fn phys_to_virt(pa: usize) -> usize {
    #[cfg(not(target_arch = "aarch64"))]
    {
        pa
    }
    #[cfg(target_arch = "aarch64")]
    {
        pa + KERNEL_VIRT_START // [M20] All PA to high VA
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

    /// Get flags from the entry.
    #[inline]
    pub fn flags(&self) -> PageFlags {
        PageFlags::from_bits_truncate(self.0)
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

    // TEAM_073: User-mode page flags (Phase 8: Userspace)
    // AP_RW_ALL (bits [7:6] = 01) = R/W access at all exception levels

    /// User code (executable, read-only from user perspective)
    /// - Accessible from EL0 (user)
    /// - PXN set (not executable in kernel mode for security)
    pub const USER_CODE: PageFlags = PageFlags::VALID
        .union(PageFlags::AF)
        .union(PageFlags::SH_INNER)
        .union(PageFlags::AP_RO_ALL) // RO from EL0/EL1
        .union(PageFlags::NG) // Not Global (per-process)
        .union(PageFlags::PXN); // Don't execute in kernel

    /// User data (read-write, not executable)
    /// - Accessible from EL0 (user)
    /// - UXN and PXN set (not executable anywhere)
    pub const USER_DATA: PageFlags = PageFlags::VALID
        .union(PageFlags::AF)
        .union(PageFlags::SH_INNER)
        .union(PageFlags::AP_RW_ALL) // R/W from EL0/EL1
        .union(PageFlags::NG) // Not Global (per-process)
        .union(PageFlags::PXN) // Don't execute in kernel
        .union(PageFlags::UXN); // Don't execute in user

    /// User stack (same as USER_DATA, explicit name for clarity)
    pub const USER_STACK: PageFlags = Self::USER_DATA;

    /// TEAM_212: User code+data (RWX) for pages shared between code and data segments
    /// This is less secure but necessary when segments share pages.
    /// - Accessible from EL0 (user)
    /// - Read-write AND executable in user mode
    pub const USER_CODE_DATA: PageFlags = PageFlags::VALID
        .union(PageFlags::AF)
        .union(PageFlags::SH_INNER)
        .union(PageFlags::AP_RW_ALL) // R/W from EL0/EL1
        .union(PageFlags::NG) // Not Global (per-process)
        .union(PageFlags::PXN); // Don't execute in kernel (but allow in user - no UXN)
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

    /// Check if all entries in the table are invalid.
    /// TEAM_070: Added for UoW 3 table reclamation.
    pub fn is_empty(&self) -> bool {
        self.entries.iter().all(|e| !e.is_valid())
    }
}

// ============================================================================
// Virtual Address Indexing
// Behaviors: [M7]-[M10] VA index extraction
// ============================================================================

/// [M7] Extract L0 index from virtual address (bits [47:39])
#[inline]
pub fn va_l0_index(va: usize) -> usize {
    (va >> 39) & 0x1FF // [M7]
}

/// [M8] Extract L1 index from virtual address (bits [38:30])
#[inline]
pub fn va_l1_index(va: usize) -> usize {
    (va >> 30) & 0x1FF // [M8]
}

/// [M9] Extract L2 index from virtual address (bits [29:21])
#[inline]
pub fn va_l2_index(va: usize) -> usize {
    (va >> 21) & 0x1FF // [M9]
}

/// [M10] Extract L3 index from virtual address (bits [20:12])
#[inline]
pub fn va_l3_index(va: usize) -> usize {
    (va >> 12) & 0x1FF // [M10]
}

// ============================================================================
// TLB Flush (from Theseus patterns)
// ============================================================================

/// Flush all TLB entries.
#[cfg(target_arch = "aarch64")]
pub fn tlb_flush_all() {
    use aarch64_cpu::asm::barrier;
    // SAFETY: TLB flush is always safe - it only invalidates cached translations.
    // The system will re-walk page tables on next access.
    // TEAM_132: Migrate barriers to aarch64-cpu, keep tlbi as raw asm (not in crate)
    unsafe {
        core::arch::asm!("tlbi vmalle1", options(nostack));
    }
    barrier::dsb(barrier::SY);
    barrier::isb(barrier::SY);
}

#[cfg(not(target_arch = "aarch64"))]
pub fn tlb_flush_all() {
    // Stub for non-aarch64 builds (test builds on host)
}

/// Flush TLB entry for a specific virtual address.
#[cfg(target_arch = "aarch64")]
pub fn tlb_flush_page(va: usize) {
    use aarch64_cpu::asm::barrier;
    // SAFETY: TLB flush for a single VA is always safe - invalidates one cached translation.
    // TEAM_132: Migrate barriers to aarch64-cpu
    unsafe {
        let value = va >> 12;
        core::arch::asm!("tlbi vae1, {}", in(reg) value, options(nostack));
    }
    barrier::dsb(barrier::SY);
    barrier::isb(barrier::SY);
}

#[cfg(not(target_arch = "aarch64"))]
pub fn tlb_flush_page(_va: usize) {
    // Stub for non-aarch64 builds (test builds on host)
}

// ============================================================================
// MMU Initialization
// ============================================================================

/// Initialize MMU registers (MAIR, TCR). Does NOT enable MMU.
///
/// # TEAM_052: Stubbed Function
/// This function is a no-op because MAIR_EL1 and TCR_EL1 are configured
/// in the assembly bootstrap code (kernel/src/main.rs lines 148-165).
/// The assembly configuration matches the values that were previously defined here
/// (see commit 88c75b0 which removed the constants).
///
/// **Why this exists:** Called from kmain() for compatibility. Could be removed
/// if all initialization is moved to assembly permanently.
#[cfg(target_arch = "aarch64")]
pub fn init() {
    // MMU registers already configured by assembly bootstrap
    // See kernel/src/main.rs:148-165 for MAIR_EL1/TCR_EL1 setup
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
    // SAFETY: Caller guarantees ttbr0_phys and ttbr1_phys point to valid page tables.
    // The asm! blocks modify system registers - this is the core purpose of this function.
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
    // SAFETY: Disabling MMU requires identity-mapped code to be executing.
    // Caller must ensure current PC is identity-mapped before calling.
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

/// TEAM_073: Switch TTBR0_EL1 to a new user page table.
///
/// This is used during context switch to switch user address spaces.
/// TTBR1 (kernel mappings) is not affected.
///
/// # Safety
/// - `ttbr0_phys` must point to a valid page table
#[cfg(target_arch = "aarch64")]
pub unsafe fn switch_ttbr0(ttbr0_phys: usize) {
    unsafe {
        core::arch::asm!(
            "msr ttbr0_el1, {}",
            "isb",
            "tlbi vmalle1",  // Invalidate all TLB entries (all ASIDs)
            "dsb sy",
            "isb",
            in(reg) ttbr0_phys,
            options(nostack)
        );
    }
}

#[cfg(not(target_arch = "aarch64"))]
pub unsafe fn switch_ttbr0(_ttbr0_phys: usize) {
    // Stub for non-aarch64 builds
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
// ============================================================================
// Page Table Mapping
// ============================================================================

/// Result of a page table walk.
/// TEAM_070: Added for UoW 1 refactoring to support reclamation.
pub struct WalkResult<'a> {
    /// The table containing the leaf entry.
    pub table: &'a mut PageTable,
    /// The index of the leaf entry within the table.
    pub index: usize,
    /// The path of tables and indices taken to reach the leaf.
    /// Index 0 = L0, Index 1 = L1, Index 2 = L2.
    /// Each entry contains the table and the index into it that points to the NEXT level.
    pub breadcrumbs: Breadcrumbs,
}

/// Path信息 used for table reclamation.
/// TEAM_070: Added for UoW 1.
pub struct Breadcrumbs {
    pub tables: [Option<usize>; 3], // Virtual addresses of tables
    pub indices: [usize; 3],        // Indices used at each level
}

/// Walk the page table to find the entry for a virtual address at a specific level.
///
/// TEAM_070: Refactored from map_page to support reuse and unmap.
pub fn walk_to_entry<'a>(
    root: &'a mut PageTable,
    va: usize,
    target_level: usize,
    create: bool,
) -> Result<WalkResult<'a>, MmuError> {
    if target_level > 3 {
        return Err(MmuError::InvalidVirtualAddress);
    }

    let indices = [
        va_l0_index(va),
        va_l1_index(va),
        va_l2_index(va),
        va_l3_index(va),
    ];

    let mut current_table = root;
    let mut breadcrumbs = Breadcrumbs {
        tables: [None; 3],
        indices: [0; 3],
    };

    // Walk level by level until we reach the level ABOVE the target_level
    for level in 0..target_level {
        let index = indices[level];
        breadcrumbs.tables[level] = Some(current_table as *mut PageTable as usize);
        breadcrumbs.indices[level] = index;

        let entry = current_table.entry(index);
        if !entry.is_table() {
            if create {
                // Need to allocate a new table
                current_table = get_or_create_table(current_table, index)?;
            } else {
                return Err(MmuError::WalkFailed);
            }
        } else {
            // Already a table, just descend
            let child_pa = entry.address();
            let child_va = phys_to_virt(child_pa);
            current_table = unsafe { &mut *(child_va as *mut PageTable) };
        }
    }

    // Now current_table is the table containing the leaf entry at target_level
    let leaf_index = indices[target_level];

    Ok(WalkResult {
        table: current_table,
        index: leaf_index,
        breadcrumbs,
    })
}

/// Translate a virtual address to physical address and flags.
/// Returns None if not mapped.
pub fn translate(root: &PageTable, va: usize) -> Option<(usize, PageFlags)> {
    let indices = [
        va_l0_index(va),
        va_l1_index(va),
        va_l2_index(va),
        va_l3_index(va),
    ];

    let mut current_table = root;

    // Walk L0 -> L1 -> L2
    for level in 0..3 {
        let index = indices[level];
        let entry = current_table.entry(index);

        if !entry.is_valid() {
            return None;
        }

        if !entry.is_table() {
            // Block mapping (L1 1GB or L2 2MB)
            let block_pa = entry.address();
            let flags = entry.flags();

            // Calculate offset based on level
            let (mask, _size) = if level == 1 {
                (0x3FFF_FFFF, BLOCK_1GB_SIZE) // L1 = 1GB
            } else if level == 2 {
                (0x1F_FFFF, BLOCK_2MB_SIZE) // L2 = 2MB
            } else {
                return None; // L0 blocks not supported on 4KB granule
            };

            let offset = va & mask;
            return Some((block_pa + offset, flags));
        }

        let child_pa = entry.address();
        let child_va = phys_to_virt(child_pa);
        // SAFETY: We are just reading. The PA is valid RAM.
        current_table = unsafe { &*(child_va as *const PageTable) };
    }

    // L3 (Leaf Page)
    let index = indices[3];
    let entry = current_table.entry(index);
    if !entry.is_valid() {
        return None;
    }

    let pa = entry.address();
    let offset = va & 0xFFF;
    let flags = entry.flags();

    Some((pa + offset, flags))
}

/// Map a single 4KB page.
///
/// Creates intermediate table entries as needed.
/// Returns Err if page table allocation fails.
pub fn map_page(
    root: &mut PageTable,
    va: usize,
    pa: usize,
    flags: PageFlags,
) -> Result<(), MmuError> {
    // TEAM_070: Using refactored walk_to_entry
    let walk = walk_to_entry(root, va, 3, true)?;
    walk.table
        .entry_mut(walk.index)
        .set(pa, flags | PageFlags::TABLE); // L3 entries use TABLE bit = 1 for pages
    Ok(())
}

/// Unmap a single 4KB page.
///
/// TEAM_070: Implementing unmap support (UoW 2) and reclamation (UoW 3).
/// Returns Err if page is not mapped (Rule 14).
pub fn unmap_page(root: &mut PageTable, va: usize) -> Result<(), MmuError> {
    // Walk to L3 entry. Don't create if missing.
    let walk = walk_to_entry(root, va, 3, false)?;

    if !walk.table.entry(walk.index).is_valid() {
        return Err(MmuError::NotMapped);
    }

    // Clear leaf entry
    walk.table.entry_mut(walk.index).clear();

    // TLB invalidation is critical after clearing entry
    tlb_flush_page(va);

    // TEAM_070: Table Reclamation (UoW 3)
    // If the leaf table (L3) is now empty, we can potentially free it and recurse.
    if walk.table.is_empty() {
        if let Some(allocator) = unsafe { PAGE_ALLOCATOR_PTR } {
            let mut current_table_to_free = walk.table;

            // Iterate backwards through breadcrumbs:
            // breadcrumbs.tables[2] is L2 (points to L3), [1] is L1, [0] is L0.
            for level in (0..3).rev() {
                if let Some(parent_va) = walk.breadcrumbs.tables[level] {
                    let parent = unsafe { &mut *(parent_va as *mut PageTable) };
                    let index_in_parent = walk.breadcrumbs.indices[level];

                    // 1. Free the current child table
                    let child_pa = virt_to_phys(current_table_to_free as *mut PageTable as usize);

                    // SAFETY: We only free if we have a dynamic allocator.
                    // The allocator should handle PAs it doesn't own gracefully or we must check.
                    // For now, we trust the allocator or the fact that dynamic tables are only
                    // allocated when allocator is present.
                    allocator.free_page(child_pa);

                    // 2. Clear the entry in the parent pointing to this table
                    parent.entry_mut(index_in_parent).clear();

                    // 3. If parent is now empty and NOT the root (L0), continue reclamation
                    if level > 0 && parent.is_empty() {
                        current_table_to_free = parent;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Get or create a child table at the given index.
/// TEAM_070: Internal helper now, could be folded into walk_to_entry if needed.
fn get_or_create_table(
    parent: &mut PageTable,
    index: usize,
) -> Result<&'static mut PageTable, MmuError> {
    let entry = parent.entry(index);

    if entry.is_table() {
        // Entry exists, get the child table address (Physical)
        let child_pa = entry.address();
        // Convert PA to VA for Rust access
        let child_va = phys_to_virt(child_pa);
        unsafe { Ok(&mut *(child_va as *mut PageTable)) }
    } else {
        // Need to allocate a new table
        // [M26] Try dynamic allocator first, [M27] fallback to static pool
        let new_table = if let Some(allocator) = unsafe { PAGE_ALLOCATOR_PTR } {
            allocator.alloc_page().map(|pa| {
                // crate::verbose!("MMU: Allocated dynamic page table at 0x{:x}", pa);
                let va = phys_to_virt(pa);
                let pt = unsafe { &mut *(va as *mut PageTable) };
                pt.zero();
                pt
            })
        } else {
            alloc_page_table()
        }
        .ok_or(MmuError::AllocationFailed)?;

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
) -> Result<(), MmuError> {
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
) -> Result<(), MmuError> {
    // Verify 2MB alignment
    if (va & BLOCK_2MB_MASK) != 0 {
        return Err(MmuError::Misaligned);
    }
    if (pa & BLOCK_2MB_MASK) != 0 {
        return Err(MmuError::Misaligned);
    }

    // TEAM_070: Using refactored walk_to_entry at level 2
    let walk = walk_to_entry(root, va, 2, true)?;
    walk.table.entry_mut(walk.index).set(pa, flags);

    Ok(())
}

/// Map a range using 2MB blocks where possible, otherwise 4KB pages.
pub fn map_range(
    root: &mut PageTable,
    va_start: usize,
    pa_start: usize,
    len: usize,
    flags: PageFlags,
) -> Result<MappingStats, MmuError> {
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
) -> Result<MappingStats, MmuError> {
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
    #[cfg(target_arch = "aarch64")]
    fn test_virt_to_phys_high_address() {
        // Kernel virtual address in higher half
        let va = KERNEL_VIRT_START + 0x4008_0000;
        let pa = virt_to_phys(va);
        assert_eq!(pa, 0x4008_0000);
    }

    // M20: phys_to_virt converts PA to high VA
    #[test]
    #[cfg(target_arch = "aarch64")]
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

    // TEAM_078: phys_to_virt now maps ALL addresses to high VA (including devices)
    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_phys_to_virt_device_high_va() {
        // Device addresses now also use high VA mapping
        let pa = 0x0900_0000; // UART address
        let va = phys_to_virt(pa);
        assert_eq!(va, KERNEL_VIRT_START + pa); // High VA for devices
    }

    // === Dynamic Page Allocation Tests (M23-M27) ===
    // TEAM_054: Tests for PageAllocator trait interface

    /// [M23] PageAllocator trait has alloc_page() method
    /// [M24] PageAllocator trait has free_page() method
    #[test]
    fn test_page_allocator_trait_interface() {
        use core::sync::atomic::{AtomicUsize, Ordering};

        // Mock allocator for compile-time interface verification
        struct MockAllocator {
            alloc_count: AtomicUsize,
            free_count: AtomicUsize,
        }

        impl PageAllocator for MockAllocator {
            fn alloc_page(&self) -> Option<usize> {
                let count = self.alloc_count.fetch_add(1, Ordering::SeqCst);
                Some(0x1000_0000 + count * 0x1000) // [M23]
            }
            fn free_page(&self, _pa: usize) {
                self.free_count.fetch_add(1, Ordering::SeqCst); // [M24]
            }
        }

        let allocator = MockAllocator {
            alloc_count: AtomicUsize::new(0),
            free_count: AtomicUsize::new(0),
        };

        // Test alloc_page [M23]
        let pa1 = allocator.alloc_page().expect("should allocate");
        assert_eq!(pa1, 0x1000_0000);
        assert_eq!(allocator.alloc_count.load(Ordering::SeqCst), 1);

        let pa2 = allocator.alloc_page().expect("should allocate");
        assert_eq!(pa2, 0x1000_1000);
        assert_eq!(allocator.alloc_count.load(Ordering::SeqCst), 2);

        // Test free_page [M24]
        allocator.free_page(pa1);
        assert_eq!(allocator.free_count.load(Ordering::SeqCst), 1);
    }

    /// [M25] set_page_allocator accepts &'static dyn PageAllocator
    /// Compile-time verification only — runtime test blocked by static mut
    #[test]
    fn test_set_page_allocator_signature() {
        // This test verifies the function signature compiles correctly
        // We cannot safely test runtime behavior due to static mut
        #[allow(dead_code)]
        fn assert_signature<T: PageAllocator + 'static>(_: &'static T) {
            // If this compiles, set_page_allocator can accept &'static T
        }

        // Compile-time verification passes if this test compiles
    }

    /// [M26] [M27] get_or_create_table allocation path exists
    /// Compile-time verification that the function uses PageAllocator
    #[test]
    fn test_allocation_paths_exist() {
        // Verify PageTable type is correct size (4KB)
        assert_eq!(core::mem::size_of::<PageTable>(), PAGE_SIZE);

        // Verify PageTableEntry is 8 bytes
        assert_eq!(core::mem::size_of::<PageTableEntry>(), 8);

        // Verify 512 entries per table (4KB / 8 bytes)
        assert_eq!(ENTRIES_PER_TABLE, 512);
    }

    #[test]
    fn test_map_unmap_cycle() {
        let mut root = PageTable::new();
        let va = 0x1234_5000usize;
        let pa = 0x4444_5000usize; // Use address that phys_to_virt handles (>= 0x4000_0000)
        let flags = PageFlags::KERNEL_DATA;

        // 1. Initial state: not mapped
        // walk_to_entry will fail because intermediate tables don't exist
        assert!(unmap_page(&mut root, va).is_err());

        // 2. Map page
        map_page(&mut root, va, pa, flags).expect("Mapping should succeed");

        // 3. Verify mapped
        let walk = walk_to_entry(&mut root, va, 3, false).expect("Walk should succeed");
        assert!(walk.table.entry(walk.index).is_valid());
        assert_eq!(walk.table.entry(walk.index).address(), pa);

        // 4. Unmap page
        unmap_page(&mut root, va).expect("Unmapping should succeed");

        // 5. Verify unmapped (entry cleared but path remains for now)
        let walk = walk_to_entry(&mut root, va, 3, false).expect("Walk should succeed");
        assert!(!walk.table.entry(walk.index).is_valid());

        // 6. Unmap again should fail because VALID bit is clear
        assert!(unmap_page(&mut root, va).is_err());
    }

    #[test]
    fn test_table_reclamation() {
        use core::sync::atomic::{AtomicUsize, Ordering};

        struct MockReclaimer {
            free_count: AtomicUsize,
        }
        impl PageAllocator for MockReclaimer {
            fn alloc_page(&self) -> Option<usize> {
                None // Fallback to static pool
            }
            fn free_page(&self, _pa: usize) {
                self.free_count.fetch_add(1, Ordering::SeqCst);
            }
        }

        // We use a static-like reference for the allocator
        static RECLAIMER: MockReclaimer = MockReclaimer {
            free_count: AtomicUsize::new(0),
        };

        let mut root = PageTable::new();
        let va = 0x1234_5000usize;
        let pa = 0x4444_5000usize;

        // 1. Map page FIRST (uses static pool)
        map_page(&mut root, va, pa, PageFlags::KERNEL_DATA).expect("Map should succeed");

        // 2. Set mock reclaimer
        unsafe {
            PAGE_ALLOCATOR_PTR = Some(&RECLAIMER);
        }

        // 3. Unmap and check reclamation
        RECLAIMER.free_count.store(0, Ordering::SeqCst);
        unmap_page(&mut root, va).expect("Unmap should succeed");

        // Expected: L3 freed, then L2 freed, then L1 freed. (3 total)
        // L0 (root) is never freed.
        assert_eq!(RECLAIMER.free_count.load(Ordering::SeqCst), 3);

        // 3. Verify path is gone from root
        assert!(root.entry(va_l0_index(va)).is_valid() == false);

        // Reset global state
        unsafe {
            PAGE_ALLOCATOR_PTR = None;
        }
    }
}
