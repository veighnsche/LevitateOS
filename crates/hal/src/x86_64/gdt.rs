//! x86_64 GDT and TSS implementation.
//!
//! Behaviors: [X86_GDT1] GDT structure, [X86_GDT2] Kernel code DPL=0,
//! [X86_GDT3] User code DPL=3, [X86_GDT4] TSS 64-bit base,
//! [X86_TSS1] TSS.rsp0 for Ring 0 transitions, [X86_TSS2] set_kernel_stack()

use core::arch::asm;
use core::ptr::addr_of;

// TEAM_296: GDT Segment Selectors
// [X86_GDT2] Kernel code segment DPL=0, [X86_GDT3] User segments DPL=3
pub const KERNEL_CODE: u16 = 0x08;
pub const KERNEL_DATA: u16 = 0x10;
pub const USER_DATA: u16 = 0x18 | 3; // 0x1B
pub const USER_CODE: u16 = 0x20 | 3; // 0x23
pub const TSS_SELECTOR: u16 = 0x28;

#[repr(C, packed)]
pub struct TaskStateSegment {
    reserved_1: u32,
    pub rsp: [u64; 3], // rsp0, rsp1, rsp2
    reserved_2: u64,
    pub ist: [u64; 7],
    reserved_3: u64,
    reserved_4: u16,
    pub iopb_offset: u16,
}

impl TaskStateSegment {
    pub const fn new() -> Self {
        Self {
            reserved_1: 0,
            rsp: [0; 3],
            reserved_2: 0,
            ist: [0; 7],
            reserved_3: 0,
            reserved_4: 0,
            iopb_offset: 104, // Size of TSS
        }
    }
}

#[repr(C, packed)]
pub struct GdtTssEntry {
    pub limit_low: u16,
    pub base_low: u16,
    pub base_mid: u8,
    pub access: u8,
    pub limit_high_flags: u8,
    pub base_high: u8,
    pub base_upper: u32,
    pub reserved: u32,
}

#[repr(C, align(16))]
pub struct Gdt {
    pub null: u64,
    pub kernel_code: u64,
    pub kernel_data: u64,
    pub user_data: u64,
    pub user_code: u64,
    pub tss: GdtTssEntry,
}

impl Gdt {
    pub const fn new() -> Self {
        Self {
            null: 0,
            kernel_code: 0x00AF9A000000FFFF, // Long mode, Present, Exec, Read, DPL=0
            kernel_data: 0x00CF92000000FFFF, // Present, Write, DPL=0
            user_data: 0x00CFF2000000FFFF,   // Present, Write, DPL=3
            user_code: 0x00AFFA000000FFFF,   // Long mode, Present, Exec, Read, DPL=3
            tss: GdtTssEntry {
                limit_low: 0,
                base_low: 0,
                base_mid: 0,
                access: 0x89, // Present, Available 64-bit TSS
                limit_high_flags: 0,
                base_high: 0,
                base_upper: 0,
                reserved: 0,
            },
        }
    }
}

pub static mut TSS: TaskStateSegment = TaskStateSegment::new();
pub static mut GDT: Gdt = Gdt::new();

#[repr(C, packed)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

pub unsafe fn init() {
    let tss_addr = addr_of!(TSS) as usize;
    let tss_limit = (core::mem::size_of::<TaskStateSegment>() - 1) as u32;

    unsafe {
        GDT.tss.limit_low = tss_limit as u16;
        GDT.tss.base_low = tss_addr as u16;
        GDT.tss.base_mid = (tss_addr >> 16) as u8;
        GDT.tss.base_high = (tss_addr >> 24) as u8;
        GDT.tss.base_upper = (tss_addr >> 32) as u32;
        GDT.tss.limit_high_flags = ((tss_limit >> 16) & 0x0F) as u8;

        let ptr = GdtPointer {
            limit: (core::mem::size_of::<Gdt>() - 1) as u16,
            base: addr_of!(GDT) as u64,
        };

        asm!(
            "lgdt [{}]",
            "ltr {tss_sel:x}",
            in(reg) &ptr,
            tss_sel = in(reg) TSS_SELECTOR,
        );
    }
}

/// TEAM_296: Update the kernel stack pointer in the TSS for user-mode entry.
/// This ensures that if an interrupt occurs in userspace, the CPU switches to the correct kernel stack.
/// [X86_TSS2] Updates TSS.rsp[0] for Ring 0 transitions
pub fn set_kernel_stack(stack_top: usize) {
    unsafe {
        // [X86_TSS1] TSS.rsp0 provides kernel stack for Ring 0 entry
        TSS.rsp[0] = stack_top as u64;
    }
}
