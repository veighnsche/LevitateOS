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
        let ptr = IdtPtr {
            limit: (size_of::<Self>() - 1) as u16,
            base: self as *const _ as u64,
        };
        unsafe {
            core::arch::asm!("lidt [{}]", in(reg) &ptr, options(readonly, nostack, preserves_flags));
        }
    }
}

#[repr(C, packed)]
struct IdtPtr {
    limit: u16,
    base: u64,
}

pub static IDT: Mutex<Idt> = Mutex::new(Idt::new());

pub fn init() {
    IDT.lock().load();
}
