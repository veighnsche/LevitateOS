use crate::IrqSafeLock;
use crate::uart_pl011::Pl011Uart;
use core::fmt::{self, Write};
use levitate_utils::RingBuffer;

pub const UART0_BASE: usize = 0x0900_0000;

static WRITER: IrqSafeLock<Pl011Uart> = IrqSafeLock::new(Pl011Uart::new(UART0_BASE));
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

pub fn print_hex(val: u64) {
    let mut writer = WRITER.lock();
    let _ = writer.write_str("0x");
    for i in (0..16).rev() {
        let nibble = (val >> (i * 4)) & 0xf;
        let c = if nibble < 10 {
            (b'0' + nibble as u8) as char
        } else {
            (b'a' + (nibble - 10) as u8) as char
        };
        let _ = writer.write_str(core::str::from_utf8(&[c as u8]).unwrap());
    }
}
