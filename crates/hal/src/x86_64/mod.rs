// TEAM_260: x86_64 HAL module structure.

use crate::traits::InterruptController;

pub mod apic;
pub mod console;
pub mod exceptions;
pub mod frame_alloc;
pub mod gdt;
pub mod tss {
    pub use crate::x86_64::gdt::*;
}
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
        // TEAM_308: Diagnostic 'a' - init_kernel_mappings Start
        unsafe {
            core::arch::asm!("mov dx, 0x3f8", "mov al, 'a'", "out dx, al");
        }
        unsafe {
            let root = &mut *core::ptr::addr_of_mut!(early_pml4);
            mmu::init_kernel_mappings(root);

            // TEAM_308: Diagnostic 'b' - init_kernel_mappings Done
            core::arch::asm!("mov al, 'b'", "out dx, al");

            // TEAM_285: Switch to our own page tables now that they are initialized.
            // This is safer than doing it in assembly because we have verified mappings.
            let phys = mmu::virt_to_phys(root as *const _ as usize);
            core::arch::asm!("mov cr3, {}", in(reg) phys);

            // TEAM_308: Diagnostic 'c' - CR3 Switched
            core::arch::asm!("mov al, 'c'", "out dx, al");
        }
    }
    // else: Limine boot - stay on Limine's page tables which have correct HHDM

    // 1. Initialize serial for early logging
    // TEAM_308: Diagnostic 'd' - Serial Init
    unsafe {
        core::arch::asm!("mov dx, 0x3f8", "mov al, 'd'", "out dx, al");
    }
    unsafe { console::WRITER.lock().init() };

    // 2. Initialize GDT, IDT and exceptions
    // TEAM_308: Diagnostic 'e' - GDT/IDT Init
    unsafe {
        core::arch::asm!("mov al, 'e'", "out dx, al");
    }
    unsafe { gdt::init() };
    idt::init();
    exceptions::init();

    // 3. Initialize APIC and IOAPIC
    // TEAM_308: Diagnostic 'f' - APIC Init
    unsafe {
        core::arch::asm!("mov al, 'f'", "out dx, al");
    }
    // TEAM_286: Skip for Limine boot - APIC region may not be identity-mapped
    if !is_limine {
        apic::APIC.init();
        ioapic::IOAPIC.init();
    }

    // 4. Initialize PIT
    // TEAM_308: Diagnostic 'g' - PIT Init
    unsafe {
        core::arch::asm!("mov al, 'g'", "out dx, al");
    }
    // TEAM_286: PIT uses I/O ports, should work on Limine
    pit::Pit::init(100); // 100Hz

    // TEAM_308: Diagnostic 'h' - HAL Init Done
    unsafe {
        core::arch::asm!("mov al, 'h'", "out dx, al");
    }
}

/// TEAM_286: Default init for multiboot boot (switches CR3).
pub fn init() {
    init_with_options(true)
}
