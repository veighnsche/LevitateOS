use crate::memory::user as mm_user;

// Memory management system calls.

/// TEAM_166: sys_sbrk - Adjust program break (heap allocation).
pub fn sys_sbrk(increment: isize) -> i64 {
    let task = crate::task::current_task();
    let mut heap = task.heap.lock();

    match heap.grow(increment) {
        Ok(old_break) => {
            if increment > 0 {
                let new_break = heap.current;
                let old_page = old_break / los_hal::mmu::PAGE_SIZE;
                // TEAM_181: Use checked arithmetic to prevent overflow
                let new_page = match new_break.checked_add(los_hal::mmu::PAGE_SIZE - 1) {
                    Some(n) => n / los_hal::mmu::PAGE_SIZE,
                    None => {
                        heap.current = old_break;
                        return 0; // Overflow
                    }
                };

                for page in old_page..new_page {
                    let va = page * los_hal::mmu::PAGE_SIZE;
                    if mm_user::user_va_to_kernel_ptr(task.ttbr0, va).is_none() {
                        if mm_user::alloc_and_map_heap_page(task.ttbr0, va).is_err() {
                            heap.current = old_break;
                            return 0; // null
                        }
                    }
                }
            }
            old_break as i64
        }
        Err(()) => 0,
    }
}
