//! TEAM_081: GPU Console Integration
//!
//! Provides a global terminal instance that can be used by the console
//! callback to mirror output to the GPU terminal.

use crate::gpu::Display;
use crate::terminal::Terminal;
use levitate_utils::Spinlock;

// TEAM_081: Global terminal state for dual console output
static GPU_TERMINAL: Spinlock<Option<Terminal>> = Spinlock::new(None);

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
    }
}

/// Clear the GPU terminal and reset cursor.
#[allow(dead_code)]
pub fn clear() {
    let mut guard = GPU_TERMINAL.lock();
    if let Some(term) = guard.as_mut() {
        let mut display = Display;
        term.clear(&mut display);
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
