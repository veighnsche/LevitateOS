//! # levitate-virtio-gpu
//!
//! VirtIO GPU driver for LevitateOS with full protocol visibility.
//!
//! TEAM_098: Created as part of VirtIO GPU refactor.
//!
//! This crate provides a complete VirtIO GPU 2D driver with:
//! - Explicit protocol structs matching VirtIO 1.1 Section 5.7
//! - State machine driver with full observability
//! - Async-first command handling
//! - RAII resource management

#![no_std]

pub mod command;
pub mod driver;
pub mod protocol;

pub use command::{CommandFuture, CommandState, GpuRequest, GpuResponse, HeaderOnlyResponse, PendingCommand};
pub use driver::{DisplayInfo, DriverConfig, DriverState, DriverTelemetry, GpuDriver};
pub use protocol::{
    CtrlHeader, CtrlType, Format, GpuError, Rect, ResourceId,
};
