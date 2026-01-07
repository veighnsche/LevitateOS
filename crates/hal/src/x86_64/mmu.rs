use crate::traits::PageAllocator;
use crate::x86_64::frame_alloc::EARLY_ALLOCATOR;
pub use crate::x86_64::paging::{
    self, ENTRIES_PER_TABLE, PageTable, PageTableEntry, PageTableFlags,
};
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
    pub const USER_DATA: PageFlags = PageFlags::PRESENT
        .union(PageFlags::USER_ACCESSIBLE)
        .union(PageFlags::WRITABLE);
    pub const USER_STACK: PageFlags = Self::USER_DATA;
    pub const USER_CODE_DATA: PageFlags = PageFlags::PRESENT
        .union(PageFlags::USER_ACCESSIBLE)
        .union(PageFlags::WRITABLE);
}

/// Page size: 4KB
pub const PAGE_SIZE: usize = 4096;
/// Page shift (log2 of page size)
pub const PAGE_SHIFT: usize = 12;

// Higher-Half Virtual Address Base for x86_64
pub const KERNEL_VIRT_BASE: usize = 0xFFFFFFFF80000000;

/// TEAM_266: Physical Memory Offset (PMO) Mapping Base
/// Maps all physical memory starting at 0 to this virtual address.
/// Canonical high-half address 0xFFFF800000000000.
pub const PHYS_OFFSET: usize = 0xFFFF800000000000;

// TEAM_260: PCI constants for x86_64 (stubs for now)
pub const ECAM_VA: usize = 0;
pub const PCI_MEM32_PA: usize = 0;
pub const PCI_MEM32_SIZE: usize = 0;

// TEAM_260: Memory mapping constants for x86_64 (stubs for now)
pub const VIRTIO_MMIO_VA: usize = 0;
pub const GIC_DIST_VA: usize = 0;
pub const GIC_CPU_VA: usize = 0;
pub const GIC_REDIST_VA: usize = 0;
pub const UART_VA: usize = 0;

pub fn virt_to_phys(va: usize) -> usize {
    if va >= KERNEL_VIRT_BASE {
        va - KERNEL_VIRT_BASE
    } else if va >= PHYS_OFFSET {
        va - PHYS_OFFSET
    } else {
        va
    }
}

pub fn phys_to_virt(pa: usize) -> usize {
    pa + PHYS_OFFSET
}

pub fn translate(root: &PageTable, va: usize) -> Option<(usize, PageFlags)> {
    let pa = paging::translate_addr(root, va)?;
    // TODO: Recover flags from leaf entry
    Some((pa as usize, PageFlags::PRESENT))
}

pub fn map_page(
    root: &mut PageTable,
    va: usize,
    pa: usize,
    flags: PageFlags,
) -> Result<(), MmuError> {
    let p_flags = PageTableFlags::from_bits_truncate(flags.bits());
    paging::map_page(root, va, pa, p_flags, || EARLY_ALLOCATOR.alloc_page())
        .map_err(|_| MmuError::AllocationFailed)
}

pub fn unmap_page(root: &mut PageTable, va: usize) -> Result<(), MmuError> {
    paging::unmap_page(root, va)
        .map(|_| ())
        .map_err(|_| MmuError::NotMapped)
}

pub fn tlb_flush_all() {
    unsafe {
        use core::arch::asm;
        asm!("mov rax, cr3", "mov cr3, rax", out("rax") _);
    }
}

pub fn tlb_flush_page(va: usize) {
    unsafe {
        use core::arch::asm;
        asm!("invlpg [{}]", in(reg) va);
    }
}

unsafe extern "C" {
    static __kernel_phys_start: usize;
    static _kernel_end: usize;
}

/// TEAM_263: Initialize higher-half mappings for the kernel.
pub fn init_kernel_mappings(root: &mut PageTable) {
    // 1. Identity map first 1MB for BIOS/Multiboot stability
    for addr in (0..0x100000).step_by(PAGE_SIZE) {
        let _ = map_page(root, addr, addr, PageFlags::KERNEL_DATA);
    }

    // 2. Map kernel binary to higher-half
    let phys_start = unsafe { &__kernel_phys_start as *const _ as usize };
    let virt_end = unsafe { &_kernel_end as *const _ as usize };
    let size = virt_end - KERNEL_VIRT_BASE;

    for offset in (0..size).step_by(PAGE_SIZE) {
        let pa = phys_start + offset;
        let va = KERNEL_VIRT_BASE + offset;
        let _ = map_page(root, va, pa, PageFlags::KERNEL_DATA);
    }

    // 3. Map VGA and Serial for debugging
    let _ = map_page(root, 0xB8000, 0xB8000, PageFlags::DEVICE);
}

impl crate::traits::MmuInterface for PageTable {
    fn map_page(&mut self, va: usize, pa: usize, flags: PageFlags) -> Result<(), MmuError> {
        map_page(self, va, pa, flags)
    }

    fn unmap_page(&mut self, va: usize) -> Result<(), MmuError> {
        unmap_page(self, va)
    }

    fn switch_to(&self) {
        let phys = virt_to_phys(self as *const _ as usize);
        unsafe {
            use core::arch::asm;
            asm!("mov cr3, {}", in(reg) phys);
        }
    }
}
/// Result of a page table walk.
pub struct WalkResult<'a> {
    /// The table containing the leaf entry.
    pub table: &'a mut PageTable,
    /// The index of the leaf entry within the table.
    pub index: usize,
    /// The path of tables taken (for reclamation).
    pub breadcrumbs: Breadcrumbs,
}

pub struct Breadcrumbs {
    pub tables: [Option<usize>; 3],
    pub indices: [usize; 3],
}

/// Walk the page table to find the entry for a virtual address at a specific level.
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
        paging::pml4_index(va),
        paging::pdpt_index(va),
        paging::pd_index(va),
        paging::pt_index(va),
    ];

    let mut current_table = root;
    let mut breadcrumbs = Breadcrumbs {
        tables: [None; 3],
        indices: [0; 3],
    };

    for level in 0..target_level {
        let index = indices[level];
        breadcrumbs.tables[level] = Some(current_table as *mut _ as usize);
        breadcrumbs.indices[level] = index;

        let entry = current_table.entries[index];
        if !entry.flags().contains(PageTableFlags::PRESENT) {
            if create {
                let new_pa = EARLY_ALLOCATOR
                    .alloc_page()
                    .ok_or(MmuError::AllocationFailed)?;
                // TEAM_266: Use phys_to_virt (PHYS_OFFSET)
                let new_va = phys_to_virt(new_pa) as *mut PageTable;
                unsafe { (*new_va).zero() };

                current_table.entries[index].set_address(
                    new_pa,
                    PageTableFlags::PRESENT
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::USER_ACCESSIBLE,
                );
                current_table = unsafe { &mut *new_va };
            } else {
                return Err(MmuError::WalkFailed);
            }
        } else {
            let child_pa = entry.address();
            // TEAM_266: Use phys_to_virt (PHYS_OFFSET)
            let child_va = phys_to_virt(child_pa);
            current_table = unsafe { &mut *(child_va as *mut PageTable) };
        }
    }

    Ok(WalkResult {
        table: current_table,
        index: indices[target_level],
        breadcrumbs,
    })
}

static mut PAGE_ALLOCATOR_PTR: Option<&'static dyn PageAllocator> = None;

pub fn set_page_allocator(allocator: &'static dyn PageAllocator) {
    unsafe {
        PAGE_ALLOCATOR_PTR = Some(allocator);
    }
}
