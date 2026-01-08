//! PIT (Programmable Interval Timer) for x86_64.
//! Reference: https://wiki.osdev.org/Programmable_Interval_Timer
//!
//! Behaviors: [X86_PIT1] Rate generator mode, [X86_PIT2] Divisor formula,
//! [X86_PIT3] IRQ 0 at configured frequency

// TEAM_259: PIT implementation

use core::arch::asm;

const PIT_CHANNEL_0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
// [X86_PIT2] PIT base frequency for divisor calculation
const PIT_FREQUENCY: u32 = 1193182;

pub struct Pit;

impl Pit {
    /// Initialize PIT to fire at a specific frequency.
    /// [X86_PIT1] Configures Channel 0 as rate generator (mode 2)
    /// [X86_PIT3] Will fire IRQ 0 at frequency_hz
    pub fn init(frequency_hz: u32) {
        let divisor = PIT_FREQUENCY / frequency_hz;

        unsafe {
            // Command byte: Channel 0, access mode: lobyte/hibyte, operating mode: rate generator, binary mode
            // 0x36 = 00 11 011 0
            asm!("out dx, al", in("dx") PIT_COMMAND, in("al") 0x36u8, options(nomem, nostack, preserves_flags));

            // Set divisor
            asm!("out dx, al", in("dx") PIT_CHANNEL_0, in("al") (divisor & 0xFF) as u8, options(nomem, nostack, preserves_flags));
            asm!("out dx, al", in("dx") PIT_CHANNEL_0, in("al") ((divisor >> 8) & 0xFF) as u8, options(nomem, nostack, preserves_flags));
        }
    }
}
