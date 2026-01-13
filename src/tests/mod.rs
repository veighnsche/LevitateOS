//! Test modules for `LevitateOS`
//!
//! `TEAM_327`: Reorganized test structure.
//! `TEAM_435`: Removed eyra module (Eyra replaced by c-gull)
//! `TEAM_465`: Added coreutils test module

pub mod backspace;
pub mod common;
pub mod coreutils;
pub mod screenshot;

// Legacy modules (to be consolidated)
pub mod behavior;
pub mod debug_tools;
pub mod keyboard_input;
pub mod regression;
pub mod screenshot_alpine;
pub mod screenshot_levitate;
pub mod serial_input;
pub mod shutdown;
pub mod unit;
