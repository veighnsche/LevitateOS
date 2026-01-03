use core::ptr::{read_volatile, write_volatile};

pub const GICD_BASE: usize = 0x08000000;
pub const GICC_BASE: usize = 0x08010000;

// Distributor registers
const GICD_CTLR: usize = 0x000;
const GICD_TYPER: usize = 0x004;
const GICD_ISENABLER: usize = 0x100;
const GICD_ICENABLER: usize = 0x180;
const GICD_ICPENDR: usize = 0x280;
const GICD_IPRIORITYR: usize = 0x400;
const GICD_ITARGETSR: usize = 0x800;
const GICD_ICFGR: usize = 0xC00;

// CPU Interface registers
const GICC_CTLR: usize = 0x000;
const GICC_PMR: usize = 0x004;
const GICC_IAR: usize = 0x00C;
const GICC_EOIR: usize = 0x010;

pub const GIC_MAX_IRQ: u32 = 256;
pub const GIC_SPI_START: u32 = 32;

// TEAM_015: Typed IRQ identifiers and handler registry
// ======================================================

/// Known IRQ sources in LevitateOS.
/// Maps symbolic names to hardware IRQ numbers for the QEMU virt machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum IrqId {
    /// Virtual Timer (PPI, IRQ 27)
    VirtualTimer = 0,
    /// PL011 UART (SPI, IRQ 33)
    Uart = 1,
    // Future: VirtioGpu, VirtioInput, VirtioBlk, VirtioNet
}

/// Maximum number of registered handlers (must match IrqId variant count)
const MAX_HANDLERS: usize = 16;

impl IrqId {
    /// [G1] Maps IrqId to correct IRQ numbers
    #[inline]
    pub const fn irq_number(self) -> u32 {
        match self {
            IrqId::VirtualTimer => 27,  // [G1]
            IrqId::Uart => 33,          // [G1]
        }
    }

    /// [G2] Returns correct IrqId for known IRQ, [G3] returns None for unknown
    #[inline]
    pub fn from_irq_number(irq: u32) -> Option<Self> {
        match irq {
            27 => Some(IrqId::VirtualTimer),  // [G2]
            33 => Some(IrqId::Uart),          // [G2]
            _ => None,                         // [G3]
        }
    }
}

/// IRQ handler function type.
pub type IrqHandler = fn();

/// Static handler table (single-core assumption, set at boot).
static mut HANDLERS: [Option<IrqHandler>; MAX_HANDLERS] = [None; MAX_HANDLERS];

/// [G4] Register a handler for an IRQ.
///
/// # Safety
/// Must be called before interrupts are enabled. Not thread-safe.
pub fn register_handler(irq: IrqId, handler: IrqHandler) {
    let idx = irq as usize;
    unsafe {
        HANDLERS[idx] = Some(handler);  // [G4] stores handler
    }
}

/// [G5] Dispatch calls registered handler, [G6] returns false if unregistered
///
/// Returns `true` if a handler was found and called, `false` otherwise.
pub fn dispatch(irq_num: u32) -> bool {
    if let Some(irq_id) = IrqId::from_irq_number(irq_num) {
        let idx = irq_id as usize;
        unsafe {
            if let Some(handler) = HANDLERS[idx] {
                handler();           // [G5] calls handler
                return true;
            }
        }
    }
    false                            // [G6] unregistered returns false
}

pub struct Gic {
    dist_base: usize,
    cpu_base: usize,
}

// GIC is safe to share between threads if we are careful (or if we trust single-core + interrupt logic)
unsafe impl Sync for Gic {}

pub static API: Gic = Gic::new(GICD_BASE, GICC_BASE);

impl Gic {
    pub const fn new(dist_base: usize, cpu_base: usize) -> Self {
        Self {
            dist_base,
            cpu_base,
        }
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn gicd_write(&self, offset: usize, value: u32) {
        unsafe {
            write_volatile((self.dist_base + offset) as *mut u32, value);
            core::arch::asm!("dmb sy");
        }
    }

    #[cfg(not(target_arch = "aarch64"))]
    unsafe fn gicd_write(&self, offset: usize, value: u32) {
        unsafe {
            write_volatile((self.dist_base + offset) as *mut u32, value);
        }
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn gicd_read(&self, offset: usize) -> u32 {
        unsafe {
            let val = read_volatile((self.dist_base + offset) as *const u32);
            core::arch::asm!("dmb sy");
            val
        }
    }

    #[cfg(not(target_arch = "aarch64"))]
    unsafe fn gicd_read(&self, offset: usize) -> u32 {
        unsafe { read_volatile((self.dist_base + offset) as *const u32) }
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn gicc_write(&self, offset: usize, value: u32) {
        unsafe {
            write_volatile((self.cpu_base + offset) as *mut u32, value);
            core::arch::asm!("dmb sy");
        }
    }

    #[cfg(not(target_arch = "aarch64"))]
    unsafe fn gicc_write(&self, offset: usize, value: u32) {
        unsafe {
            write_volatile((self.cpu_base + offset) as *mut u32, value);
        }
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn gicc_read(&self, offset: usize) -> u32 {
        unsafe {
            let val = read_volatile((self.cpu_base + offset) as *const u32);
            core::arch::asm!("dmb sy");
            val
        }
    }

    #[cfg(not(target_arch = "aarch64"))]
    unsafe fn gicc_read(&self, offset: usize) -> u32 {
        unsafe { read_volatile((self.cpu_base + offset) as *const u32) }
    }

    pub fn init(&self) {
        unsafe {
            // Disable distributor
            self.gicd_write(GICD_CTLR, 0);

            let typer = self.gicd_read(GICD_TYPER);
            let num_irqs = ((typer & 0x1F) + 1) * 32;
            let num_irqs = if num_irqs > GIC_MAX_IRQ {
                GIC_MAX_IRQ
            } else {
                num_irqs
            };

            // Disable all interrupts
            for i in 0..(num_irqs / 32) {
                self.gicd_write(GICD_ICENABLER + (i as usize * 4), 0xFFFFFFFF);
            }

            // Clear all pending
            for i in 0..(num_irqs / 32) {
                self.gicd_write(GICD_ICPENDR + (i as usize * 4), 0xFFFFFFFF);
            }

            // Set priority to lowest
            for i in 0..(num_irqs / 4) {
                self.gicd_write(GICD_IPRIORITYR + (i as usize * 4), 0xA0A0A0A0);
            }

            // Target SPIs to CPU0
            for i in (GIC_SPI_START / 4)..(num_irqs / 4) {
                self.gicd_write(GICD_ITARGETSR + (i as usize * 4), 0x01010101);
            }

            // Configure level-triggered
            for i in (GIC_SPI_START / 16)..(num_irqs / 16) {
                self.gicd_write(GICD_ICFGR + (i as usize * 4), 0);
            }

            // TEAM_016: Enable Group0 and Group1 non-secure distribution
            self.gicd_write(GICD_CTLR, 0x3);

            // CPU Interface init
            self.gicc_write(GICC_PMR, 0xFF);
            self.gicc_write(GICC_CTLR, 1);
        }
    }

    /// Acknowledge an IRQ.
    /// Returns 1023 for spurious interrupts (caller should skip processing).
    pub fn acknowledge(&self) -> u32 {
        unsafe { self.gicc_read(GICC_IAR) & 0x3FF }
    }

    /// Check if an IRQ is spurious (1023 or 1022).
    #[inline]
    pub fn is_spurious(irq: u32) -> bool {
        irq >= 1020
    }

    pub fn end_interrupt(&self, irq: u32) {
        unsafe {
            self.gicc_write(GICC_EOIR, irq);
        }
    }

    pub fn enable_irq(&self, irq: u32) {
        if irq >= GIC_MAX_IRQ {
            return;
        }
        let reg = irq / 32;
        let bit = irq % 32;
        unsafe {
            self.gicd_write(GICD_ISENABLER + (reg as usize * 4), 1 << bit);
        }
    }

    pub fn disable_irq(&self, irq: u32) {
        if irq >= GIC_MAX_IRQ {
            return;
        }
        let reg = irq / 32;
        let bit = irq % 32;
        unsafe {
            self.gicd_write(GICD_ICENABLER + (reg as usize * 4), 1 << bit);
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    /// Tests: [G1] irq_number mapping, [G2] from_irq_number valid, [G3] from_irq_number unknown
    #[test]
    fn test_irq_id_mapping() {
        assert_eq!(IrqId::VirtualTimer.irq_number(), 27);  // [G1]
        assert_eq!(IrqId::Uart.irq_number(), 33);          // [G1]

        assert_eq!(IrqId::from_irq_number(27), Some(IrqId::VirtualTimer));  // [G2]
        assert_eq!(IrqId::from_irq_number(33), Some(IrqId::Uart));          // [G2]
        assert_eq!(IrqId::from_irq_number(100), None);                      // [G3]
    }

    /// Tests: [G4] handler registration, [G5] dispatch calls handler, [G6] unregistered returns false
    #[test]
    fn test_handler_registration_and_dispatch() {
        static mut CALLED: bool = false;
        fn test_handler() {
            unsafe {
                CALLED = true;
            }
        }

        register_handler(IrqId::Uart, test_handler);      // [G4]

        let handled = dispatch(33);                        // [G5]
        assert!(handled);
        unsafe {
            assert!(CALLED);
        }

        let handled_none = dispatch(27);                   // [G6] no handler registered
        assert!(!handled_none);
    }

    /// Tests: [G7] spurious IRQ detection (1020-1023)
    #[test]
    fn test_spurious_check() {
        assert!(Gic::is_spurious(1023));                   // [G7]
        assert!(Gic::is_spurious(1022));                   // [G7]
        assert!(Gic::is_spurious(1021));                   // [G7]
        assert!(Gic::is_spurious(1020));                   // [G7]
        assert!(!Gic::is_spurious(33));
    }
}
