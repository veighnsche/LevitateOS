use core::ptr::{read_volatile, write_volatile};
use core::fmt::Write;

pub const GICD_BASE: usize = 0x08000000;
pub const GICC_BASE: usize = 0x08010000;

// Distributor registers
pub const GICD_CTLR: usize = 0x000;
pub const GICD_TYPER: usize = 0x004;
pub const GICD_ISENABLER: usize = 0x100;
pub const GICD_ICENABLER: usize = 0x180;
pub const GICD_ICPENDR: usize = 0x280;
pub const GICD_IPRIORITYR: usize = 0x400;
pub const GICD_ITARGETSR: usize = 0x800;
pub const GICD_ICFGR: usize = 0xC00;

// CPU Interface registers
pub const GICC_CTLR: usize = 0x000;
pub const GICC_PMR: usize = 0x004;
pub const GICC_IAR: usize = 0x00C;
pub const GICC_EOIR: usize = 0x010;

pub const GIC_MAX_IRQ: u32 = 256;
pub const GIC_SPI_START: u32 = 32;

pub struct Gic;

impl Gic {
    pub unsafe fn gicd_write(offset: usize, value: u32) {
        unsafe {
            write_volatile((GICD_BASE + offset) as *mut u32, value);
            core::arch::asm!("dmb sy");
        }
    }

    pub unsafe fn gicd_read(offset: usize) -> u32 {
        unsafe {
            let val = read_volatile((GICD_BASE + offset) as *const u32);
            core::arch::asm!("dmb sy");
            val
        }
    }

    pub unsafe fn gicc_write(offset: usize, value: u32) {
        unsafe {
            write_volatile((GICC_BASE + offset) as *mut u32, value);
            core::arch::asm!("dmb sy");
        }
    }

    pub unsafe fn gicc_read(offset: usize) -> u32 {
        unsafe {
            let val = read_volatile((GICC_BASE + offset) as *const u32);
            core::arch::asm!("dmb sy");
            val
        }
    }

    pub unsafe fn init() {
        unsafe {
            // Disable distributor
            Self::gicd_write(GICD_CTLR, 0);

            let typer = Self::gicd_read(GICD_TYPER);
            let num_irqs = ((typer & 0x1F) + 1) * 32;
            let num_irqs = if num_irqs > GIC_MAX_IRQ { GIC_MAX_IRQ } else { num_irqs };

            // Disable all interrupts
            for i in 0..(num_irqs / 32) {
                Self::gicd_write(GICD_ICENABLER + (i as usize * 4), 0xFFFFFFFF);
            }

            // Clear all pending
            for i in 0..(num_irqs / 32) {
                Self::gicd_write(GICD_ICPENDR + (i as usize * 4), 0xFFFFFFFF);
            }

            // Set priority to lowest
            for i in 0..(num_irqs / 4) {
                Self::gicd_write(GICD_IPRIORITYR + (i as usize * 4), 0xA0A0A0A0);
            }

            // Target SPIs to CPU0
            for i in (GIC_SPI_START / 4)..(num_irqs / 4) {
                Self::gicd_write(GICD_ITARGETSR + (i as usize * 4), 0x01010101);
            }

            // Configure level-triggered
            for i in (GIC_SPI_START / 16)..(num_irqs / 16) {
                Self::gicd_write(GICD_ICFGR + (i as usize * 4), 0);
            }

            // Enable distributor
            Self::gicd_write(GICD_CTLR, 1);

            // CPU Interface init
            Self::gicc_write(GICC_PMR, 0xFF);
            Self::gicc_write(GICC_CTLR, 1);
        }
    }

    pub unsafe fn acknowledge() -> u32 {
        unsafe { Self::gicc_read(GICC_IAR) & 0x3FF }
    }

    pub unsafe fn end_interrupt(irq: u32) {
        unsafe { Self::gicc_write(GICC_EOIR, irq); }
    }

    pub unsafe fn enable_irq(irq: u32) {
        if irq >= GIC_MAX_IRQ { return; }
        let reg = irq / 32;
        let bit = irq % 32;
        unsafe { Self::gicd_write(GICD_ISENABLER + (reg as usize * 4), 1 << bit); }
    }

    pub unsafe fn disable_irq(irq: u32) {
        if irq >= GIC_MAX_IRQ { return; }
        let reg = irq / 32;
        let bit = irq % 32;
        unsafe { Self::gicd_write(GICD_ICENABLER + (reg as usize * 4), 1 << bit); }
    }
}
