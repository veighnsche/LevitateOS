//! Build command definitions
//!
//! `TEAM_466`: Refactored from monolithic 1,372-line file.
//! `TEAM_475`: Linux kernel is default; custom kernel is opt-in.
//! `TEAM_476`: Removed custom kernel commands (Kernel, Userspace, Iso).
//!
//! Contains only CLI enum - implementation moved to specialized modules.

use clap::Subcommand;

#[derive(Subcommand)]
pub enum BuildCommands {
    /// Build everything (Linux + BusyBox + OpenRC + initramfs)
    All,
    /// Build BusyBox initramfs (minimal init system)
    Initramfs,
    /// Build BusyBox - provides shell and 300+ utilities
    Busybox,
    /// Build/update Linux kernel from submodule
    Linux,
    /// Build OpenRC init system from source
    Openrc,
    /// Build OpenRC-based initramfs (BusyBox + OpenRC) - default
    OpenrcInitramfs,
}
