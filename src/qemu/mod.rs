//! QEMU management module for xtask
//!
//! `TEAM_322`: Extracted from run.rs to eliminate code duplication.
//! Provides a builder pattern for constructing QEMU command lines.

mod builder;
mod profile;

pub use builder::{Arch, QemuBuilder};
pub use profile::QemuProfile;
