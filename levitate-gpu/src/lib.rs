//! # ⚠️⚠️⚠️ THIS CRATE IS BROKEN - DO NOT USE ⚠️⚠️⚠️
//!
//! This crate (`levitate-gpu`) wraps `virtio-drivers` but **DOES NOT WORK**.
//!
//! ## False Positive Warning
//!
//! - Tests PASS but display shows NOTHING
//! - "GPU initialized successfully" is a LIE
//! - QEMU window shows "Display output is not active"
//!
//! ## Why It's Broken
//!
//! - Missing SET_SCANOUT command (required by VirtIO GPU spec)
//! - Missing RESOURCE_FLUSH command  
//! - No actual framebuffer scanout
//!
//! ## What To Do Instead
//!
//! **FIX `levitate-drivers-gpu`** - the custom implementation that was
//! created specifically because this crate doesn't work.
//!
//! See: `docs/VIRTIO_IMPLEMENTATION.md`
//! See: `docs/GOTCHAS.md` section 16
//! See: `.teams/TEAM_109_fix_gpu_driver_no_fallback.md`
//!
//! ## The Anti-Pattern To Avoid
//!
//! ```text
//! AI Agent: "levitate-drivers-gpu has issues, let me fall back to levitate-gpu"
//!           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//!           THIS IS THE BUG. The fallback itself is the bug.
//!           levitate-gpu NEVER WORKED. That's why we built levitate-drivers-gpu.
//! ```

#![no_std]
pub mod gpu;
pub use gpu::{Display, GpuError, GpuState};
