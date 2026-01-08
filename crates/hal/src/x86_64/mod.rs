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

/// TEAM_286: Initialize HAL with optional CR3 switch.
/// `switch_cr3`: Set to false for Limine boot (Limine's page tables are already correct).
/// When false, APIC/IOAPIC init is also skipped (Limine may not identity-map APIC region).
pub fn init_with_options(switch_cr3: bool) {
    let is_limine = !switch_cr3; // If not switching CR3, we're on Limine
    // 0. Initialize MMU with higher-half mappings using early_pml4
    unsafe extern "C" {
        static mut early_pml4: paging::PageTable;
    }
    
    if switch_cr3 {
        unsafe {
            let root = &mut *core::ptr::addr_of_mut!(early_pml4);
            mmu::init_kernel_mappings(root);
            
            // TEAM_285: Switch to our own page tables now that they are initialized.
            // This is safer than doing it in assembly because we have verified mappings.
            let phys = mmu::virt_to_phys(root as *const _ as usize);
            core::arch::asm!("mov cr3, {}", in(reg) phys);
        }
    }
    // else: Limine boot - stay on Limine's page tables which have correct HHDM

    // 1. Initialize serial for early logging
    unsafe { console::WRITER.lock().init() };

    // 2. Initialize IDT and exceptions
    idt::init();
    exceptions::init();
    
    // 3. Initialize APIC and IOAPIC
    // TEAM_286: Skip for Limine boot - APIC region may not be identity-mapped
    if !is_limine {
        apic::APIC.init();
        ioapic::IOAPIC.init();
    }

    // 4. Initialize PIT
    // TEAM_286: PIT uses I/O ports, should work on Limine
    pit::Pit::init(100); // 100Hz
}

/// TEAM_286: Default init for multiboot boot (switches CR3).
pub fn init() {
    init_with_options(true)
}
