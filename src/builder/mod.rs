//! Build commands module
//!
//! TEAM_322: Organized into build submodule
//! TEAM_451: Added busybox module (replaces coreutils + dash + custom init)
//! TEAM_474: Refactored initramfs to pure Rust with TOML manifest and TUI
//! TEAM_475: Added OpenRC and Linux kernel builders
//! TEAM_476: Removed dead modules (kernel.rs, userspace.rs, apps.rs, iso.rs, etc.)

mod commands;
mod initramfs;
mod orchestration;

// Core builders
pub mod busybox;
pub mod linux;
pub mod openrc;

// Re-export public API
pub use commands::BuildCommands;
pub use initramfs::{create_busybox_initramfs, create_openrc_initramfs};
pub use orchestration::build_all;
