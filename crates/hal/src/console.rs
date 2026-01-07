// TEAM_262: Generic console abstraction.
// Delegates architecture-specific hardware logic to crate::arch::console.

use crate::IrqSafeLock;
#[cfg(target_arch = "aarch64")]
use crate::aarch64::serial::Pl011Uart;
#[cfg(target_arch = "x86_64")]
use crate::x86_64::serial::SerialPort;
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use los_utils::RingBuffer;

// TEAM_039: Re-export hex utilities from levitate-utils
pub use los_utils::hex::{format_hex, nibble_to_hex};

// TEAM_259: Secondary output callback for dual console (GPU terminal or VGA)
type SecondaryOutputFn = fn(&str);
static SECONDARY_OUTPUT: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());
static SECONDARY_OUTPUT_ENABLED: AtomicBool = AtomicBool::new(false);

static RX_BUFFER: IrqSafeLock<RingBuffer<u8, 1024>> = IrqSafeLock::new(RingBuffer::new(0));

/// TEAM_244: Flag set when Ctrl+C (0x03) is received via serial interrupt
static CTRL_C_PENDING: AtomicBool = AtomicBool::new(false);

pub fn init() {
    #[cfg(target_arch = "aarch64")]
    {
        let mut uart = crate::arch::console::WRITER.lock();
        uart.init();
        uart.enable_rx_interrupt();
    }
    #[cfg(target_arch = "x86_64")]
    {
        // x86_64 SerialPort::init is handled in x86_64::init()
    }
}

pub fn handle_interrupt() {
    #[cfg(target_arch = "aarch64")]
    {
        let mut uart = crate::arch::console::WRITER.lock();
        while let Some(byte) = uart.read_byte() {
            if byte == 0x03 {
                CTRL_C_PENDING.store(true, Ordering::Release);
            }
            RX_BUFFER.lock().push(byte);
        }
        uart.clear_interrupts();
    }
    #[cfg(target_arch = "x86_64")]
    {
        // x86_64 interrupt handling for serial is currently polled or stubbed
    }
}

pub fn check_and_clear_ctrl_c() -> bool {
    CTRL_C_PENDING.swap(false, Ordering::AcqRel)
}

pub fn poll_for_ctrl_c() -> bool {
    if let Some(byte) = crate::arch::console::WRITER.lock().read_byte() {
        if byte == 0x03 {
            return true;
        }
        RX_BUFFER.lock().push(byte);
    }
    false
}

pub fn read_byte() -> Option<u8> {
    if let Some(byte) = RX_BUFFER.lock().pop() {
        return Some(byte);
    }
    crate::arch::console::WRITER.lock().read_byte()
}

pub fn set_secondary_output(callback: SecondaryOutputFn) {
    SECONDARY_OUTPUT.store(callback as *mut (), Ordering::SeqCst);
    SECONDARY_OUTPUT_ENABLED.store(true, Ordering::SeqCst);
}

#[allow(dead_code)]
pub fn disable_secondary_output() {
    SECONDARY_OUTPUT_ENABLED.store(false, Ordering::SeqCst);
}

pub fn _print(args: fmt::Arguments) {
    let _ = crate::arch::console::WRITER.lock().write_fmt(args);

    if SECONDARY_OUTPUT_ENABLED.load(Ordering::SeqCst) {
        let ptr = SECONDARY_OUTPUT.load(Ordering::SeqCst);
        if !ptr.is_null() {
            let callback: SecondaryOutputFn = unsafe { core::mem::transmute(ptr) };
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

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// print_hex moved to arch::console::print_hex

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        let _ = core::fmt::Write::write_fmt(&mut *$crate::arch::console::WRITER.lock(), format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => { $crate::serial_print!("\n") };
    ($($arg:tt)*) => { $crate::serial_print!("{}\n", format_args!($($arg)*)) };
}
