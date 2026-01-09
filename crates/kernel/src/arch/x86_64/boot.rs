use linked_list_allocator::LockedHeap;
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

// TEAM_269: Linker symbols for heap region
unsafe extern "C" {
    static __heap_start: u8;
    static __heap_end: u8;
}

pub fn init_mmu() {
    // stub
}

pub fn get_dtb_phys() -> Option<usize> {
    // stub
    None
}

pub fn print_boot_regs() {
    // stub
}

/// TEAM_269: Initialize the kernel heap from linker-defined region.
/// Must be called before any code that allocates (println!, Vec, etc.)
pub fn init_heap() {
    let heap_start = unsafe { &__heap_start as *const _ as usize };
    let heap_end = unsafe { &__heap_end as *const _ as usize };
    let heap_size = heap_end - heap_start;

    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
    crate::verbose!("Heap initialized.");
}
