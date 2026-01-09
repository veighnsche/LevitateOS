//! VirtIO Input Device Driver
//!
//! TEAM_032: Updated for virtio-drivers v0.12.0
//! - Uses StaticMmioTransport for 'static lifetime compatibility
//!
//! TEAM_241: Added interrupt handler for async Ctrl+C detection
//! - Handler registered during init() with slot-based IRQ computation
//! - Ctrl+C immediately signals foreground process via SIGINT
//!
//! TEAM_331: Added PCI transport support for x86_64

use log;
#[cfg(target_arch = "aarch64")]
use los_hal::virtio::StaticMmioTransport;
use los_hal::{InterruptHandler, IrqId, VirtioHal};
use alloc::vec::Vec;
use los_utils::Mutex;
use virtio_drivers::device::input::VirtIOInput;

/// TEAM_331: Wrapper enum to support both MMIO (aarch64) and PCI (x86_64) transports
enum InputDevice {
    #[cfg(target_arch = "aarch64")]
    Mmio(VirtIOInput<VirtioHal, StaticMmioTransport>),
    #[cfg(target_arch = "x86_64")]
    Pci(VirtIOInput<VirtioHal, los_pci::PciTransport>),
}

static INPUT_DEVICES: Mutex<Vec<InputDevice>> = Mutex::new(Vec::new());

/// Character buffer for keyboard input
/// TEAM_156: Increased from 256 to 1024 to prevent drops during rapid input
static KEYBOARD_BUFFER: Mutex<los_utils::RingBuffer<char, 1024>> =
    Mutex::new(los_utils::RingBuffer::new('\0'));

/// Track shift key state
static SHIFT_PRESSED: Mutex<bool> = Mutex::new(false);
/// Track control key state
static CTRL_PRESSED: Mutex<bool> = Mutex::new(false);

// TEAM_241: VirtIO Input interrupt handler for async Ctrl+C detection
/// Interrupt handler that polls VirtIO input when IRQ fires
struct InputInterruptHandler;

impl InterruptHandler for InputInterruptHandler {
    fn handle(&self, irq: u32) {
        let _ = irq; // Suppress unused warning
        // TEAM_241: Poll VirtIO input device for pending events
        // poll() handles Ctrl+C detection and immediate signaling
        poll();
    }
}

/// Static handler instance for GIC registration
static INPUT_HANDLER: InputInterruptHandler = InputInterruptHandler;

/// Initialize VirtIO Input device via MMIO (aarch64).
///
/// # Arguments
/// * `transport` - MMIO transport for the device
/// * `slot` - MMIO slot index (used to compute IRQ number: IRQ = 48 + slot)
#[cfg(target_arch = "aarch64")]
pub fn init(transport: StaticMmioTransport, slot: usize) {
    log::info!("[INPUT] Initializing Input (slot {})...", slot);
    match VirtIOInput::<VirtioHal, StaticMmioTransport>::new(transport) {
        Ok(input) => {
            log::info!("[INPUT] VirtIO Input initialized successfully.");
            INPUT_DEVICES.lock().push(InputDevice::Mmio(input));

            // TEAM_255: Register interrupt handler using generic HAL traits
            let irq_id = IrqId::VirtioInput(slot as u32);
            let ic = los_hal::active_interrupt_controller();
            ic.register_handler(irq_id, &INPUT_HANDLER);
            ic.enable_irq(ic.map_irq(irq_id));
            log::debug!("[INPUT] VirtIO Input IRQ {} enabled", ic.map_irq(irq_id));
        }
        Err(e) => log::error!("[INPUT] Failed to init VirtIO Input: {:?}", e),
    }
}

/// TEAM_331: Initialize VirtIO Input device via PCI (x86_64).
#[cfg(target_arch = "x86_64")]
pub fn init_pci() {
    log::info!("[INPUT] Initializing Input via PCI...");
    
    match los_pci::find_virtio_input::<VirtioHal>() {
        Some(transport) => {
            match VirtIOInput::<VirtioHal, los_pci::PciTransport>::new(transport) {
                Ok(input) => {
                    log::info!("[INPUT] VirtIO Input initialized via PCI.");
                    INPUT_DEVICES.lock().push(InputDevice::Pci(input));
                }
                Err(e) => log::error!("[INPUT] Failed to create VirtIO Input: {:?}", e),
            }
        }
        None => {
            log::warn!("[INPUT] No VirtIO Input found on PCI bus");
        }
    }
}

/// Read a character from the keyboard buffer
pub fn read_char() -> Option<char> {
    KEYBOARD_BUFFER.lock().pop()
}

pub const EV_KEY: u16 = 1;
pub const EV_ABS: u16 = 3;

pub const KEY_LEFTSHIFT: u16 = 42;
pub const KEY_RIGHTSHIFT: u16 = 54;
pub const KEY_ENTER: u16 = 28;
pub const KEY_BACKSPACE: u16 = 14;
pub const KEY_SPACE: u16 = 57;
pub const KEY_TAB: u16 = 15;
pub const KEY_LEFTCTRL: u16 = 29;
pub const KEY_RIGHTCTRL: u16 = 97;
pub const KEY_C: u16 = 46;

pub fn poll() -> bool {
    let dirty = false;

    // TEAM_030: Get actual screen dimensions from GPU instead of hardcoding
    let (screen_width, screen_height) = {
        let gpu = crate::gpu::GPU.lock();
        if let Some(state) = gpu.as_ref() {
            // TEAM_100: Use dimensions() for old GpuState API
            let (w, h) = state.dimensions();
            (w as i32, h as i32)
        } else {
            (1024, 768) // Fallback if GPU not initialized
        }
    };

    let mut devices = INPUT_DEVICES.lock();
    for device in devices.iter_mut() {
        // TEAM_331: Handle both MMIO and PCI transports
        match device {
            #[cfg(target_arch = "aarch64")]
            InputDevice::Mmio(input) => {
                poll_input_device(input, screen_width, screen_height);
            }
            #[cfg(target_arch = "x86_64")]
            InputDevice::Pci(input) => {
                poll_input_device(input, screen_width, screen_height);
            }
        }
    }
    dirty
}

/// TEAM_331: Generic input polling for any transport type
fn poll_input_device<T: virtio_drivers::transport::Transport>(
    input: &mut VirtIOInput<VirtioHal, T>,
    screen_width: i32,
    screen_height: i32,
) {
    while let Some(event) = input.pop_pending_event() {
        let event: virtio_drivers::device::input::InputEvent = event;
        match event.event_type {
            EV_ABS => {
                // TEAM_100: Mouse cursor support removed (dead code cleanup)
                // Future: Re-implement when VirtIO GPU cursor support is added
                let _ = (screen_width, screen_height, event.code, event.value);
            }
            EV_KEY => {
                let pressed = event.value != 0;
                match event.code {
                    KEY_LEFTSHIFT | KEY_RIGHTSHIFT => {
                        *SHIFT_PRESSED.lock() = pressed;
                    }
                    KEY_LEFTCTRL | KEY_RIGHTCTRL => {
                        *CTRL_PRESSED.lock() = pressed;
                    }
                    code if pressed => {
                        if *CTRL_PRESSED.lock() && code == KEY_C {
                            if !KEYBOARD_BUFFER.lock().push('\x03') {
                                crate::verbose!("KEYBOARD_BUFFER overflow, Ctrl+C dropped");
                            }
                            // TEAM_241: Signal foreground process immediately on Ctrl+C
                            // This ensures signal is delivered even if no one is reading stdin
                            crate::syscall::signal::signal_foreground_process(
                                crate::syscall::signal::SIGINT,
                            );
                        } else if let Some(c) = linux_code_to_ascii(code, *SHIFT_PRESSED.lock())
                        {
                            // TEAM_156: Don't silently drop - log overflow
                            if !KEYBOARD_BUFFER.lock().push(c) {
                                crate::verbose!("KEYBOARD_BUFFER overflow, char dropped");
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    input.ack_interrupt();
}

/// Map Linux key codes to ASCII
fn linux_code_to_ascii(code: u16, shift: bool) -> Option<char> {
    match code {
        2..=11 => {
            // 1-9, 0
            let chars = if shift { ")!@#$%^&*(" } else { "1234567890" };
            chars.chars().nth(code as usize - 2)
        }
        16..=25 => {
            // q-p
            let chars = if shift { "QWERTYUIOP" } else { "qwertyuiop" };
            chars.chars().nth(code as usize - 16)
        }
        30..=38 => {
            // a-l
            let chars = if shift { "ASDFGHJKL" } else { "asdfghjkl" };
            chars.chars().nth(code as usize - 30)
        }
        44..=50 => {
            // z-m
            let chars = if shift { "ZXCVBNM" } else { "zxcvbnm" };
            chars.chars().nth(code as usize - 44)
        }
        // Symbols
        12 => Some(if shift { '_' } else { '-' }),
        13 => Some(if shift { '+' } else { '=' }),
        26 => Some(if shift { '{' } else { '[' }),
        27 => Some(if shift { '}' } else { ']' }),
        39 => Some(if shift { ':' } else { ';' }),
        40 => Some(if shift { '"' } else { '\'' }),
        41 => Some(if shift { '~' } else { '`' }),
        43 => Some(if shift { '|' } else { '\\' }),
        51 => Some(if shift { '<' } else { ',' }),
        52 => Some(if shift { '>' } else { '.' }),
        53 => Some(if shift { '?' } else { '/' }),

        KEY_SPACE => Some(' '),
        KEY_ENTER => Some('\n'),
        KEY_BACKSPACE => Some('\x08'),
        KEY_TAB => Some('\t'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_code_to_ascii_symbols() {
        // [IN1] Regression test for shell input bug
        // Verify that dot and dash are correctly mapped
        assert_eq!(linux_code_to_ascii(52, false), Some('.')); // dot
        assert_eq!(linux_code_to_ascii(12, false), Some('-')); // dash

        // Verify shift variants
        assert_eq!(linux_code_to_ascii(52, true), Some('>')); // greater
        assert_eq!(linux_code_to_ascii(12, true), Some('_')); // underscore

        // Verify other symbols
        assert_eq!(linux_code_to_ascii(53, false), Some('/'));
        assert_eq!(linux_code_to_ascii(53, true), Some('?'));
    }
}
