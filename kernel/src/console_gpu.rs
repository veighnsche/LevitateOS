//! TEAM_081: GPU Console Integration
//!
//! Provides a global terminal instance that can be used by the console
//! callback to mirror output to the GPU terminal.

use crate::gpu::{Display, GPU};
use crate::terminal::Terminal;
use levitate_hal::IrqSafeLock;

// TEAM_081: Global terminal state for dual console output
// TEAM_083: Changed to IrqSafeLock to prevent deadlocks when printing from interrupts
static GPU_TERMINAL: IrqSafeLock<Option<Terminal>> = IrqSafeLock::new(None);

/// Initialize the global GPU terminal with the given screen dimensions.
/// Called once during boot after GPU initialization.
pub fn init(width: u32, height: u32) {
    let term = Terminal::new(width, height);
    *GPU_TERMINAL.lock() = Some(term);
}

/// Write a string to the GPU terminal.
/// This is the callback function registered with the console.
pub fn write_str(s: &str) {
    let mut guard = GPU_TERMINAL.lock();
    if let Some(term) = guard.as_mut() {
        let mut display = Display;
        for c in s.chars() {
            term.write_char(&mut display, c);
        }
        // TEAM_083: Explicit flush after writing the whole string
        let mut gpu = GPU.lock();
        if let Some(state) = gpu.as_mut() {
            state.flush();
        }
    }
}

/// TEAM_083: Dummy callback for debugging
pub fn dummy_write_str(_s: &str) {
    levitate_hal::serial_println!("DUMMY_CALLBACK");
}

/// Clear the GPU terminal and reset cursor.
#[allow(dead_code)]
pub fn clear() {
    let mut guard = GPU_TERMINAL.lock();
    if let Some(term) = guard.as_mut() {
        let mut display = Display;
        term.clear(&mut display);
        // TEAM_083: Explicit flush after clear
        let mut gpu = GPU.lock();
        if let Some(state) = gpu.as_mut() {
            state.flush();
        }
    }
}

/// Get terminal size (columns, rows) if initialized.
#[allow(dead_code)]
pub fn size() -> Option<(u32, u32)> {
    GPU_TERMINAL.lock().as_ref().map(|t| t.size())
}

/// Check cursor blink timer.
pub fn check_blink() {
    let mut guard = GPU_TERMINAL.lock();
    if let Some(term) = guard.as_mut() {
        let mut display = Display;
        term.check_blink(&mut display);
    }
}
