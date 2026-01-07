// TEAM_259: I/O APIC driver for x86_64.
// Reference: https://wiki.osdev.org/IOAPIC

pub struct IoApic {
    base_addr: usize,
}

impl IoApic {
    pub const fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    unsafe fn read_reg(&self, reg: u8) -> u32 {
        let ioregsel = self.base_addr as *mut u32;
        let iowin = (self.base_addr + 0x10) as *const u32;

        unsafe {
            ioregsel.write_volatile(reg as u32);
            iowin.read_volatile()
        }
    }

    unsafe fn write_reg(&self, reg: u8, value: u32) {
        let ioregsel = self.base_addr as *mut u32;
        let iowin = (self.base_addr + 0x10) as *mut u32;

        unsafe {
            ioregsel.write_volatile(reg as u32);
            iowin.write_volatile(value);
        }
    }

    pub fn init(&self) {
        // Read IOAPIC ID and version if needed
        // let _version = unsafe { self.read_reg(0x01) };
    }

    /// Route a hardware IRQ to an IDT vector.
    pub fn route_irq(&self, irq: u8, vector: u8, dest_apic_id: u8) {
        let low_reg = 0x10 + irq * 2;
        let high_reg = 0x11 + irq * 2;

        // Low 32 bits: vector, delivery mode (000: fixed), dest mode (0: physical), 
        // pin polarity (0: high active), trigger mode (0: edge), mask (0: unmasked)
        let low_value = vector as u32;
        // High 32 bits: destination APIC ID in bits 56-63 (shifted to top of 32-bit register)
        let high_value = (dest_apic_id as u32) << 24;

        unsafe {
            self.write_reg(low_reg, low_value);
            self.write_reg(high_reg, high_value);
        }
    }
}

pub static IOAPIC: IoApic = IoApic::new(0xFEC00000);
