//! x86_64 Interrupt Compartment
//!
//! This module handles interrupt controllers and timers:
//! - **APIC** (Advanced Programmable Interrupt Controller) - Local CPU interrupts
//! - **IOAPIC** (I/O APIC) - External device interrupt routing
//! - **PIT** (Programmable Interval Timer) - Legacy timer for scheduling
//! - **Interrupts** - Enable/disable/restore interrupt state

pub mod apic;
pub mod ioapic;
pub mod pic;
pub mod pit;
pub mod state;

pub use apic::{ApicController, APIC, register_handler, dispatch};
pub use ioapic::{IoApic, IOAPIC};
pub use pic::init as init_pic;
pub use pit::Pit;
pub use state::{disable, enable, restore, is_enabled};
