//! TEAM_222: x86_64 power stubs

pub fn system_off() -> ! {
    // ACPI shutdown or QEMU debug exit would go here
    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}
