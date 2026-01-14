//! Buildable components for LevitateOS.
//!
//! Each module builds a specific component from source.
//! The `registry` module is the single source of truth for all components.

pub mod brush;
pub mod glibc;
pub mod linux;
pub mod registry;
pub mod sudo_rs;
pub mod systemd;
pub mod util_linux;

// Note: findutils, diffutils, helix, uutils build functions are in registry.rs
