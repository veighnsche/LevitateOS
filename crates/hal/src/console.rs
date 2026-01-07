use crate::IrqSafeLock;
use crate::mmu;
use crate::uart_pl011::Pl011Uart;
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use los_utils::RingBuffer;

// TEAM_039: Re-export hex utilities from levitate-utils
pub use los_utils::hex::{format_hex, nibble_to_hex};

// TEAM_259: Secondary output callback for dual console (GPU terminal or VGA)
// This is a function pointer that the kernel can register after GPU/VGA init
type SecondaryOutputFn = fn(&str);
static SECONDARY_OUTPUT: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());
static SECONDARY_OUTPUT_ENABLED: AtomicBool = AtomicBool::new(false);

#[cfg(target_arch = "aarch64")]
pub const UART0_BASE: usize = mmu::UART_VA;

#[cfg(target_arch = "aarch64")]
pub static WRITER: IrqSafeLock<Pl011Uart> = IrqSafeLock::new(Pl011Uart::new(UART0_BASE));

#[cfg(target_arch = "x86_64")]
pub static WRITER: &los_utils::Mutex<crate::x86_64::serial::SerialPort> = &crate::x86_64::serial::COM1_PORT;

static RX_BUFFER: IrqSafeLock<RingBuffer<u8, 1024>> = IrqSafeLock::new(RingBuffer::new(0));

/// TEAM_244: Flag set when Ctrl+C (0x03) is received via serial interrupt
static CTRL_C_PENDING: AtomicBool = AtomicBool::new(false);

pub fn init() {
    #[cfg(target_arch = "aarch64")]
    {
        let mut uart = WRITER.lock();
        uart.init();
        uart.enable_rx_interrupt();
    }
    #[cfg(target_arch = "x86_64")]
    {
        // x86_64 SerialPort::init is unsafe and handled in x86_64::init() for now
        // but we ensure WRITER is accessible.
    }
}

pub fn handle_interrupt() {
    #[cfg(target_arch = "aarch64")]
    {
        let mut uart = WRITER.lock();
        while let Some(byte) = uart.read_byte() {
            // TEAM_244: Detect Ctrl+C (0x03) in interrupt handler for immediate signaling
            if byte == 0x03 {
                CTRL_C_PENDING.store(true, Ordering::Release);
                // Still buffer it so poll_input_devices can also see it
            }
            RX_BUFFER.lock().push(byte);
        }
        uart.clear_interrupts();
    }
}

/// TEAM_244: Check if Ctrl+C was received and clear the flag.
/// Called by kernel interrupt handler to signal foreground process.
pub fn check_and_clear_ctrl_c() -> bool {
    CTRL_C_PENDING.swap(false, Ordering::AcqRel)
}

/// TEAM_244: Poll UART for Ctrl+C without waiting for interrupt.
/// Called from timer handler as fallback since QEMU terminal mode
/// doesn't trigger UART RX interrupts reliably.
pub fn poll_for_ctrl_c() -> bool {
    // Direct poll UART (no interrupt needed)
    if let Some(byte) = WRITER.lock().read_byte() {
        if byte == 0x03 {
            return true;
        }
        // Buffer non-Ctrl+C bytes for later reading
        RX_BUFFER.lock().push(byte);
    }
    false
}

pub fn read_byte() -> Option<u8> {
    // First try the interrupt-driven buffer
    if let Some(byte) = RX_BUFFER.lock().pop() {
        return Some(byte);
    }

    #[cfg(target_arch = "aarch64")]
    {
        // TEAM_139: Fallback to direct UART polling
        // QEMU may not trigger RX interrupts when stdin is piped
        // This directly checks the UART RX FIFO for data
        WRITER.lock().read_byte()
    }
    #[cfg(target_arch = "x86_64")]
    {
        WRITER.lock().read_byte()
    }
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
/// Safe for use in IRQ handlers and low-level driver flushes as it can deadlock
/// with the secondary output lock.
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
