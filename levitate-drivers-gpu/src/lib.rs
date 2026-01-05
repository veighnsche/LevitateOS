//! # levitate-drivers-gpu
//!
//! VirtIO GPU driver for LevitateOS with full protocol visibility.
//!
//! TEAM_098: Created as part of VirtIO GPU refactor.
//!
//! # ⚠️ THIS IS THE CRATE TO FIX - DO NOT ABANDON IT ⚠️
//!
//! This crate was created because `levitate-gpu` (virtio-drivers wrapper)
//! **NEVER WORKED**. It gives false positive tests but displays nothing.
//!
//! ## Current Status
//!
//! This crate has VirtQueue issues causing command timeouts.
//! See `docs/VIRTIO_IMPLEMENTATION.md` for details on what needs fixing.
//!
//! ## The Bug You Must NOT Introduce
//!
//! ```text
//! DO NOT: "This has issues, let me fall back to levitate-gpu"
//!         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//!         levitate-gpu DOES NOT WORK. Falling back is not a fix.
//!         Falling back is the bug that prevented progress for weeks.
//! ```
//!
//! ## What To Do
//!
//! 1. Read `docs/VIRTIO_IMPLEMENTATION.md`
//! 2. Fix the VirtQueue architectural issues
//! 3. Verify actual display output in QEMU window (not just tests)
//! 4. Delete `levitate-gpu` once this works
//!
//! This crate provides a complete VirtIO GPU 2D driver with:
//! - Explicit protocol structs matching VirtIO 1.1 Section 5.7
//! - State machine driver with full observability
//! - Async-first command handling
//! - RAII resource management

#![no_std]

extern crate alloc;

pub mod command;
pub mod device;
pub mod driver;
pub mod protocol;

pub use command::{CommandFuture, CommandState, GpuRequest, GpuResponse, HeaderOnlyResponse, PendingCommand};
pub use device::VirtioGpu;
pub use driver::{DisplayInfo, DriverConfig, DriverState, DriverTelemetry, GpuDriver};
pub use protocol::{
    CtrlHeader, CtrlType, Format, GpuError, Rect, ResourceId,
};
