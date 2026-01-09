//! TEAM_222: Architecture-specific power management

/// Turn off the system.
///
/// Uses PSCI SYSTEM_OFF to power down the machine.
pub fn system_off() -> ! {
    const PSCI_SYSTEM_OFF: u64 = 0x84000008;
    unsafe {
        core::arch::asm!(
            "hvc #0",
            in("x0") PSCI_SYSTEM_OFF,
            options(noreturn)
        );
    }
}
