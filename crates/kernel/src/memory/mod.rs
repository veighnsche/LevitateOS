use los_hal::mmu;
use los_hal::traits::PageAllocator;
use los_utils::Mutex;
use los_hal::allocator::{BuddyAllocator, Page};

pub use crate::task::TaskControlBlock;

pub mod heap; // TEAM_208: Process heap management
pub mod user; // TEAM_208: User-space memory management
pub mod vma; // TEAM_238: VMA tracking for munmap support

/// Global Frame Allocator
pub struct FrameAllocator(Mutex<BuddyAllocator>);

impl PageAllocator for FrameAllocator {
    fn alloc_page(&self) -> Option<usize> {
        self.0.lock().alloc(0)
    }
    fn free_page(&self, pa: usize) {
        self.0.lock().free(pa, 0)
    }
}

pub static FRAME_ALLOCATOR: FrameAllocator = FrameAllocator(Mutex::new(BuddyAllocator::new()));

fn add_reserved(
    regions: &mut [Option<(usize, usize)>],
    count: &mut usize,
    start: usize,
    end: usize,
) {
    if *count < regions.len() {
        regions[*count] = Some((start & !4095, (end + 4095) & !4095));
        *count += 1;
    }
}

fn add_range_with_holes(
    allocator: &mut BuddyAllocator,
    ram: (usize, usize),
    reserved: &[Option<(usize, usize)>],
) {
    fn add_split(
        allocator: &mut BuddyAllocator,
        ram: (usize, usize),
        reserved: &[Option<(usize, usize)>],
        idx: usize,
    ) {
        if ram.0 >= ram.1 {
            return;
        }

        // Find next valid reserved region
        let mut next_idx = idx;
        while next_idx < reserved.len() && reserved[next_idx].is_none() {
            next_idx += 1;
        }

        if next_idx >= reserved.len() {
            // No more reserved regions to check
            // SAFETY: The range (ram.0, ram.1) has been validated against all reserved
            // regions and is within discovered RAM bounds.
            unsafe {
                allocator.add_range(ram.0, ram.1);
            }
            return;
        }

        let res = reserved[next_idx].as_ref().expect("Checked None above");

        // Check for overlap
        if ram.0 < res.1 && ram.1 > res.0 {
            // Overlap! Split RAM into pieces that don't overlap with THIS reserved range
            // 1. Portion before reserved
            if ram.0 < res.0 {
                add_split(allocator, (ram.0, res.0), reserved, next_idx + 1);
            }
            // 2. Portion after reserved
            if ram.1 > res.1 {
                add_split(allocator, (res.1, ram.1), reserved, next_idx + 1);
            }
            // Portion inside is skipped
        } else {
            // No overlap with this one, try next reserved
            add_split(allocator, ram, reserved, next_idx + 1);
        }
    }

    add_split(allocator, ram, reserved, 0);
}

/// Initialize physical memory management.
#[cfg(target_arch = "aarch64")]
pub fn init(boot_info: &crate::boot::BootInfo) {
    // 1. Collect RAM and Reserved regions from BootInfo
    let mut ram_regions: [Option<(usize, usize)>; 16] = [None; 16];
    let mut ram_count = 0;

    for region in boot_info.memory_map.iter() {
        if region.kind == crate::boot::MemoryKind::Usable && ram_count < 16 {
            ram_regions[ram_count] = Some((region.base, region.base + region.size));
            ram_count += 1;
        }
    }

    let mut reserved_regions: [Option<(usize, usize)>; 32] = [None; 32];
    let mut res_count = 0;

    for region in boot_info.memory_map.iter() {
        if region.kind != crate::boot::MemoryKind::Usable && res_count < 32 {
            add_reserved(
                &mut reserved_regions,
                &mut res_count,
                region.base,
                region.base + region.size,
            );
        }
    }

    // Add mandatory kernel/heap reservations
    let kernel_start = mmu::virt_to_phys(unsafe { &_kernel_virt_start as *const _ as usize });
    let kernel_end_symbol = mmu::virt_to_phys(unsafe { &_kernel_end as *const _ as usize });
    let heap_end_symbol = mmu::virt_to_phys(unsafe { &__heap_end as *const _ as usize });
    let kernel_end = core::cmp::max(kernel_end_symbol, heap_end_symbol);
    add_reserved(
        &mut reserved_regions,
        &mut res_count,
        kernel_start,
        kernel_end,
    );

    // 2. Determine phys_min and phys_max for mem_map sizing
    let mut phys_min = !0;
    let mut phys_max = 0;
    for r in ram_regions.iter().flatten() {
        if r.0 < phys_min {
            phys_min = r.0;
        }
        if r.1 > phys_max {
            phys_max = r.1;
        }
    }

    if phys_min == !0 {
        crate::verbose!("No RAM discovered in BootInfo!");
        return;
    }

    let total_pages = (phys_max - phys_min) / 4096;
    let mem_map_size = total_pages * core::mem::size_of::<Page>();

    // Find a safe location for mem_map
    let mut mem_map_pa = 0;
    'outer: for ram in ram_regions.iter().flatten() {
        let mut check_start = ram.0;
        check_start = (check_start + 0x1FFFFF) & !0x1FFFFF;

        loop {
            let check_end = check_start + mem_map_size;
            if check_end > ram.1 {
                break;
            }

            let mut overlaps = false;
            for res in reserved_regions.iter().flatten() {
                if check_start < res.1 && check_end > res.0 {
                    overlaps = true;
                    check_start = (res.1 + 0x1FFFFF) & !0x1FFFFF;
                    break;
                }
            }

            if !overlaps {
                mem_map_pa = check_start;
                break 'outer;
            }
        }
    }

    if mem_map_pa == 0 {
        panic!("Failed to allocate physical memory for mem_map!");
    }

    add_reserved(
        &mut reserved_regions,
        &mut res_count,
        mem_map_pa,
        mem_map_pa + mem_map_size,
    );

    // 3. Initialize mem_map
    let mem_map_va = mmu::phys_to_virt(mem_map_pa);
    let mem_map = unsafe { core::slice::from_raw_parts_mut(mem_map_va as *mut Page, total_pages) };
    for page in mem_map.iter_mut() {
        *page = Page::new();
    }

    // 4. Initialize Allocator
    let mut allocator = FRAME_ALLOCATOR.0.lock();
    unsafe {
        allocator.init(mem_map, phys_min);
    }

    // 5. Add RAM regions, skipping reserved ones
    for ram in ram_regions.iter().flatten() {
        add_range_with_holes(&mut allocator, *ram, &reserved_regions);
    }

    // 6. Register with MMU
    drop(allocator);
    mmu::set_page_allocator(&FRAME_ALLOCATOR);

    crate::verbose!("Buddy Allocator initialized with unified BootInfo.");
}

unsafe extern "C" {
    static _kernel_virt_start: u8;
    static _kernel_end: u8;
    static __heap_end: u8;
}

// =============================================================================
// TEAM_267: x86_64 Physical Memory Initialization
// =============================================================================

/// Initialize physical memory management for x86_64.
#[cfg(target_arch = "x86_64")]
pub fn init(boot_info: &crate::boot::BootInfo) {
    // 1. Collect RAM and Reserved regions from BootInfo
    let mut ram_regions: [Option<(usize, usize)>; 16] = [None; 16];
    let mut ram_count = 0;

    for region in boot_info.memory_map.iter() {
        if region.kind == crate::boot::MemoryKind::Usable && ram_count < 16 {
            ram_regions[ram_count] = Some((region.base, region.base + region.size));
            ram_count += 1;
        }
    }

    let mut reserved_regions: [Option<(usize, usize)>; 32] = [None; 32];
    let mut res_count = 0;

    // BIOS/Legacy reservations (first 1MB)
    add_reserved(&mut reserved_regions, &mut res_count, 0, 0x100000);

    for region in boot_info.memory_map.iter() {
        if region.kind != crate::boot::MemoryKind::Usable && res_count < 32 {
            add_reserved(
                &mut reserved_regions,
                &mut res_count,
                region.base,
                region.base + region.size,
            );
        }
    }

    // Kernel/Heap reservations
    let kernel_start = unsafe { &_kernel_virt_start as *const _ as usize };
    let kernel_end = unsafe { &_kernel_end as *const _ as usize };
    let kernel_phys_start = mmu::virt_to_phys(kernel_start);
    let kernel_phys_end = mmu::virt_to_phys(kernel_end);
    add_reserved(
        &mut reserved_regions,
        &mut res_count,
        kernel_phys_start,
        kernel_phys_end,
    );

    let heap_end = unsafe { &__heap_end as *const _ as usize };
    let heap_phys_end = mmu::virt_to_phys(heap_end);
    if heap_phys_end > kernel_phys_end {
        add_reserved(
            &mut reserved_regions,
            &mut res_count,
            kernel_phys_end,
            heap_phys_end,
        );
    }

    // Early allocator reservation (TODO: remove if not needed with new scheme)
    add_reserved(&mut reserved_regions, &mut res_count, 0x800000, 0x1000000);

    // 2. Determine phys_min and phys_max
    let mut phys_min = !0usize;
    let mut phys_max = 0usize;

    for r in ram_regions.iter().flatten() {
        if r.0 < phys_min {
            phys_min = r.0;
        }
        if r.1 > phys_max {
            phys_max = r.1;
        }
    }

    if phys_min == !0 {
        los_hal::println!("[MEM] No RAM discovered in BootInfo!");
        return;
    }

    let total_pages = (phys_max - phys_min) / 4096;
    let mem_map_size = total_pages * core::mem::size_of::<Page>();

    // 3. Find a safe location for mem_map
    let mut mem_map_pa = 0;
    'outer: for region in ram_regions.iter().flatten() {
        let mut check_start = region.0;
        check_start = (check_start + 0x1FFFFF) & !0x1FFFFF; // 2MB align

        loop {
            let check_end = check_start + mem_map_size;
            if check_end > region.1 {
                break;
            }

            let mut overlaps = false;
            for res in reserved_regions.iter().flatten() {
                if check_start < res.1 && check_end > res.0 {
                    overlaps = true;
                    check_start = (res.1 + 0x1FFFFF) & !0x1FFFFF;
                    break;
                }
            }

            if !overlaps {
                mem_map_pa = check_start;
                break 'outer;
            }
        }
    }

    if mem_map_pa == 0 {
        los_hal::println!("[MEM] Failed to allocate physical memory for mem_map!");
        return;
    }

    add_reserved(
        &mut reserved_regions,
        &mut res_count,
        mem_map_pa,
        mem_map_pa + mem_map_size,
    );

    // 4. Initialize mem_map
    let mem_map_va = mmu::phys_to_virt(mem_map_pa);
    let mem_map = unsafe { core::slice::from_raw_parts_mut(mem_map_va as *mut Page, total_pages) };
    for page in mem_map.iter_mut() {
        *page = Page::new();
    }

    // 5. Initialize allocator
    let mut allocator = FRAME_ALLOCATOR.0.lock();
    unsafe {
        allocator.init(mem_map, phys_min);
    }

    // 6. Add RAM regions, skipping reserved ones
    for region in ram_regions.iter().flatten() {
        add_range_with_holes(&mut allocator, *region, &reserved_regions);
    }

    // 7. Register with MMU
    drop(allocator);
    mmu::set_page_allocator(&FRAME_ALLOCATOR);

    los_hal::println!("[MEM] Buddy Allocator initialized with unified BootInfo");
}
