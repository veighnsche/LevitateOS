//! TEAM_299: x86_64 Processor Control Region (PCR) and CPU state management.

use core::arch::asm;

/// TEAM_299: MSR addresses
const IA32_GS_BASE: u32 = 0xC000_0101;
const IA32_KERNEL_GS_BASE: u32 = 0xC000_0102;

use los_hal::arch::gdt::{Gdt, TaskStateSegment};

/// TEAM_299: Processor Control Region (PCR).
/// This structure is pointed to by the GS segment register in kernel mode.
/// It contains per-CPU state that must be accessed quickly and safely.
///
/// IMPORTANT: The layout of this struct must match the offsets used in assembly.
#[repr(C, align(16))]
pub struct ProcessorControlRegion {
    /// Self-reference to this structure (offset 0)
    pub self_ptr: *const ProcessorControlRegion,
    /// Temporary scratch space for user RSP during syscall entry (offset 8)
    pub user_rsp_scratch: usize,
    /// The current kernel stack top for this CPU (offset 16)
    pub kernel_stack: usize,
    /// Pointer to the current TaskControlBlock (offset 24)
    pub current_task_ptr: usize,
    /// Per-CPU GDT (offset 32)
    pub gdt: Gdt,
    /// Per-CPU TSS (offset 96)
    pub tss: TaskStateSegment,
    /// Padding to ensure 16-byte alignment of the entire struct (offset 200)
    pub _padding: [u64; 1],
}

pub const PCR_SELF_OFFSET: usize = 0;
pub const PCR_USER_RSP_OFFSET: usize = 8;
pub const PCR_KSTACK_OFFSET: usize = 16;
pub const PCR_CURRENT_TASK_OFFSET: usize = 24;
pub const PCR_GDT_OFFSET: usize = 32;
pub const PCR_TSS_OFFSET: usize = 96;
pub const PCR_TSS_RSP0_OFFSET: usize = PCR_TSS_OFFSET + 4; // Offset of rsp0 within PCR

impl ProcessorControlRegion {
    pub const fn new() -> Self {
        Self {
            self_ptr: core::ptr::null(),
            user_rsp_scratch: 0,
            kernel_stack: 0,
            current_task_ptr: 0,
            gdt: Gdt::new(),
            tss: TaskStateSegment::new(),
            _padding: [0; 1],
        }
    }
}

/// TEAM_299: Get the current PCR for the calling CPU.
#[inline(always)]
pub unsafe fn get_pcr() -> &'static mut ProcessorControlRegion {
    let ptr: *mut ProcessorControlRegion;
    asm!("mov {}, gs:[0]", out(reg) ptr, options(nostack, nomem, preserves_flags));
    &mut *ptr
}

/// TEAM_299: Per-CPU PCR storage.
/// In a multi-core system, this would be an array or allocated per CPU.
pub static mut PCR: ProcessorControlRegion = ProcessorControlRegion::new();

pub unsafe fn init() {
    let pcr_ptr = &raw mut PCR;
    PCR.self_ptr = pcr_ptr;

    // Initialize per-CPU TSS and GDT
    let tss_addr = &raw const PCR.tss as usize;
    let tss_limit = (core::mem::size_of::<TaskStateSegment>() - 1) as u32;

    PCR.gdt.tss.limit_low = tss_limit as u16;
    PCR.gdt.tss.base_low = tss_addr as u16;
    PCR.gdt.tss.base_mid = (tss_addr >> 16) as u8;
    PCR.gdt.tss.base_high = (tss_addr >> 24) as u8;
    PCR.gdt.tss.base_upper = (tss_addr >> 32) as u32;
    PCR.gdt.tss.limit_high_flags = ((tss_limit >> 16) & 0x0F) as u8;

    #[repr(C, packed)]
    struct GdtPointer {
        limit: u16,
        base: u64,
    }

    let ptr = GdtPointer {
        limit: (core::mem::size_of::<Gdt>() - 1) as u16,
        base: &raw const PCR.gdt as u64,
    };

    asm!(
        "lgdt [{}]",
        "ltr {tss_sel:x}",
        in(reg) &ptr,
        tss_sel = in(reg) 0x28_u16, // TSS_SELECTOR
    );

    // Set GS_BASE to point to our PCR
    wrmsr(IA32_GS_BASE, pcr_ptr as u64);
    // KERNEL_GS_BASE will be swapped with GS_BASE on syscall entry
    wrmsr(IA32_KERNEL_GS_BASE, 0);

    los_hal::println!("[CPU] x86_64 PCR initialized at {:p}", pcr_ptr);
}

#[inline(always)]
unsafe fn wrmsr(msr: u32, value: u64) {
    let lo = value as u32;
    let hi = (value >> 32) as u32;
    asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") lo,
        in("edx") hi,
        options(nostack, nomem)
    );
}

pub fn wait_for_interrupt() {
    // x86: hlt
    unsafe { core::arch::asm!("hlt") };
}

pub fn halt() -> ! {
    loop {
        wait_for_interrupt();
    }
}
