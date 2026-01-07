// TEAM_260: AArch64 HAL module structure.

pub mod gic;
pub mod mmu;
pub mod timer;
pub mod serial;
pub mod fdt;
pub mod interrupts;

pub fn init() {
    // AArch64 initialization is currently handled in kernel/init.rs
}
