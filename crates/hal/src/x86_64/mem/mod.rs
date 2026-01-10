//! x86_64 Memory Management Compartment
//!
//! This module handles all memory-related functionality:
//! - **Paging** - 4-level page table structures (PML4, PDPT, PD, PT)
//! - **MMU** - Memory mapping, virtual-to-physical translation
//! - **Frame Allocator** - Physical frame allocation for page tables

pub mod frame_alloc;
pub mod mmu;
pub mod paging;

pub use frame_alloc::*;
pub use mmu::{
    MmuError, PageFlags, get_phys_offset, map_page, phys_to_virt, set_phys_offset, translate,
    unmap_page, virt_to_phys,
};
pub use paging::{
    ENTRIES_PER_TABLE, PageTable, PageTableEntry, PageTableFlags, pd_index, pdpt_index, pml4_index,
    pt_index, translate_addr as paging_translate_addr,
};
