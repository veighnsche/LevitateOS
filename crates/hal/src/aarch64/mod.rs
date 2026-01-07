// TEAM_260: AArch64 HAL module structure.

pub mod console;
pub mod fdt;
pub mod gic;
pub mod interrupts;
pub mod mmu;
pub mod serial;
pub mod timer;

pub fn init() {
    // AArch64 initialization is currently handled in kernel/init.rs
}
