// TEAM_259: x86_64 HAL module structure.

use crate::traits::InterruptController;

pub mod serial;
pub mod vga;
pub mod idt;
pub mod exceptions;
pub mod apic;
pub mod ioapic;
pub mod pit;

pub fn init() {
    // 1. Initialize serial for early logging
    unsafe { serial::COM1_PORT.lock().init() };
    
    // 2. Initialize IDT and exceptions
    idt::init();
    exceptions::init();
    
    // 3. Initialize APIC and IOAPIC
    apic::APIC.init();
    ioapic::IOAPIC.init();
    
    // 4. Initialize PIT
    pit::Pit::init(100); // 100Hz
}
