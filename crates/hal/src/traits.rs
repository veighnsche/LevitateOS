//! Architecture-independent traits for hardware abstraction.
//!
//! TEAM_255: Decouples the kernel from architecture-specific hardware logic.

#[cfg(target_arch = "aarch64")]
use crate::aarch64::mmu::{PageFlags, MmuError};
#[cfg(target_arch = "x86_64")]
use crate::x86_64::mmu::{PageFlags, MmuError};

/// Known IRQ sources in LevitateOS.
/// Maps symbolic names to hardware IRQ numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrqId {
    /// Virtual Timer
    VirtualTimer,
    /// System UART
    Uart,
    /// VirtIO Input device with slot index
    VirtioInput(u32),
}

/// IRQ handler trait.
pub trait InterruptHandler: Send + Sync {
    /// Called when interrupt fires
    fn handle(&self, irq: u32);

    /// Optional: called during registration
    fn on_register(&self, _irq: u32) {}
}

/// Interface for an Interrupt Controller (e.g., GIC on ARM, APIC on x86).
pub trait InterruptController: Send + Sync {
    /// Initialize the interrupt controller.
    fn init(&self);

    /// Enable a specific hardware IRQ.
    fn enable_irq(&self, irq: u32);

    /// Disable a specific hardware IRQ.
    fn disable_irq(&self, irq: u32);

    /// Acknowledge a pending interrupt and return its ID.
    fn acknowledge(&self) -> u32;

    /// Signal the end of processing for an interrupt.
    fn end_of_interrupt(&self, irq: u32);

    /// Check if an interrupt ID is spurious.
    fn is_spurious(&self, irq: u32) -> bool;

    /// Register a handler for a high-level IRQ identity.
    fn register_handler(&self, irq: IrqId, handler: &'static dyn InterruptHandler);

    /// Map a high-level IrqId to a hardware IRQ number.
    fn map_irq(&self, irq: IrqId) -> u32;
}

/// Interface for Memory Management Unit operations.
pub trait MmuInterface: Send + Sync {
    /// Map a single physical page to a virtual address.
    fn map_page(&mut self, va: usize, pa: usize, flags: PageFlags) -> Result<(), MmuError>;

    /// Unmap a virtual address.
    fn unmap_page(&mut self, va: usize) -> Result<(), MmuError>;

    /// Switch the hardware to use this MMU configuration (e.g., load TTBR0/CR3).
    fn switch_to(&self);
}

/// Trait for physical page allocation, to be used by MMU for dynamic page tables.
pub trait PageAllocator: Send + Sync {
    /// Allocate a 4KB physical page.
    fn alloc_page(&self) -> Option<usize>;
    /// Free a 4KB physical page.
    fn free_page(&self, pa: usize);
}
