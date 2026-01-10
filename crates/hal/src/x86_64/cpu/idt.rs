// TEAM_259: Interrupt Descriptor Table (IDT) for x86_64.
// Reference: https://wiki.osdev.org/Interrupt_Descriptor_Table

use core::mem::size_of;
use los_utils::Mutex;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    pub const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    pub fn set_handler(&mut self, handler: u64) {
        self.offset_low = handler as u16;
        self.selector = 0x08; // Kernel code segment selector
        self.ist = 0;
        self.type_attr = 0x8E; // Interrupt Gate, Present, Ring 0
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.zero = 0;
    }
}

#[repr(C, align(16))]
pub struct Idt([IdtEntry; 256]);

impl Idt {
    pub const fn new() -> Self {
        Self([IdtEntry::missing(); 256])
    }

    pub fn set_handler(&mut self, index: u8, handler: u64) {
        self.0[index as usize].set_handler(handler);
    }

    pub fn load(&self) {
        let ptr = crate::x86_64::cpu::DescriptorTablePointer {
            limit: (size_of::<Self>() - 1) as u16,
            base: self as *const _ as u64,
        };
        unsafe {
            crate::x86_64::cpu::lidt(&ptr);
        }
    }
}

pub static IDT: Mutex<Idt> = Mutex::new(Idt::new());

pub fn init() {
    IDT.lock().load();
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::x86_64::cpu::control::get_idt;

    #[test]
    fn test_idt_entry_set_handler() {
        let mut entry = IdtEntry::missing();
        let handler = 0xDEADBEEFCAFEBABEu64; // 8-byte address
        entry.set_handler(handler);

        // Verification must use local copies to avoid unaligned reference errors in packed struct
        let offset_low = entry.offset_low;
        let offset_mid = entry.offset_mid;
        let offset_high = entry.offset_high;

        let selector = entry.selector;
        let type_attr = entry.type_attr;

        assert_eq!(offset_low, 0xBABEu16);
        assert_eq!(offset_mid, 0xCAFEu16);
        assert_eq!(offset_high, 0xDEADBEEFu32);
        assert_eq!(selector, 0x08);
        assert_eq!(type_attr, 0x8E);
    }

    #[test]
    fn test_idt_load() {
        let idt = Idt::new();
        idt.load();

        let (base, limit) = get_idt();
        assert_eq!(base, &idt as *const _ as u64);
        assert_eq!(limit as usize, size_of::<Idt>() - 1);
    }
}
