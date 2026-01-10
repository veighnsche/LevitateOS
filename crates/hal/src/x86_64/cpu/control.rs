//! x86_64 Specialized Control Instructions with mock support.
//! TEAM_373: Centralized control helpers to enable unit testing of CPU structures.

#[repr(C, packed)]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: u64,
}

// =============================================================================
// Real implementation for bare metal (no_std)
// =============================================================================

#[cfg(not(feature = "std"))]
mod real_impl {
    use super::DescriptorTablePointer;
    use core::arch::asm;

    pub unsafe fn lgdt(ptr: &DescriptorTablePointer) {
        asm!("lgdt [{}]", in(reg) ptr, options(readonly, nostack, preserves_flags));
    }

    pub unsafe fn ltr(sel: u16) {
        asm!("ltr {0:x}", in(reg) sel, options(nomem, nostack, preserves_flags));
    }

    pub unsafe fn lidt(ptr: &DescriptorTablePointer) {
        asm!("lidt [{}]", in(reg) ptr, options(readonly, nostack, preserves_flags));
    }

    pub unsafe fn load_cr3(phys: u64) {
        asm!("mov cr3, {}", in(reg) phys, options(nomem, nostack, preserves_flags));
    }

    pub unsafe fn read_cr3() -> u64 {
        let res: u64;
        asm!("mov {}, cr3", out(reg) res, options(nomem, nostack, preserves_flags));
        res
    }

    pub unsafe fn invlpg(va: usize) {
        asm!("invlpg [{}]", in(reg) va, options(nomem, nostack, preserves_flags));
    }

    pub unsafe fn flush_tlb() {
        asm!("mov rax, cr3", "mov cr3, rax", out("rax") _, options(nomem, nostack, preserves_flags));
    }
}

// =============================================================================
// Mock implementation for std feature (user-space tests)
// =============================================================================

#[cfg(feature = "std")]
mod mock_impl {
    use super::DescriptorTablePointer;
    use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};

    static GDT_BASE: AtomicU64 = AtomicU64::new(0);
    static GDT_LIMIT: AtomicU16 = AtomicU16::new(0);
    static IDT_BASE: AtomicU64 = AtomicU64::new(0);
    static IDT_LIMIT: AtomicU16 = AtomicU16::new(0);
    static TSS_SEL: AtomicU16 = AtomicU16::new(0);
    static CR3: AtomicU64 = AtomicU64::new(0);

    pub unsafe fn lgdt(ptr: &DescriptorTablePointer) {
        GDT_BASE.store(ptr.base, Ordering::SeqCst);
        GDT_LIMIT.store(ptr.limit, Ordering::SeqCst);
    }

    pub unsafe fn ltr(sel: u16) {
        TSS_SEL.store(sel, Ordering::SeqCst);
    }

    pub unsafe fn lidt(ptr: &DescriptorTablePointer) {
        IDT_BASE.store(ptr.base, Ordering::SeqCst);
        IDT_LIMIT.store(ptr.limit, Ordering::SeqCst);
    }

    pub unsafe fn load_cr3(phys: u64) {
        CR3.store(phys, Ordering::SeqCst);
    }

    pub unsafe fn read_cr3() -> u64 {
        CR3.load(Ordering::SeqCst)
    }

    // --- Verification Helpers ---
    pub fn get_gdt() -> (u64, u16) {
        (
            GDT_BASE.load(Ordering::SeqCst),
            GDT_LIMIT.load(Ordering::SeqCst),
        )
    }
    pub fn get_idt() -> (u64, u16) {
        (
            IDT_BASE.load(Ordering::SeqCst),
            IDT_LIMIT.load(Ordering::SeqCst),
        )
    }
    pub fn get_tss_sel() -> u16 {
        TSS_SEL.load(Ordering::SeqCst)
    }

    pub unsafe fn invlpg(_va: usize) {
        // Mock
    }

    pub unsafe fn flush_tlb() {
        // Mock
    }
}

// =============================================================================
// Public API
// =============================================================================

#[cfg(not(feature = "std"))]
pub use real_impl::*;

#[cfg(feature = "std")]
pub use mock_impl::*;
