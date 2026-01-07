// TEAM_260: x86_64 Paging support stubs.
// To be fully implemented in Stage 4.

use bitflags::bitflags;
use los_error::define_kernel_error;

define_kernel_error! {
    pub enum MmuError(0x01) {
        AllocationFailed = 0x01 => "Page table allocation failed",
        NotMapped = 0x02 => "Address not mapped",
        InvalidVirtualAddress = 0x03 => "Invalid virtual address",
        Misaligned = 0x04 => "Address not properly aligned",
        WalkFailed = 0x05 => "Page table walk failed",
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PageFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const NO_EXECUTE = 1 << 63;
        
        // Aliases for AArch64 parity where used generically in kernel
        const VALID = 1 << 0;
        const TABLE = 1 << 1; 
    }
}

impl PageFlags {
    pub const KERNEL_CODE: PageFlags = PageFlags::PRESENT;
    pub const KERNEL_DATA: PageFlags = PageFlags::PRESENT.union(PageFlags::WRITABLE);
    pub const DEVICE: PageFlags = PageFlags::PRESENT.union(PageFlags::WRITABLE);
    pub const USER_CODE: PageFlags = PageFlags::PRESENT.union(PageFlags::USER_ACCESSIBLE);
    pub const USER_DATA: PageFlags = PageFlags::PRESENT.union(PageFlags::USER_ACCESSIBLE).union(PageFlags::WRITABLE);
    pub const USER_STACK: PageFlags = Self::USER_DATA;
    pub const USER_CODE_DATA: PageFlags = PageFlags::PRESENT.union(PageFlags::USER_ACCESSIBLE).union(PageFlags::WRITABLE);
}

/// Page size: 4KB
pub const PAGE_SIZE: usize = 4096;
/// Page shift (log2 of page size)
pub const PAGE_SHIFT: usize = 12;

pub fn virt_to_phys(va: usize) -> usize {
    // Identity mapping for early boot/testing
    va
}

pub fn translate(_root: &PageTable, _va: usize) -> Option<(usize, PageFlags)> {
    None
}

pub fn phys_to_virt(pa: usize) -> usize {
    // Identity mapping for early boot/testing
    pa
}

// TEAM_260: PCI constants for x86_64 (stubs for now)
pub const ECAM_VA: usize = 0;
pub const PCI_MEM32_PA: usize = 0;
pub const PCI_MEM32_SIZE: usize = 0;

pub const ENTRIES_PER_TABLE: usize = 512;

// TEAM_260: Memory mapping constants for x86_64 (stubs for now)
pub const VIRTIO_MMIO_VA: usize = 0;
pub const GIC_DIST_VA: usize = 0;
pub const GIC_CPU_VA: usize = 0;
pub const GIC_REDIST_VA: usize = 0;
pub const UART_VA: usize = 0;

pub fn map_page(_root: &mut PageTable, _va: usize, _pa: usize, _flags: PageFlags) -> Result<(), MmuError> {
    Ok(())
}

pub fn unmap_page(_root: &mut PageTable, _va: usize) -> Result<(), MmuError> {
    Ok(())
}

pub fn tlb_flush_all() {}

pub fn set_page_allocator(_allocator: &'static dyn crate::traits::PageAllocator) {
    // Stub
}

#[derive(Clone, Copy)]
pub struct PageTableEntry(u64);
impl PageTableEntry {
    pub fn is_valid(&self) -> bool { (self.0 & PageFlags::PRESENT.bits()) != 0 }
    pub fn is_table(&self) -> bool { true } // Stub
    pub fn clear(&mut self) { self.0 = 0; }
    pub fn set(&mut self, pa: usize, flags: PageFlags) { self.0 = (pa as u64) | flags.bits(); }
    pub fn address(&self) -> usize { (self.0 & !0xFFF) as usize }
    pub fn flags(&self) -> PageFlags { PageFlags::from_bits_truncate(self.0) }
}

pub struct PageTable([PageTableEntry; 512]);

impl PageTable {
    pub const fn new() -> Self {
        Self([PageTableEntry(0); 512])
    }
    pub fn zero(&mut self) {
        for entry in self.0.iter_mut() {
            entry.clear();
        }
    }
    pub fn entry(&self, index: usize) -> &PageTableEntry { &self.0[index] }
    pub fn entry_mut(&mut self, index: usize) -> &mut PageTableEntry { 
        &mut self.0[index]
    }
}

impl crate::traits::MmuInterface for PageTable {
    fn map_page(&mut self, _va: usize, _pa: usize, _flags: PageFlags) -> Result<(), MmuError> {
        Ok(())
    }

    fn unmap_page(&mut self, _va: usize) -> Result<(), MmuError> {
        Ok(())
    }

    fn switch_to(&self) {}
}

pub fn tlb_flush_page(_va: usize) {}
pub fn walk_to_entry<'a>(_root: &'a mut PageTable, _va: usize, _level: usize, _create: bool) -> Result<WalkResult<'a>, MmuError> {
    Err(MmuError::WalkFailed)
}

pub struct WalkResult<'a> {
    pub table: &'a mut PageTable,
    pub index: usize,
}
