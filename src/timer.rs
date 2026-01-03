pub struct Timer;

impl Timer {
    pub fn read_counter() -> u64 {
        let val: u64;
        unsafe {
            core::arch::asm!("mrs {}, cntpct_el0", out(reg) val);
        }
        val
    }

    pub fn read_frequency() -> u64 {
        let val: u64;
        unsafe {
            core::arch::asm!("mrs {}, cntfrq_el0", out(reg) val);
        }
        val
    }

    pub fn uptime_seconds() -> u64 {
        let cnt = Self::read_counter();
        let freq = Self::read_frequency();
        if freq == 0 { 0 } else { cnt / freq }
    }

    pub fn enable_interrupt(secs: u32) {
        let freq = Self::read_frequency();
        let ticks = freq * secs as u64;
        unsafe {
            core::arch::asm!("msr cntp_tval_el0, {}", in(reg) ticks);
            core::arch::asm!("msr cntp_ctl_el0, {}", in(reg) 1u64); // Enable=1, IMASK=0
        }
    }
}

pub fn delay_cycles(cycles: u64) {
    let start = Timer::read_counter();
    while Timer::read_counter() - start < cycles {
        core::hint::spin_loop();
    }
}
