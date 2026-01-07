use bitflags::bitflags;

// TEAM_263: x86_64 Page Table Entry and Page Table structures.
// Follows the 4-level paging scheme (PML4, PDPT, PD, PT).

bitflags! {
    /// x86_64 Page Table Entry flags.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PageTableFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
        const NO_EXECUTE = 1 << 63;
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn is_valid(&self) -> bool {
        self.flags().contains(PageTableFlags::PRESENT)
    }

    pub fn is_table(&self) -> bool {
        self.is_valid() && !self.flags().contains(PageTableFlags::HUGE_PAGE)
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.0)
    }

    pub fn address(&self) -> usize {
        (self.0 & 0x000f_ffff_ffff_f000) as usize
    }

    pub fn set_address(&mut self, addr: usize, flags: PageTableFlags) {
        assert!(addr & 0xfff == 0, "Physical address must be 4KB aligned");
        self.0 = (addr as u64) | flags.bits();
    }

    pub fn set(&mut self, addr: usize, flags: crate::x86_64::mmu::PageFlags) {
        self.0 = (addr as u64) | flags.bits();
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

/// Extract PML4 index (bits 39-47)
pub fn pml4_index(addr: usize) -> usize {
    ((addr >> 39) & 0x1ff) as usize
}

/// Extract PDPT index (bits 30-38)
pub fn pdpt_index(addr: usize) -> usize {
    ((addr >> 30) & 0x1ff) as usize
}

/// Extract PD index (bits 21-29)
pub fn pd_index(addr: usize) -> usize {
    ((addr >> 21) & 0x1ff) as usize
}

/// Extract PT index (bits 12-20)
pub fn pt_index(addr: usize) -> usize {
    ((addr >> 12) & 0x1ff) as usize
}

pub const ENTRIES_PER_TABLE: usize = 512;

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; ENTRIES_PER_TABLE],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::empty(); ENTRIES_PER_TABLE],
        }
    }

    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }

    pub fn entry(&self, index: usize) -> &PageTableEntry {
        &self.entries[index]
    }

    pub fn entry_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
}

/// Translate a virtual address to physical address by walking page tables.
/// Returns None if not mapped.
pub fn translate_addr(pml4: &PageTable, virt: usize) -> Option<usize> {
    let p3 = walk(pml4, pml4_index(virt))?;
    let p2 = walk(p3, pdpt_index(virt))?;
    let p1 = walk(p2, pd_index(virt))?;

    let entry = p1.entries[pt_index(virt)];
    if entry.flags().contains(PageTableFlags::PRESENT) {
        Some(entry.address() + (virt & 0xfff))
    } else {
        None
    }
}

/// Helper to walk one level.
/// Note: This assumes physical addresses are identity mapped or accessible.
fn walk(table: &PageTable, index: usize) -> Option<&PageTable> {
    let entry = table.entries[index];
    if entry.flags().contains(PageTableFlags::PRESENT) {
        // [TEAM_266]: Use PHYS_OFFSET to access the sub-table.
        let phys = entry.address();
        let virt = crate::x86_64::mmu::phys_to_virt(phys);
        unsafe { Some(&*(virt as *const PageTable)) }
    } else {
        None
    }
}

/// Helper to walk or create one level.
fn walk_mut<F>(table: &mut PageTable, index: usize, mut alloc_fn: F) -> Option<&mut PageTable>
where
    F: FnMut() -> Option<usize>,
{
    let entry = table.entries[index];
    if entry.flags().contains(PageTableFlags::PRESENT) {
        let phys = entry.address();
        let virt = crate::x86_64::mmu::phys_to_virt(phys);
        unsafe { Some(&mut *(virt as *mut PageTable)) }
    } else {
        let new_table_pa = alloc_fn()?;
        // TEAM_266: Use phys_to_virt (PHYS_OFFSET) to access new table.
        let new_table_va = crate::x86_64::mmu::phys_to_virt(new_table_pa);
        let new_table = unsafe { &mut *(new_table_va as *mut PageTable) };
        new_table.zero();

        table.entries[index].set_address(
            new_table_pa,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
        );
        Some(new_table)
    }
}

/// Map a virtual address to a physical address.
pub fn map_page<F>(
    pml4: &mut PageTable,
    virt: usize,
    phys: usize,
    flags: PageTableFlags,
    alloc_fn: F,
) -> Result<(), ()>
where
    F: FnMut() -> Option<usize>,
{
    let mut alloc_fn = alloc_fn;
    let p3 = walk_mut(pml4, pml4_index(virt), &mut alloc_fn).ok_or(())?;
    let p2 = walk_mut(p3, pdpt_index(virt), &mut alloc_fn).ok_or(())?;
    let p1 = walk_mut(p2, pd_index(virt), &mut alloc_fn).ok_or(())?;

    p1.entries[pt_index(virt)].set_address(phys, flags | PageTableFlags::PRESENT);
    Ok(())
}

/// Unmap a virtual address.
pub fn unmap_page(pml4: &mut PageTable, virt: usize) -> Result<usize, ()> {
    let p3 = walk_mut(pml4, pml4_index(virt), || None).ok_or(())?;
    let p2 = walk_mut(p3, pdpt_index(virt), || None).ok_or(())?;
    let p1 = walk_mut(p2, pd_index(virt), || None).ok_or(())?;

    let entry = p1.entries[pt_index(virt)];
    if !entry.flags().contains(PageTableFlags::PRESENT) {
        return Err(());
    }

    let pa = entry.address();
    p1.entries[pt_index(virt)].set_unused();
    Ok(pa)
}
