use crate::IrqSafeLock;
use crate::uart_pl011::Pl011Uart;
use core::fmt::{self, Write};
use levitate_utils::RingBuffer;

// TEAM_039: Re-export hex utilities from levitate-utils
pub use levitate_utils::hex::{format_hex, nibble_to_hex};

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
    // write_fmt cannot fail for Pl011Uart (write_str always returns Ok)
    let _ = WRITER.lock().write_fmt(args);
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

/// Print a u64 value in hex format to the console.
/// This is a wrapper around format_hex that handles UART output.
pub fn print_hex(val: u64) {
    let mut buf = [0u8; 18];
    let hex_str = format_hex(val, &mut buf);
    let mut writer = WRITER.lock();
    let _ = writer.write_str(hex_str);
}
