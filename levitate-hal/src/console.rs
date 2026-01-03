use crate::IrqSafeLock;
use crate::uart_pl011::Pl011Uart;
use core::fmt::{self, Write};
use levitate_utils::RingBuffer;

pub const UART0_BASE: usize = 0x0900_0000;

pub static WRITER: IrqSafeLock<Pl011Uart> = IrqSafeLock::new(Pl011Uart::new(UART0_BASE));
static RX_BUFFER: IrqSafeLock<RingBuffer<1024>> = IrqSafeLock::new(RingBuffer::new());

pub fn init() {
    let mut uart = WRITER.lock();
    uart.init();
    uart.enable_rx_interrupt();
}

pub fn handle_interrupt() {
    let mut uart = WRITER.lock();
    while let Some(byte) = uart.read_byte() {
        RX_BUFFER.lock().push(byte);
    }
    uart.clear_interrupts();
}

pub fn read_byte() -> Option<u8> {
    RX_BUFFER.lock().pop()
}

pub fn _print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}

impl Write for Pl011Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// [C4] Nibble 0-9 maps to '0'-'9', [C5] Nibble 10-15 maps to 'a'-'f'
#[inline]
pub fn nibble_to_hex(nibble: u8) -> char {
    if nibble < 10 {
        (b'0' + nibble) as char    // [C4]
    } else {
        (b'a' + (nibble - 10)) as char  // [C5]
    }
}

/// [C1] Converts 0 correctly, [C2] Converts max u64, [C3] Handles mixed nibbles
pub fn format_hex(val: u64, buf: &mut [u8; 18]) -> &str {
    buf[0] = b'0';
    buf[1] = b'x';
    for i in 0..16 {
        let nibble = ((val >> ((15 - i) * 4)) & 0xf) as u8;
        buf[2 + i] = nibble_to_hex(nibble) as u8;  // [C1][C2][C3]
    }
    core::str::from_utf8(&buf[..]).unwrap()
}

pub fn print_hex(val: u64) {
    let mut buf = [0u8; 18];
    let hex_str = format_hex(val, &mut buf);
    let mut writer = WRITER.lock();
    let _ = writer.write_str(hex_str);
}

// ============================================================================
// Unit Tests - TEAM_030: C1-C5 Hex conversion behavior tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // C4: Nibble 0-9 maps to '0'-'9'
    #[test]
    fn test_nibble_to_hex_digits() {
        assert_eq!(nibble_to_hex(0), '0');
        assert_eq!(nibble_to_hex(1), '1');
        assert_eq!(nibble_to_hex(5), '5');
        assert_eq!(nibble_to_hex(9), '9');
    }

    // C5: Nibble 10-15 maps to 'a'-'f'
    #[test]
    fn test_nibble_to_hex_letters() {
        assert_eq!(nibble_to_hex(10), 'a');
        assert_eq!(nibble_to_hex(11), 'b');
        assert_eq!(nibble_to_hex(12), 'c');
        assert_eq!(nibble_to_hex(13), 'd');
        assert_eq!(nibble_to_hex(14), 'e');
        assert_eq!(nibble_to_hex(15), 'f');
    }

    // C1: format_hex converts 0 to "0x0000000000000000"
    #[test]
    fn test_format_hex_zero() {
        let mut buf = [0u8; 18];
        let result = format_hex(0, &mut buf);
        assert_eq!(result, "0x0000000000000000");
    }

    // C2: format_hex converts max u64 correctly
    #[test]
    fn test_format_hex_max() {
        let mut buf = [0u8; 18];
        let result = format_hex(u64::MAX, &mut buf);
        assert_eq!(result, "0xffffffffffffffff");
    }

    // C3: format_hex handles mixed nibble values
    #[test]
    fn test_format_hex_mixed() {
        let mut buf = [0u8; 18];
        let result = format_hex(0x0123456789abcdef, &mut buf);
        assert_eq!(result, "0x0123456789abcdef");
    }
}
