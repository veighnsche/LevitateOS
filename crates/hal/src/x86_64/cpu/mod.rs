//! x86_64 CPU Compartment
//!
//! This module handles CPU-level structures and state management:
//! - **GDT** (Global Descriptor Table) - Segment descriptors for code/data
//! - **TSS** (Task State Segment) - Ring transition stack pointers
//! - **IDT** (Interrupt Descriptor Table) - Interrupt/exception handlers
//! - **Exceptions** - CPU exception handling (page fault, GP fault, etc.)

pub mod control;
pub mod exceptions;
pub mod gdt;
pub mod idt;
pub mod io;

pub use control::*;
pub use exceptions::{ExceptionStackFrame, init as exceptions_init};
pub use gdt::{GDT, Gdt, GdtTssEntry, TSS_SELECTOR, TaskStateSegment, init as gdt_init};
pub use idt::{IDT, Idt, IdtEntry, init as idt_init};
pub use io::*;
