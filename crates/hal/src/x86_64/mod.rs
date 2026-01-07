// TEAM_260: x86_64 HAL module structure.

use crate::traits::InterruptController;

pub mod apic;
pub mod console;
pub mod exceptions;
pub mod frame_alloc;
pub mod idt;
pub mod interrupts;
pub mod ioapic;
pub mod mmu;
pub mod multiboot2; // TEAM_267: Multiboot2 boot info parsing
pub mod paging;
pub mod pit;
pub mod serial;
pub mod vga;

pub fn init() {
    // 0. Initialize MMU with higher-half mappings using early_pml4
    unsafe extern "C" {
        static mut early_pml4: paging::PageTable;
    }
    unsafe {
        mmu::init_kernel_mappings(&mut *core::ptr::addr_of_mut!(early_pml4));
    }

    // 1. Initialize serial for early logging
    unsafe { console::WRITER.lock().init() };

    // 2. Initialize IDT and exceptions
    idt::init();
    exceptions::init();

    // 3. Initialize APIC and IOAPIC
    apic::APIC.init();
    ioapic::IOAPIC.init();

    // 4. Initialize PIT
    pit::Pit::init(100); // 100Hz
}
