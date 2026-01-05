//! VirtIO Input Device Driver
//!
//! TEAM_032: Updated for virtio-drivers v0.12.0
//! - Uses StaticMmioTransport for 'static lifetime compatibility

use crate::virtio::{StaticMmioTransport, VirtioHal};
use alloc::vec::Vec;
use levitate_utils::Spinlock;
use virtio_drivers::device::input::VirtIOInput;

// TEAM_032: Use StaticMmioTransport (MmioTransport<'static>) for static storage
static INPUT_DEVICES: Spinlock<Vec<VirtIOInput<VirtioHal, StaticMmioTransport>>> =
    Spinlock::new(Vec::new());

/// Character buffer for keyboard input
static KEYBOARD_BUFFER: Spinlock<levitate_utils::RingBuffer<char, 256>> =
    Spinlock::new(levitate_utils::RingBuffer::new('\0'));

/// Track shift key state
static SHIFT_PRESSED: Spinlock<bool> = Spinlock::new(false);

pub fn init(transport: StaticMmioTransport) {
    crate::verbose!("Initializing Input...");
    match VirtIOInput::<VirtioHal, StaticMmioTransport>::new(transport) {
        Ok(input) => {
            crate::verbose!("VirtIO Input initialized successfully.");
            INPUT_DEVICES.lock().push(input);
        }
        Err(e) => crate::println!("Failed to init VirtIO Input: {:?}", e),
    }
}

/// Read a character from the keyboard buffer
pub fn read_char() -> Option<char> {
    KEYBOARD_BUFFER.lock().pop()
}

pub const EV_KEY: u16 = 1;
pub const EV_ABS: u16 = 3;
pub const ABS_X: u16 = 0;
pub const ABS_Y: u16 = 1;

pub const KEY_LEFTSHIFT: u16 = 42;
pub const KEY_RIGHTSHIFT: u16 = 54;
pub const KEY_ENTER: u16 = 28;
pub const KEY_BACKSPACE: u16 = 14;
pub const KEY_SPACE: u16 = 57;
pub const KEY_TAB: u16 = 15;

pub fn poll() -> bool {
    let mut dirty = false;

    // TEAM_030: Get actual screen dimensions from GPU instead of hardcoding
    let (screen_width, screen_height) = {
        let gpu = crate::gpu::GPU.lock();
        if let Some(state) = gpu.as_ref() {
            let (w, h) = state.dimensions();
            (w as i32, h as i32)
        } else {
            (1024, 768) // Fallback if GPU not initialized
        }
    };

    let mut devices = INPUT_DEVICES.lock();
    for input in devices.iter_mut() {
        let input: &mut VirtIOInput<VirtioHal, StaticMmioTransport> = input;
        while let Some(event) = input.pop_pending_event() {
            let event: virtio_drivers::device::input::InputEvent = event;
            match event.event_type {
                EV_ABS => match event.code {
                    ABS_X => {
                        let x = (event.value as i32 * screen_width) / 32768;
                        crate::cursor::set_x(x);
                        dirty = true;
                    }
                    ABS_Y => {
                        let y = (event.value as i32 * screen_height) / 32768;
                        crate::cursor::set_y(y);
                        dirty = true;
                    }
                    _ => {}
                },
                EV_KEY => {
                    let pressed = event.value != 0;
                    match event.code {
                        KEY_LEFTSHIFT | KEY_RIGHTSHIFT => {
                            *SHIFT_PRESSED.lock() = pressed;
                        }
                        code if pressed => {
                            if let Some(c) = linux_code_to_ascii(code, *SHIFT_PRESSED.lock()) {
                                let _ = KEYBOARD_BUFFER.lock().push(c);
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
    dirty
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
        KEY_SPACE => Some(' '),
        KEY_ENTER => Some('\n'),
        KEY_BACKSPACE => Some('\x08'),
        KEY_TAB => Some('\t'),
        _ => None,
    }
}
