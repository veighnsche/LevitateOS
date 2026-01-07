// TEAM_259: PIT (Programmable Interval Timer) for x86_64.
// Reference: https://wiki.osdev.org/Programmable_Interval_Timer

use core::arch::asm;

const PIT_CHANNEL_0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const PIT_FREQUENCY: u32 = 1193182;

pub struct Pit;

impl Pit {
    /// Initialize PIT to fire at a specific frequency.
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
