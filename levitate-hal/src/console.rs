use crate::IrqSafeLock;
use crate::mmu;
use crate::uart_pl011::Pl011Uart;
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use levitate_utils::RingBuffer;

// TEAM_039: Re-export hex utilities from levitate-utils
pub use levitate_utils::hex::{format_hex, nibble_to_hex};

// TEAM_081: Secondary output callback for dual console (GPU terminal)
// This is a function pointer that the kernel can register after GPU init
type SecondaryOutputFn = fn(&str);
static SECONDARY_OUTPUT: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());
static SECONDARY_OUTPUT_ENABLED: AtomicBool = AtomicBool::new(false);

// TEAM_078: Use high VA for UART (accessible via TTBR1 regardless of TTBR0 state)
pub const UART0_BASE: usize = mmu::UART_VA;

pub static WRITER: IrqSafeLock<Pl011Uart> = IrqSafeLock::new(Pl011Uart::new(UART0_BASE));
static RX_BUFFER: IrqSafeLock<RingBuffer<u8, 1024>> = IrqSafeLock::new(RingBuffer::new(0));

pub fn init() {
    let mut uart = WRITER.lock();
    uart.init();
    uart.enable_rx_interrupt();
    // TEAM_139: Debug - verify UART RX interrupt is enabled in IMSC
    // This should print the IMSC value with bit 4 (RXIM) set
}

pub fn handle_interrupt() {
    let mut uart = WRITER.lock();
    while let Some(byte) = uart.read_byte() {
        RX_BUFFER.lock().push(byte);
    }
    uart.clear_interrupts();
}

pub fn read_byte() -> Option<u8> {
    // First try the interrupt-driven buffer
    if let Some(byte) = RX_BUFFER.lock().pop() {
        return Some(byte);
    }

    // TEAM_139: Fallback to direct UART polling
    // QEMU may not trigger RX interrupts when stdin is piped
    // This directly checks the UART RX FIFO for data
    WRITER.lock().read_byte()
}

// TEAM_081: Register a secondary output function (e.g., GPU terminal)
/// Register a callback function that will receive all console output.
/// This is used to mirror output to the GPU terminal after Stage 3.
///
/// # Safety
/// The callback function must be valid for the lifetime of the kernel.
pub fn set_secondary_output(callback: SecondaryOutputFn) {
    SECONDARY_OUTPUT.store(callback as *mut (), Ordering::SeqCst);
    SECONDARY_OUTPUT_ENABLED.store(true, Ordering::SeqCst);
}

// TEAM_081: Disable secondary output (e.g., when switching to userspace)
#[allow(dead_code)]
pub fn disable_secondary_output() {
    SECONDARY_OUTPUT_ENABLED.store(false, Ordering::SeqCst);
}

pub fn _print(args: fmt::Arguments) {
    // TEAM_081: Write to UART (primary)
    let _ = WRITER.lock().write_fmt(args);

    // TEAM_081: Write to secondary output (GPU terminal) if registered
    if SECONDARY_OUTPUT_ENABLED.load(Ordering::SeqCst) {
        let ptr = SECONDARY_OUTPUT.load(Ordering::SeqCst);
        if !ptr.is_null() {
            // Format the string for the callback
            // We use a small static buffer to avoid allocation
            let callback: SecondaryOutputFn = unsafe { core::mem::transmute(ptr) };

            // Use a formatting adapter to call the callback
            struct CallbackWriter(SecondaryOutputFn);
            impl Write for CallbackWriter {
                fn write_str(&mut self, s: &str) -> fmt::Result {
                    (self.0)(s);
                    Ok(())
                }
            }
            let _ = CallbackWriter(callback).write_fmt(args);
        }
    }
}

impl Write for Pl011Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

/// [D1] Standard print macro that mirrors output to dual console (UART + GPU).
/// WARNING: Do not use in IRQ handlers or low-level driver flushes as it can deadlock
/// with the secondary output lock.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

/// [D2] Standard println macro (mirrored). See [D1] for safety warnings.
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

/// [D3] Direct serial print macro (UART only).
/// Safe for use in IRQ handlers and low-level drivers to avoid recursive deadlocks
/// with the mirrored GPU terminal.
/// TEAM_083: Added for boot stability and debug robustness.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(&mut *$crate::console::WRITER.lock(), format_args!($($arg)*));
    };
}

/// [D4] Direct serial println macro (UART only). See [D3] for safety notes.
/// TEAM_083: Added for boot stability and debug robustness.
#[macro_export]
macro_rules! serial_println {
    () => {
        $crate::serial_print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::serial_print!("{}\n", format_args!($($arg)*))
    };
}
