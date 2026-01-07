use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicPtr, Ordering};

// TEAM_042: GICv2 and GICv3 support for Pixel 6 compatibility
// GICv2 uses memory-mapped GICC registers
// GICv3 uses system registers (ICC_*_EL1) and adds Redistributor (GICR)
// TEAM_045: FDT-based detection implemented for reliable version discovery

// TEAM_078: Use high VA for GIC (accessible via TTBR1 regardless of TTBR0 state)
pub const GICD_BASE: usize = crate::aarch64::mmu::GIC_DIST_VA;
pub const GICC_BASE: usize = crate::aarch64::mmu::GIC_CPU_VA; // GICv2 CPU interface
pub const GICR_BASE: usize = crate::aarch64::mmu::GIC_REDIST_VA; // GICv3 Redistributor (QEMU virt)

// Distributor registers (shared between v2 and v3)
const GICD_CTLR: usize = 0x000;
const GICD_TYPER: usize = 0x004;
const GICD_ISENABLER: usize = 0x100;
const GICD_ICENABLER: usize = 0x180;
const GICD_ICPENDR: usize = 0x280;
const GICD_IPRIORITYR: usize = 0x400;
const GICD_ITARGETSR: usize = 0x800; // GICv2 only
const GICD_ICFGR: usize = 0xC00;

// GICv2 CPU Interface registers (memory-mapped)
const GICC_CTLR: usize = 0x000;
const GICC_PMR: usize = 0x004;
const GICC_IAR: usize = 0x00C;
const GICC_EOIR: usize = 0x010;

// GICv3 Redistributor registers (per-CPU, 64KB stride)
const GICR_WAKER: usize = 0x0014;
// SGI base is at offset 0x10000 from redistributor base
const GICR_SGI_BASE: usize = 0x10000;
const GICR_IGROUPR0: usize = 0x0080;
const GICR_ICENABLER0: usize = 0x0180;
const GICR_IPRIORITYR: usize = 0x0400;

use bitflags::bitflags;

bitflags! {
    /// GICD_CTLR register flags
    pub struct GicdCtlrFlags: u32 {
        /// Register Write Pending (GICv3)
        const RWP = 1 << 31;
        /// Affinity Routing Enable (Secure)
        const ARE_S = 1 << 4;
        /// Affinity Routing Enable (Non-secure)
        const ARE_NS = 1 << 5;
        /// Enable Group 1 Non-secure interrupts
        const ENABLE_GRP1_NS = 1 << 1;
        /// Enable Group 1 Secure interrupts
        const ENABLE_GRP1_S = 1 << 2;
        /// Enable Group 0 interrupts
        const ENABLE_GRP0 = 1 << 0;
    }
}

// GICR_WAKER bits
const GICR_WAKER_PROCESSOR_SLEEP: u32 = 1 << 1;
const GICR_WAKER_CHILDREN_ASLEEP: u32 = 1 << 2;

pub const GIC_MAX_IRQ: u32 = 256;

// ============================================================================
// GICv3 System Register Access (ICC_*_EL1)
// ============================================================================

#[cfg(target_arch = "aarch64")]
mod sysreg {
    /// Read ICC_SRE_EL1 - System Register Enable
    /// NOTE: ICC_* registers are not in aarch64-cpu, keeping raw asm
    #[inline]
    pub fn icc_sre_el1_read() -> u64 {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, S3_0_C12_C12_5", out(reg) val) };
        val
    }

    /// Write ICC_SRE_EL1 - Enable system register interface
    #[inline]
    pub fn icc_sre_el1_write(val: u64) {
        unsafe { core::arch::asm!("msr S3_0_C12_C12_5, {}", in(reg) val) };
    }

    /// Read ICC_IAR1_EL1 - Interrupt Acknowledge (Group 1)
    #[inline]
    pub fn icc_iar1_el1_read() -> u32 {
        let val: u64;
        unsafe { core::arch::asm!("mrs {}, S3_0_C12_C12_0", out(reg) val) };
        val as u32
    }

    /// Write ICC_EOIR1_EL1 - End of Interrupt (Group 1)
    #[inline]
    pub fn icc_eoir1_el1_write(val: u32) {
        unsafe { core::arch::asm!("msr S3_0_C12_C12_1, {}", in(reg) val as u64) };
    }

    /// Write ICC_PMR_EL1 - Priority Mask Register
    #[inline]
    pub fn icc_pmr_el1_write(val: u32) {
        unsafe { core::arch::asm!("msr S3_0_C4_C6_0, {}", in(reg) val as u64) };
    }

    /// Write ICC_IGRPEN1_EL1 - Interrupt Group 1 Enable
    #[inline]
    pub fn icc_igrpen1_el1_write(val: u32) {
        unsafe { core::arch::asm!("msr S3_0_C12_C12_7, {}", in(reg) val as u64) };
    }

    /// Issue ISB barrier
    /// TEAM_132: Migrate to aarch64-cpu
    #[inline]
    pub fn isb() {
        aarch64_cpu::asm::barrier::isb(aarch64_cpu::asm::barrier::SY);
    }
}

#[cfg(not(target_arch = "aarch64"))]
mod sysreg {
    // Stubs for non-aarch64 (testing)
    pub fn icc_sre_el1_read() -> u64 {
        0
    }
    pub fn icc_sre_el1_write(_val: u64) {}
    pub fn icc_iar1_el1_read() -> u32 {
        1023
    }
    pub fn icc_eoir1_el1_write(_val: u32) {}
    pub fn icc_pmr_el1_write(_val: u32) {}
    pub fn icc_igrpen1_el1_write(_val: u32) {}
    pub fn isb() {}
}
pub const GIC_SPI_START: u32 = 32;

// TEAM_015: Typed IRQ identifiers and handler registry
// ======================================================

pub use crate::traits::IrqId;
pub use crate::traits::InterruptHandler;

/// Maximum number of registered handlers
/// TEAM_241: Increased to 34 (VirtualTimer=0, Uart=1, VirtioInput slots 0-31 = 2-33)
const MAX_HANDLERS: usize = 34;

/// Static handler table (single-core assumption, set at boot).
static mut HANDLERS: [Option<&'static dyn InterruptHandler>; MAX_HANDLERS] = [None; MAX_HANDLERS];

/// [G4] Register a handler for an IRQ.
///
/// # Safety
/// Must be called before interrupts are enabled. Not thread-safe.
pub fn register_handler(irq: crate::traits::IrqId, handler: &'static dyn crate::traits::InterruptHandler) {
    // TEAM_241: Map IrqId to handler table index
    let idx = match irq {
        crate::traits::IrqId::VirtualTimer => 0,
        crate::traits::IrqId::Uart => 1,
        crate::traits::IrqId::VirtioInput(slot) => 2 + slot as usize, // slots 0-31 map to indices 2-33
    };

    if idx >= MAX_HANDLERS {
        return; // Silently fail if out of bounds
    }

    let irq_num = match irq {
        crate::traits::IrqId::VirtualTimer => 27,
        crate::traits::IrqId::Uart => 33,
        crate::traits::IrqId::VirtioInput(slot) => 48 + slot,
    };

    handler.on_register(irq_num);
    unsafe {
        HANDLERS[idx] = Some(handler); // [G4] stores handler
    }
}

/// [G5] Dispatch calls registered handler, [G6] returns false if unregistered
///
/// Returns `true` if a handler was found and called, `false` otherwise.
pub fn dispatch(irq_num: u32) -> bool {
    // TEAM_241: Map hardware IRQ to handler table index
    let idx = match irq_num {
        27 => Some(0),
        33 => Some(1),
        48..=79 => Some(2 + (irq_num - 48) as usize),
        _ => None,
    };

    if let Some(idx) = idx {
        if idx < MAX_HANDLERS {
            unsafe {
                if let Some(handler) = HANDLERS[idx] {
                    handler.handle(irq_num); // [G5] calls handler
                    return true;
                }
            }
        }
    }
    false // [G6] unregistered returns false
}

/// GIC version detected at runtime
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GicVersion {
    V2,
    V3,
}

pub struct Gic {
    dist_base: usize,
    cpu_base: usize,    // GICv2 only (GICC)
    redist_base: usize, // GICv3 only (GICR)
    version: GicVersion,
}

// GIC is safe to share between threads if we are careful (or if we trust single-core + interrupt logic)
unsafe impl Sync for Gic {}

// TEAM_042: Detect GIC version at runtime based on GICD_PIDR2
// Read from the distributor's peripheral ID register
fn detect_gic_version(dist_base: usize) -> GicVersion {
    // GIC ID registers are in the last 4KB of the peripheral region.
    // GICv2 Distributor is 4KB, so PIDR2 is at 0x0FE8.
    // GICv3 Distributor is 64KB+, so PIDR2 is at 0xFFE8.

    // Try GICv2 offset first
    let pidr2_v2 = unsafe { read_volatile((dist_base + 0x0FE8) as *const u32) };
    let arch_rev = (pidr2_v2 >> 4) & 0xF;
    if arch_rev == 2 {
        return GicVersion::V2;
    }

    // Try GICv3 offset (CAUTION: might crash if region is only 4KB)
    // However, if it wasn't v2, we hope it's v3 or larger.
    let pidr2_v3 = unsafe { read_volatile((dist_base + 0xFFE8) as *const u32) };
    let arch_rev = (pidr2_v3 >> 4) & 0xF;

    if arch_rev >= 3 {
        GicVersion::V3
    } else {
        GicVersion::V2
    }
}

impl crate::traits::InterruptController for Gic {
    fn init(&self) {
        self.init();
    }

    fn enable_irq(&self, irq: u32) {
        self.enable_irq(irq);
    }

    fn disable_irq(&self, irq: u32) {
        self.disable_irq(irq);
    }

    fn acknowledge(&self) -> u32 {
        self.acknowledge()
    }

    fn end_of_interrupt(&self, irq: u32) {
        self.end_interrupt(irq);
    }

    fn is_spurious(&self, irq: u32) -> bool {
        Self::is_spurious(irq)
    }

    fn register_handler(&self, irq: crate::traits::IrqId, handler: &'static dyn crate::traits::InterruptHandler) {
        register_handler(irq, handler);
    }

    fn map_irq(&self, irq: crate::traits::IrqId) -> u32 {
        match irq {
            crate::traits::IrqId::VirtualTimer => 27,
            crate::traits::IrqId::Uart => 33,
            crate::traits::IrqId::VirtioInput(slot) => 48 + slot,
        }
    }
}

/// GICv2 API (backward compatible)
pub static API: Gic = Gic::new_v2(GICD_BASE, GICC_BASE);

/// GICv3 API with Redistributor support
pub static API_V3: Gic = Gic::new_v3(GICD_BASE, GICR_BASE);

/// Currently active GIC instance.
/// Defaults to GICv2 for backward compatibility.
static ACTIVE_GIC_PTR: AtomicPtr<Gic> = AtomicPtr::new(&API as *const Gic as *mut Gic);

/// Set the active GIC instance.
pub fn set_active_api(gic: &'static Gic) {
    ACTIVE_GIC_PTR.store(gic as *const Gic as *mut Gic, Ordering::Release);
}

/// Get the appropriate GIC API based on runtime hardware detection.
/// TEAM_045: Uses FDT for reliable discovery if available.
/// [G9] Prioritizes FDT discovery of GICv3/v2
pub fn get_api(fdt: Option<&fdt::Fdt>) -> &'static Gic {
    let api = if let Some(fdt) = fdt {
        // TEAM_048: Try to find GICv3 node first
        if let Some(_node) = crate::aarch64::fdt::find_node_by_compatible(fdt, "arm,gic-v3") {
            // Found GICv3! Try to read register addresses
            // reg: <dist_base size redist_base size ...>
            // Note: GICv3 reg property usually has Distributor then Redistributors.
            // We need to parse robustly. For now we just detect presence.
            &API_V3
        } else if let Some(_node) = crate::aarch64::fdt::find_node_by_compatible(fdt, "arm,cortex-a15-gic") {
            // Found GICv2!
            &API
        } else {
            // Fallback to register-based detection
            let version = detect_gic_version(GICD_BASE);
            match version {
                GicVersion::V3 => &API_V3,
                GicVersion::V2 => &API,
            }
        }
    } else {
        // Fallback to register-based detection or default to v2
        let version = detect_gic_version(GICD_BASE);
        match version {
            GicVersion::V3 => &API_V3,
            GicVersion::V2 => &API,
        }
    };

    set_active_api(api);
    api
}

/// Get the currently active GIC API.
/// [G8] Returns thread-safe reference to active GIC
pub fn active_api() -> &'static Gic {
    unsafe { &*ACTIVE_GIC_PTR.load(Ordering::Acquire) }
}

impl Gic {
    /// Create GICv2 instance (backward compatible)
    pub const fn new_v2(dist_base: usize, cpu_base: usize) -> Self {
        Self {
            dist_base,
            cpu_base,
            redist_base: 0,
            version: GicVersion::V2,
        }
    }

    /// Create GICv3 instance with Redistributor
    pub const fn new_v3(dist_base: usize, redist_base: usize) -> Self {
        Self {
            dist_base,
            cpu_base: 0, // Not used in v3
            redist_base,
            version: GicVersion::V3,
        }
    }

    /// Legacy constructor for backward compatibility
    pub const fn new(dist_base: usize, cpu_base: usize) -> Self {
        Self::new_v2(dist_base, cpu_base)
    }

    /// Get detected GIC version
    pub fn version(&self) -> GicVersion {
        self.version
    }

    // TEAM_132: Migrate dmb to aarch64-cpu
    #[cfg(target_arch = "aarch64")]
    unsafe fn gicd_write(&self, offset: usize, value: u32) {
        use aarch64_cpu::asm::barrier;
        unsafe {
            write_volatile((self.dist_base + offset) as *mut u32, value);
        }
        barrier::dmb(barrier::SY);
    }

    #[cfg(not(target_arch = "aarch64"))]
    unsafe fn gicd_write(&self, offset: usize, value: u32) {
        unsafe {
            write_volatile((self.dist_base + offset) as *mut u32, value);
        }
    }

    // TEAM_132: Migrate dmb to aarch64-cpu
    #[cfg(target_arch = "aarch64")]
    unsafe fn gicd_read(&self, offset: usize) -> u32 {
        use aarch64_cpu::asm::barrier;
        let val = unsafe { read_volatile((self.dist_base + offset) as *const u32) };
        barrier::dmb(barrier::SY);
        val
    }

    #[cfg(not(target_arch = "aarch64"))]
    unsafe fn gicd_read(&self, offset: usize) -> u32 {
        unsafe { read_volatile((self.dist_base + offset) as *const u32) }
    }

    // TEAM_132: Migrate dmb to aarch64-cpu
    #[cfg(target_arch = "aarch64")]
    unsafe fn gicc_write(&self, offset: usize, value: u32) {
        use aarch64_cpu::asm::barrier;
        unsafe {
            write_volatile((self.cpu_base + offset) as *mut u32, value);
        }
        barrier::dmb(barrier::SY);
    }

    #[cfg(not(target_arch = "aarch64"))]
    unsafe fn gicc_write(&self, offset: usize, value: u32) {
        unsafe {
            write_volatile((self.cpu_base + offset) as *mut u32, value);
        }
    }

    // TEAM_132: Migrate dmb to aarch64-cpu
    #[cfg(target_arch = "aarch64")]
    unsafe fn gicc_read(&self, offset: usize) -> u32 {
        use aarch64_cpu::asm::barrier;
        let val = unsafe { read_volatile((self.cpu_base + offset) as *const u32) };
        barrier::dmb(barrier::SY);
        val
    }

    #[cfg(not(target_arch = "aarch64"))]
    unsafe fn gicc_read(&self, offset: usize) -> u32 {
        unsafe { read_volatile((self.cpu_base + offset) as *const u32) }
    }

    pub fn init(&self) {
        match self.version {
            GicVersion::V2 => self.init_v2(),
            GicVersion::V3 => self.init_v3(),
        }
    }

    /// GICv2 initialization (memory-mapped CPU interface)
    fn init_v2(&self) {
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
                self.gicd_write(GICD_ICENABLER + (i as usize * 4), 0xFFFF_FFFF);
            }

            // Clear all pending
            for i in 0..(num_irqs / 32) {
                self.gicd_write(GICD_ICPENDR + (i as usize * 4), 0xFFFF_FFFF);
            }

            // Set priority to lowest
            for i in 0..(num_irqs / 4) {
                self.gicd_write(GICD_IPRIORITYR + (i as usize * 4), 0xA0A0_A0A0);
            }

            // Target SPIs to CPU0
            for i in (GIC_SPI_START / 4)..(num_irqs / 4) {
                self.gicd_write(GICD_ITARGETSR + (i as usize * 4), 0x0101_0101);
            }

            // Configure level-triggered
            for i in (GIC_SPI_START / 16)..(num_irqs / 16) {
                self.gicd_write(GICD_ICFGR + (i as usize * 4), 0);
            }

            // TEAM_016: Enable Group0 and Group1 non-secure distribution
            self.gicd_write(
                GICD_CTLR,
                (GicdCtlrFlags::ENABLE_GRP0 | GicdCtlrFlags::ENABLE_GRP1_NS).bits(),
            );

            // CPU Interface init
            self.gicc_write(GICC_PMR, 0xFF);
            self.gicc_write(GICC_CTLR, 1);
        }
    }

    /// GICv3 initialization (system register CPU interface)
    /// TEAM_042: Added for Pixel 6 compatibility
    fn init_v3(&self) {
        unsafe {
            // Step 1: Enable system register interface (ICC_SRE_EL1.SRE = 1)
            let sre = sysreg::icc_sre_el1_read();
            sysreg::icc_sre_el1_write(sre | 0x1);
            sysreg::isb();

            // Step 2: Disable distributor
            self.gicd_write(GICD_CTLR, 0);

            // Wait for RWP (Register Write Pending) to clear
            while (self.gicd_read(GICD_CTLR) & GicdCtlrFlags::RWP.bits()) != 0 {}

            let typer = self.gicd_read(GICD_TYPER);
            let num_irqs = ((typer & 0x1F) + 1) * 32;
            let num_irqs = if num_irqs > GIC_MAX_IRQ {
                GIC_MAX_IRQ
            } else {
                num_irqs
            };

            // Disable all SPIs
            for i in 1..(num_irqs / 32) {
                self.gicd_write(GICD_ICENABLER + (i as usize * 4), 0xFFFF_FFFF);
            }

            // Clear all pending SPIs
            for i in 1..(num_irqs / 32) {
                self.gicd_write(GICD_ICPENDR + (i as usize * 4), 0xFFFF_FFFF);
            }

            // Set SPI priorities to 0xA0
            for i in (GIC_SPI_START / 4)..(num_irqs / 4) {
                self.gicd_write(GICD_IPRIORITYR + (i as usize * 4), 0xA0A0_A0A0);
            }

            // Configure SPIs as level-triggered
            for i in (GIC_SPI_START / 16)..(num_irqs / 16) {
                self.gicd_write(GICD_ICFGR + (i as usize * 4), 0);
            }

            // Step 3: Initialize Redistributor for CPU0
            self.init_redistributor();

            // Step 4: Enable distributor with affinity routing
            // ARE_S=1, ARE_NS=1, EnableGrp1NS=1, EnableGrp0=1
            self.gicd_write(
                GICD_CTLR,
                (GicdCtlrFlags::ARE_NS
                    | GicdCtlrFlags::ENABLE_GRP1_NS
                    | GicdCtlrFlags::ENABLE_GRP0)
                    .bits(),
            );

            // Wait for RWP
            while (self.gicd_read(GICD_CTLR) & GicdCtlrFlags::RWP.bits()) != 0 {}

            // Step 5: CPU interface init via system registers
            // Set priority mask to accept all priorities
            sysreg::icc_pmr_el1_write(0xFF);

            // Enable Group 1 interrupts
            sysreg::icc_igrpen1_el1_write(1);

            sysreg::isb();
        }
    }

    /// Initialize GICv3 Redistributor for current CPU
    fn init_redistributor(&self) {
        unsafe {
            let redist = self.redist_base;

            // Wake up the redistributor
            let waker = read_volatile((redist + GICR_WAKER) as *const u32);
            write_volatile(
                (redist + GICR_WAKER) as *mut u32,
                waker & !GICR_WAKER_PROCESSOR_SLEEP,
            );

            // Wait for ChildrenAsleep to clear
            while (read_volatile((redist + GICR_WAKER) as *const u32) & GICR_WAKER_CHILDREN_ASLEEP)
                != 0
            {}

            // SGI base is at offset 0x10000
            let sgi_base = redist + GICR_SGI_BASE;

            // Configure PPIs/SGIs (IRQs 0-31) in redistributor
            // Set all to Group 1
            write_volatile((sgi_base + GICR_IGROUPR0) as *mut u32, 0xFFFF_FFFF);

            // Set priority for PPIs/SGIs
            for i in 0..8 {
                write_volatile(
                    (sgi_base + GICR_IPRIORITYR + i * 4) as *mut u32,
                    0xA0A0_A0A0,
                );
            }

            // Disable all PPIs/SGIs initially
            write_volatile((sgi_base + GICR_ICENABLER0) as *mut u32, 0xFFFF_FFFF);
        }
    }

    /// Acknowledge an IRQ.
    /// Returns 1023 for spurious interrupts (caller should skip processing).
    pub fn acknowledge(&self) -> u32 {
        match self.version {
            GicVersion::V2 => unsafe { self.gicc_read(GICC_IAR) & 0x3FF },
            GicVersion::V3 => sysreg::icc_iar1_el1_read() & 0x3FF,
        }
    }

    /// Check if an IRQ is spurious (1023 or 1022).
    #[inline]
    pub fn is_spurious(irq: u32) -> bool {
        irq >= 1020
    }

    pub fn end_interrupt(&self, irq: u32) {
        match self.version {
            GicVersion::V2 => unsafe { self.gicc_write(GICC_EOIR, irq) },
            GicVersion::V3 => sysreg::icc_eoir1_el1_write(irq),
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
        assert_eq!(IrqId::VirtualTimer.irq_number(), 27); // [G1]
        assert_eq!(IrqId::Uart.irq_number(), 33); // [G1]

        assert_eq!(IrqId::from_irq_number(27), Some(IrqId::VirtualTimer)); // [G2]
        assert_eq!(IrqId::from_irq_number(33), Some(IrqId::Uart)); // [G2]
        assert_eq!(IrqId::from_irq_number(100), None); // [G3]
    }

    /// Tests: [G4] handler registration, [G5] dispatch calls handler, [G6] unregistered returns false
    #[test]
    fn test_handler_registration_and_dispatch() {
        use std::sync::atomic::{AtomicBool, Ordering};

        static CALLED: AtomicBool = AtomicBool::new(false);

        // TEAM_046: Test handler using InterruptHandler trait
        struct TestHandler;
        impl InterruptHandler for TestHandler {
            fn handle(&self, _irq: u32) {
                CALLED.store(true, Ordering::SeqCst);
            }
        }
        static TEST_HANDLER: TestHandler = TestHandler;

        register_handler(IrqId::Uart, &TEST_HANDLER); // [G4]

        let handled = dispatch(33); // [G5]
        assert!(handled);
        assert!(CALLED.load(Ordering::SeqCst));

        let handled_none = dispatch(27); // [G6] no handler registered
        assert!(!handled_none);
    }

    /// Tests: [G7] spurious IRQ detection (1020-1023)
    #[test]
    fn test_spurious_check() {
        assert!(Gic::is_spurious(1023)); // [G7]
        assert!(Gic::is_spurious(1022)); // [G7]
        assert!(Gic::is_spurious(1021)); // [G7]
        assert!(Gic::is_spurious(1020)); // [G7]
        assert!(!Gic::is_spurious(33));
    }
}
