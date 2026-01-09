use core::arch::global_asm;

global_asm!(include_str!("asm/boot.S"));

/// TEAM_131: DTB address passed by bootloader (x0)
/// Stored in a separate variable for verification.
#[unsafe(no_mangle)]
pub static mut BOOT_DTB_ADDR: u64 = 0;

/// TEAM_043: Preserved registers from bootloader (x0-x3)
#[unsafe(no_mangle)]
pub static mut BOOT_REGS: [u64; 4] = [0; 4];

use linked_list_allocator::LockedHeap;
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize the kernel heap from linker-defined symbols.
pub fn init_heap() {
    unsafe extern "C" {
        static __heap_start: u8;
        static __heap_end: u8;
    }

    unsafe {
        let heap_start = &__heap_start as *const u8 as usize;
        let heap_end = &__heap_end as *const u8 as usize;
        let heap_size = heap_end - heap_start;
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
    crate::verbose!("Heap initialized.");
}

/// Initialize MMU with fine-grained page tables.
pub fn init_mmu() {
    use los_hal::mmu;
    mmu::init();

    let root = unsafe {
        static mut ROOT_PT: mmu::PageTable = mmu::PageTable::new();
        &mut *core::ptr::addr_of_mut!(ROOT_PT)
    };

    let kernel_flags = mmu::PageFlags::KERNEL_DATA.difference(mmu::PageFlags::PXN);

    // Critical boot mappings
    {
        mmu::identity_map_range_optimized(
            root,
            mmu::KERNEL_PHYS_START,
            mmu::KERNEL_PHYS_END,
            kernel_flags,
        )
        .unwrap();
        mmu::map_range(
            root,
            mmu::KERNEL_VIRT_START + mmu::KERNEL_PHYS_START,
            mmu::KERNEL_PHYS_START,
            mmu::KERNEL_PHYS_END - mmu::KERNEL_PHYS_START,
            kernel_flags,
        )
        .unwrap();
        mmu::map_range(
            root,
            mmu::KERNEL_VIRT_START + 0x4000_0000,
            0x4000_0000,
            0x4000_0000,
            kernel_flags,
        )
        .unwrap();
        mmu::map_range(
            root,
            mmu::UART_VA,
            0x0900_0000,
            0x1000,
            mmu::PageFlags::DEVICE,
        )
        .unwrap();
        mmu::map_range(
            root,
            mmu::GIC_DIST_VA,
            0x0800_0000,
            0x20_0000,
            mmu::PageFlags::DEVICE,
        )
        .unwrap();
        mmu::map_range(
            root,
            mmu::VIRTIO_MMIO_VA,
            0x0a00_0000,
            0x10_0000,
            mmu::PageFlags::DEVICE,
        )
        .unwrap();
        mmu::map_range(
            root,
            mmu::ECAM_VA,
            mmu::ECAM_PA,
            0x10_0000,
            mmu::PageFlags::DEVICE,
        )
        .unwrap();
        mmu::map_range(
            root,
            mmu::PCI_MEM32_VA,
            mmu::PCI_MEM32_PA,
            mmu::PCI_MEM32_SIZE,
            mmu::PageFlags::DEVICE,
        )
        .unwrap();
        mmu::identity_map_range_optimized(root, 0x4000_0000, 0x5000_0000, kernel_flags).unwrap();
    }

    mmu::tlb_flush_all();
    let root_phys = mmu::virt_to_phys(root as *const _ as usize);
    unsafe {
        mmu::enable_mmu(root_phys, root_phys);
    }
    crate::verbose!("MMU re-initialized (Higher-Half + Identity).");
}

pub fn get_dtb_phys() -> Option<usize> {
    let addr = unsafe { BOOT_DTB_ADDR };
    if addr != 0 {
        crate::verbose!("DTB address from x0: 0x{:x}", addr);
        return Some(addr as usize);
    }
    let scan_start = 0x4000_0000usize;
    let scan_end = 0x4900_0000usize;
    for addr in (scan_start..scan_end).step_by(0x1000) {
        let magic = unsafe { core::ptr::read_volatile(addr as *const u32) };
        if u32::from_be(magic) == 0xd00d_feed {
            crate::verbose!("Found DTB at 0x{:x}", addr);
            return Some(addr);
        }
    }
    None
}

/// TEAM_282: Initialize BootInfo from DTB.
///
/// Parses the Device Tree Blob and stores the unified BootInfo globally.
/// This should be called early in the boot process after heap is available.
pub fn init_boot_info() {
    if let Some(dtb_phys) = get_dtb_phys() {
        let boot_info = unsafe { crate::boot::dtb::parse(dtb_phys) };
        
        // Store boot info globally
        unsafe {
            crate::boot::set_boot_info(boot_info);
        }
        
        // Log boot info
        if let Some(info) = crate::boot::boot_info() {
            crate::verbose!("[BOOT] Protocol: {:?}", info.protocol);
            if info.memory_map.len() > 0 {
                crate::verbose!(
                    "[BOOT] Memory: {} regions, {} MB usable",
                    info.memory_map.len(),
                    info.memory_map.total_usable() / (1024 * 1024)
                );
            }
        }
    }
}

pub fn print_boot_regs() {
    unsafe {
        crate::println!(
            "BOOT_REGS: x0={:x} x1={:x} x2={:x} x3={:x}",
            BOOT_REGS[0],
            BOOT_REGS[1],
            BOOT_REGS[2],
            BOOT_REGS[3]
        );
    }
}
