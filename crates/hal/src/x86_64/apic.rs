// TEAM_259: Local APIC driver for x86_64.
// Reference: https://wiki.osdev.org/APIC

use crate::traits::{InterruptController, IrqId, InterruptHandler};
use los_utils::Mutex;

const MAX_HANDLERS: usize = 256;
static HANDLERS: Mutex<[Option<&'static dyn InterruptHandler>; MAX_HANDLERS]> = Mutex::new([None; MAX_HANDLERS]);

pub fn register_handler(vector: u8, handler: &'static dyn InterruptHandler) {
    let mut handlers = HANDLERS.lock();
    handlers[vector as usize] = Some(handler);
}

pub fn dispatch(vector: u8) -> bool {
    let handlers = HANDLERS.lock();
    if let Some(handler) = handlers[vector as usize] {
        handler.handle(vector as u32);
        true
    } else {
        false
    }
}

const IA32_APIC_BASE_MSR: u32 = 0x1B;
const APIC_REGISTER_ID: u32 = 0x20;
const APIC_REGISTER_VERSION: u32 = 0x30;
const APIC_REGISTER_EOI: u32 = 0xB0;
const APIC_REGISTER_SPURIOUS: u32 = 0xF0;

pub struct ApicController {
    base_addr: usize,
}

impl ApicController {
    pub const fn new() -> Self {
        Self {
            base_addr: 0xFEE00000, // Default base address
        }
    }

    /// Read from an APIC register.
    unsafe fn read_reg(&self, reg: u32) -> u32 {
        let ptr = (self.base_addr + reg as usize) as *const u32;
        unsafe { ptr.read_volatile() }
    }

    /// Write to an APIC register.
    unsafe fn write_reg(&self, reg: u32, value: u32) {
        let ptr = (self.base_addr + reg as usize) as *mut u32;
        unsafe { ptr.write_volatile(value) };
    }

    pub fn id(&self) -> u32 {
        unsafe { self.read_reg(APIC_REGISTER_ID) >> 24 }
    }

    pub fn signal_eoi(&self) {
        unsafe { self.write_reg(APIC_REGISTER_EOI, 0) };
    }
}

impl InterruptController for ApicController {
    fn init(&self) {
        unsafe {
            // Enable APIC by setting bit 8 of the Spurious Interrupt Vector Register
            let spurious = self.read_reg(APIC_REGISTER_SPURIOUS);
            self.write_reg(APIC_REGISTER_SPURIOUS, spurious | 0x1FF); // 0xFF is the spurious vector
        }
    }

    fn enable_irq(&self, _irq: u32) {
        // Handled by IOAPIC
    }

    fn disable_irq(&self, _irq: u32) {
        // Handled by IOAPIC
    }

    fn acknowledge(&self) -> u32 {
        // On x86, acknowledgment is implicit when the interrupt is delivered to the CPU.
        // We might need to return the vector from the stack in the handler instead.
        0
    }

    fn end_of_interrupt(&self, _irq: u32) {
        self.signal_eoi();
    }

    fn is_spurious(&self, irq: u32) -> bool {
        irq == 0xFF
    }

    fn register_handler(&self, irq: IrqId, handler: &'static dyn InterruptHandler) {
        let vector = self.map_irq(irq);
        register_handler(vector as u8, handler);
    }

    fn map_irq(&self, irq: IrqId) -> u32 {
        match irq {
            IrqId::VirtualTimer => 32, // Map PIT to vector 32
            IrqId::Uart => 36,         // Map COM1 (IRQ 4) to vector 36
            _ => 0,
        }
    }
}

pub static APIC: ApicController = ApicController::new();

pub fn active_api() -> &'static dyn InterruptController {
    &APIC
}
