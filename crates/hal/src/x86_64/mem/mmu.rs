use super::frame_alloc::EARLY_ALLOCATOR;
pub use super::paging::{self, ENTRIES_PER_TABLE, PageTable, PageTableEntry, PageTableFlags};
use crate::traits::PageAllocator;
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

    pub fn is_user(&self) -> bool {
        self.contains(PageFlags::USER_ACCESSIBLE)
    }

    pub fn is_writable(&self) -> bool {
        self.contains(PageFlags::WRITABLE)
    }
}

/// Page size: 4KB
pub const PAGE_SIZE: usize = 4096;
/// Page shift (log2 of page size)
pub const PAGE_SHIFT: usize = 12;

// Higher-Half Virtual Address Base for x86_64
pub const KERNEL_VIRT_BASE: usize = 0xFFFFFFFF80000000;

/// TEAM_285: Physical Memory Offset (PMO) Mapping Base.
/// This is now dynamic to support Limine's HHDM offset which can vary.
/// Defaults to the legacy 0xFFFF800000000000.
pub static mut PHYS_OFFSET: usize = 0xFFFF800000000000;

/// TEAM_288: Runtime kernel physical base address.
/// Set during boot from Limine's KernelAddressRequest response.
/// Defaults to linker script value (0x200000) for non-Limine boots.
pub static mut KERNEL_PHYS_BASE: usize = 0x200000;

/// TEAM_285: Set the physical memory offset (HHDM offset).
/// Called during early boot once the offset is known from the bootloader.
pub fn set_phys_offset(offset: usize) {
    unsafe {
        PHYS_OFFSET = offset;
    }
}

/// TEAM_285: Get the current physical memory offset.
pub fn get_phys_offset() -> usize {
    unsafe { PHYS_OFFSET }
}

/// TEAM_288: Set the kernel physical base address.
/// Called during early boot from Limine's KernelAddressRequest response.
pub fn set_kernel_phys_base(pa: usize) {
    unsafe {
        KERNEL_PHYS_BASE = pa;
    }
}

/// TEAM_288: Get the kernel physical base address.
pub fn get_kernel_phys_base() -> usize {
    unsafe { KERNEL_PHYS_BASE }
}

// TEAM_284: PCI constants for x86_64 (QEMU q35)
pub const ECAM_PA: usize = 0xB0000000;
pub const ECAM_VA: usize = 0xFFFFFFFF40000000;
pub const PCI_MEM32_PA: usize = 0xC0000000;
pub const PCI_MEM32_VA: usize = 0xFFFFFFFF60000000;
pub const PCI_MEM32_SIZE: usize = 0x20000000;

// TEAM_284: VirtIO MMIO constants for x86_64 (QEMU default)
pub const VIRTIO_MMIO_PA: usize = 0xFEB00000; // Typical location for MMIO on x86_64 QEMU
pub const VIRTIO_MMIO_VA: usize = 0xFFFFFFFF80000000 - 0x1000000; // Just below kernel
pub const VIRTIO_MMIO_SIZE: usize = 0x2000;

pub const GIC_DIST_VA: usize = 0;
pub const GIC_CPU_VA: usize = 0;
pub const GIC_REDIST_VA: usize = 0;
pub const UART_VA: usize = 0;

/// TEAM_288: Convert virtual address to physical address.
/// Uses runtime KERNEL_PHYS_BASE for higher-half addresses (set from Limine).
pub fn virt_to_phys(va: usize) -> usize {
    if va >= KERNEL_VIRT_BASE {
        // TEAM_288: Use runtime kernel physical base instead of linker symbol
        va - KERNEL_VIRT_BASE + unsafe { KERNEL_PHYS_BASE }
    } else if va >= unsafe { PHYS_OFFSET } {
        va - unsafe { PHYS_OFFSET }
    } else {
        va
    }
}

pub fn phys_to_virt(pa: usize) -> usize {
    pa + unsafe { PHYS_OFFSET }
}

pub fn translate(root: &PageTable, va: usize) -> Option<(usize, PageFlags)> {
    let pml4_idx = paging::pml4_index(va);
    let pdpt_idx = paging::pdpt_index(va);
    let pd_idx = paging::pd_index(va);
    let pt_idx = paging::pt_index(va);

    let pml4_entry = root.entries[pml4_idx];
    if !pml4_entry.flags().contains(PageTableFlags::PRESENT) {
        return None;
    }

    let pdpt = unsafe { &*(phys_to_virt(pml4_entry.address()) as *const PageTable) };
    let pdpt_entry = pdpt.entries[pdpt_idx];
    if !pdpt_entry.flags().contains(PageTableFlags::PRESENT) {
        return None;
    }

    let pd = unsafe { &*(phys_to_virt(pdpt_entry.address()) as *const PageTable) };
    let pd_entry = pd.entries[pd_idx];
    if !pd_entry.flags().contains(PageTableFlags::PRESENT) {
        return None;
    }

    if pd_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
        let pa = pd_entry.address() + (va & 0x1f_ffff);
        let flags = PageFlags::from_bits_truncate(pd_entry.flags().bits());
        return Some((pa, flags));
    }

    let pt = unsafe { &*(phys_to_virt(pd_entry.address()) as *const PageTable) };
    let pt_entry = pt.entries[pt_idx];
    if !pt_entry.flags().contains(PageTableFlags::PRESENT) {
        return None;
    }

    let pa = pt_entry.address() + (va & 0xfff);
    let flags = PageFlags::from_bits_truncate(pt_entry.flags().bits());
    Some((pa, flags))
}

pub fn map_page(
    root: &mut PageTable,
    va: usize,
    pa: usize,
    flags: PageFlags,
) -> Result<(), MmuError> {
    let p_flags = PageTableFlags::from_bits_truncate(flags.bits());
    let alloc_fn = || unsafe {
        if let Some(alloc) = PAGE_ALLOCATOR_PTR {
            alloc.alloc_page()
        } else {
            EARLY_ALLOCATOR.alloc_page()
        }
    };
    paging::map_page(root, va, pa, p_flags, alloc_fn).map_err(|_| MmuError::AllocationFailed)
}

pub fn unmap_page(root: &mut PageTable, va: usize) -> Result<(), MmuError> {
    paging::unmap_page(root, va)
        .map(|_| ())
        .map_err(|_| MmuError::NotMapped)
}

pub fn tlb_flush_all() {
    unsafe {
        crate::x86_64::cpu::flush_tlb();
    }
}

pub fn tlb_flush_page(va: usize) {
    unsafe {
        crate::x86_64::cpu::invlpg(va);
    }
}

unsafe extern "C" {
    static __kernel_phys_start: usize;
    static _kernel_end: usize;
}

/// TEAM_263: Initialize higher-half mappings for the kernel.
/// TEAM_316: Simplified - removed APIC/PCI mappings that crash due to 1GB PMO limit
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

    // 3. Map VGA for debugging
    let _ = map_page(root, 0xB8000, 0xB8000, PageFlags::DEVICE);

    // 4. APIC/IOAPIC - Already mapped by assembly as 2MB huge pages
    // TEAM_316: Skip - phys_to_virt(0xFEC00000/0xFEE00000) outside 1GB PMO range

    // 5. PCI ECAM/MMIO - Deferred to PCI subsystem init
    // TEAM_316: Skip - ECAM_PA/PCI_MEM32_PA outside 1GB PMO range
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
            crate::x86_64::cpu::load_cr3(phys as u64);
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
                let new_pa = unsafe {
                    if let Some(alloc) = PAGE_ALLOCATOR_PTR {
                        alloc.alloc_page()
                    } else {
                        EARLY_ALLOCATOR.alloc_page()
                    }
                }
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
                // crate::println!("[MMU] Walk failed at level {} (not present)", level);
                return Err(MmuError::WalkFailed);
            }
        } else {
            let child_pa = entry.address();
            // TEAM_266: Use phys_to_virt (PHYS_OFFSET)
            let child_va = phys_to_virt(child_pa);
            current_table = unsafe { &mut *(child_va as *mut PageTable) };
        }
    }

    let leaf_index = indices[target_level];
    if !current_table.entries[leaf_index]
        .flags()
        .contains(PageTableFlags::PRESENT)
    {
        // crate::println!("[MMU] Walk failed: leaf not present at level {}", target_level);
        return Err(MmuError::WalkFailed);
    }

    Ok(WalkResult {
        table: current_table,
        index: leaf_index,
        breadcrumbs,
    })
}

static mut PAGE_ALLOCATOR_PTR: Option<&'static dyn PageAllocator> = None;

pub fn set_page_allocator(allocator: &'static dyn PageAllocator) {
    unsafe {
        PAGE_ALLOCATOR_PTR = Some(allocator);
    }
}

// =============================================================================
// TEAM_316: Kernel Segment Permissions (Limine-only, PMO handled by bootloader)
// =============================================================================

// Linker symbols for kernel segment boundaries
unsafe extern "C" {
    static __text_start: u8;
    static __text_end: u8;
    static __rodata_start: u8;
    static __rodata_end: u8;
    static __data_start: u8;
    static __data_end: u8;
    static __bss_start: u8;
    static __bss_end: u8;
}

/// TEAM_267: Initialize kernel mappings with proper segment permissions.
/// - .text: Read-Execute (R-X)
/// - .rodata: Read-Only (R--)
/// - .data/.bss: Read-Write (RW-)
pub fn init_kernel_mappings_refined(root: &mut PageTable) {
    // 1. Identity map first 1MB for BIOS/Multiboot stability
    for addr in (0..0x100000).step_by(PAGE_SIZE) {
        let _ = map_page(root, addr, addr, PageFlags::KERNEL_DATA);
    }

    // 2. Map kernel segments with proper permissions
    let phys_start = unsafe { &__kernel_phys_start as *const _ as usize };

    // Calculate segment boundaries (virtual addresses)
    let text_start_va = unsafe { &__text_start as *const _ as usize };
    let text_end_va = unsafe { &__text_end as *const _ as usize };
    let rodata_start_va = unsafe { &__rodata_start as *const _ as usize };
    let rodata_end_va = unsafe { &__rodata_end as *const _ as usize };
    let data_start_va = unsafe { &__data_start as *const _ as usize };
    let data_end_va = unsafe { &__data_end as *const _ as usize };
    let bss_start_va = unsafe { &__bss_start as *const _ as usize };
    let bss_end_va = unsafe { &__bss_end as *const _ as usize };

    // .text segment: Read-Execute (no NX bit)
    map_segment(
        root,
        text_start_va,
        text_end_va,
        phys_start,
        PageFlags::KERNEL_CODE,
    );

    // .rodata segment: Read-Only + NX
    map_segment(
        root,
        rodata_start_va,
        rodata_end_va,
        phys_start,
        PageFlags::PRESENT.union(PageFlags::NO_EXECUTE),
    );

    // .data segment: Read-Write + NX
    map_segment(
        root,
        data_start_va,
        data_end_va,
        phys_start,
        PageFlags::KERNEL_DATA.union(PageFlags::NO_EXECUTE),
    );

    // .bss segment: Read-Write + NX
    map_segment(
        root,
        bss_start_va,
        bss_end_va,
        phys_start,
        PageFlags::KERNEL_DATA.union(PageFlags::NO_EXECUTE),
    );

    // 3. Map VGA for debugging
    let _ = map_page(root, 0xB8000, 0xB8000, PageFlags::DEVICE);
}

/// Helper to map a kernel segment
fn map_segment(
    root: &mut PageTable,
    va_start: usize,
    va_end: usize,
    phys_base: usize,
    flags: PageFlags,
) {
    let mut va = va_start & !(PAGE_SIZE - 1);
    while va < va_end {
        let offset = va - KERNEL_VIRT_BASE;
        let pa = phys_base + offset;
        let _ = map_page(root, va, pa, flags);
        va += PAGE_SIZE;
    }
}

/// TEAM_267: Copy kernel higher-half mappings to a new page table root.
/// Used when creating new process address spaces.
pub fn copy_kernel_mappings(dst: &mut PageTable, src: &PageTable) {
    // Copy PML4 entries for higher-half (entries 256-511)
    // Entry 256+ covers 0xFFFF800000000000 and above
    for i in 256..512 {
        if src.entries[i].is_valid() {
            dst.entries[i] = src.entries[i];
        }
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    /// Tests: [M23] Physical address subtraction, [M21] Virtual address addition
    #[test]
    fn test_address_conversion() {
        unsafe {
            let original_offset = get_phys_offset();
            let original_phys_base = get_kernel_phys_base();

            set_phys_offset(0xFFFF800000000000);
            set_kernel_phys_base(0x2000000);

            // Higher-half HHDM
            assert_eq!(phys_to_virt(0x1000), 0xFFFF800000001000);
            assert_eq!(virt_to_phys(0xFFFF800000001000), 0x1000);

            // Higher-half Kernel
            assert_eq!(virt_to_phys(0xFFFFFFFF80000000), 0x2000000);
            assert_eq!(virt_to_phys(0xFFFFFFFF80001000), 0x2000000 + 0x1000);

            // Cleanup
            set_phys_offset(original_offset);
            set_kernel_phys_base(original_phys_base);
        }
    }

    #[test]
    fn test_page_flags() {
        let flags = PageFlags::KERNEL_DATA;
        assert!(flags.contains(PageFlags::PRESENT));
        assert!(flags.contains(PageFlags::WRITABLE));
        assert!(!flags.is_user());

        let u_flags = PageFlags::USER_CODE;
        assert!(u_flags.contains(PageFlags::PRESENT));
        assert!(u_flags.is_user());
        assert!(!u_flags.is_writable());
    }

    /// TEAM_373: Test page table walk and translation using mock memory.
    #[test]
    fn test_mmu_translation_mock() {
        #[repr(C, align(4096))]
        #[derive(Clone, Copy)]
        struct AlignedPage([u8; 4096]);

        // Allocate 16 pages of aligned memory
        let mut mock_phys_mem = vec![AlignedPage([0u8; 4096]); 16];
        let base_ptr = mock_phys_mem.as_mut_ptr() as usize;

        unsafe {
            let original_offset = get_phys_offset();
            // Set offset such that phys_to_virt(pa) returns a pointer into our vec
            set_phys_offset(base_ptr);

            // 1. Setup a simple PML4 -> PDPT -> PD -> PT -> Page structure
            // We use offsets within the vec as "physical addresses"
            let pml4_pa = 0;
            let pdpt_pa = 4096;
            let pd_pa = 8192;
            let pt_pa = 12288;
            let target_pa = 16384;

            let root = &mut *(phys_to_virt(pml4_pa) as *mut PageTable);
            root.zero();

            // PML4[0] -> PDPT
            root.entries[0]
                .set_address(pdpt_pa, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);

            // PDPT[0] -> PD
            let pdpt = &mut *(phys_to_virt(pdpt_pa) as *mut PageTable);
            pdpt.zero();
            pdpt.entries[0].set_address(pd_pa, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);

            // PD[0] -> PT
            let pd = &mut *(phys_to_virt(pd_pa) as *mut PageTable);
            pd.zero();
            pd.entries[0].set_address(pt_pa, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);

            // PT[0] -> target
            let pt = &mut *(phys_to_virt(pt_pa) as *mut PageTable);
            pt.zero();
            pt.entries[0].set_address(
                target_pa,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            );

            // 2. Translate virtual address 0
            if let Some((pa, flags)) = translate(root, 0) {
                assert_eq!(pa, target_pa);
                assert!(flags.contains(PageFlags::PRESENT));
            } else {
                panic!("Translation failed");
            }

            // 3. Translate non-existent address
            assert!(translate(root, 0x1000 * 512 * 512 * 512).is_none());

            // Cleanup
            set_phys_offset(original_offset);
        }
    }
}
