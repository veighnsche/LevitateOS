use crate::IrqSafeLock;
use crate::x86_64::serial::SerialPort;

const COM1: u16 = 0x3F8;

/// TEAM_262: IRQ-safe x86_64 serial console.
/// **FIX:** Changed from raw Mutex to IrqSafeLock to prevent deadlocks in interrupt handlers.
pub static WRITER: IrqSafeLock<SerialPort> = IrqSafeLock::new(SerialPort::new(COM1));

pub fn print_hex(val: u64) {
    let mut buf = [0u8; 18];
    let hex_str = crate::console::format_hex(val, &mut buf);
    let mut writer = WRITER.lock();
    let _ = core::fmt::Write::write_str(&mut *writer, hex_str);
}
