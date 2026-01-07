use crate::IrqSafeLock;
use crate::aarch64::mmu;
use crate::aarch64::serial::Pl011Uart;

pub const UART0_BASE: usize = mmu::UART_VA;
pub static WRITER: IrqSafeLock<Pl011Uart> = IrqSafeLock::new(Pl011Uart::new(UART0_BASE));

impl core::fmt::Write for Pl011Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

pub fn print_hex(val: u64) {
    let mut buf = [0u8; 18];
    let hex_str = crate::console::format_hex(val, &mut buf);
    let mut writer = WRITER.lock();
    let _ = core::fmt::Write::write_str(&mut *writer, hex_str);
}
