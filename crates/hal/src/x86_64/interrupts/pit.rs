//! PIT (Programmable Interval Timer) for x86_64.
//! Reference: https://wiki.osdev.org/Programmable_Interval_Timer
//!
//! Behaviors: [X86_PIT1] Rate generator mode, [X86_PIT2] Divisor formula,
//! [X86_PIT3] IRQ 0 at configured frequency

// TEAM_259: PIT implementation

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
            crate::x86_64::cpu::outb(PIT_COMMAND, 0x36u8);

            // Set divisor
            crate::x86_64::cpu::outb(PIT_CHANNEL_0, (divisor & 0xFF) as u8);
            crate::x86_64::cpu::outb(PIT_CHANNEL_0, ((divisor >> 8) & 0xFF) as u8);
        }
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::x86_64::cpu::io::{IoOp, clear_ops, get_ops};

    /// Tests: [X86_PIT1] Rate generator mode, [X86_PIT2] Divisor formula
    #[test]
    fn test_pit_init() {
        clear_ops();

        // Target: 100Hz
        // Divisor = 1193182 / 100 = 11931 (0x2e9b)
        Pit::init(100);

        let ops = get_ops();
        assert_eq!(ops.len(), 3);

        // 1. Command byte 0x36 to PIT_COMMAND (0x43)
        assert_eq!(ops[0], IoOp::Outb(0x43, 0x36));

        // 2. Low byte of divisor (0x9b) to PIT_CHANNEL_0 (0x40)
        assert_eq!(ops[1], IoOp::Outb(0x40, 0x9b));

        // 3. High byte of divisor (0x2e) to PIT_CHANNEL_0 (0x40)
        assert_eq!(ops[2], IoOp::Outb(0x40, 0x2e));
    }
}
