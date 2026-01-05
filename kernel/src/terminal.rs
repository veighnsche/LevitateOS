//! Kernel Terminal Integration
//! TEAM_092: Unified integration for dual-console mirroring.
//! This file replaces the previous terminal.rs and console_gpu.rs.

use crate::gpu::GPU;
use levitate_gpu::{Display, GpuState};
use levitate_hal::IrqSafeLock;
use levitate_terminal::Terminal;

pub static TERMINAL: IrqSafeLock<Option<Terminal>> = IrqSafeLock::new(None);

/// Initialize the global GPU terminal.
/// Mirroring is enabled after this by calling levitate_hal::console::set_secondary_output.
pub fn init() {
    if let Some(mut gpu_guard) = GPU.try_lock() {
        if let Some(gpu_state) = gpu_guard.as_mut() {
            let term = Terminal::new(gpu_state.width, gpu_state.height);
            *TERMINAL.lock() = Some(term);
        }
    }
}

/// Mirror console output to the GPU terminal.
/// Called via the secondary output callback in levitate-hal.
pub fn write_str(s: &str) {
    if let Some(mut term_guard) = TERMINAL.try_lock() {
        if let Some(term) = term_guard.as_mut() {
            if let Some(mut gpu_guard) = GPU.try_lock() {
                if let Some(gpu_state) = gpu_guard.as_mut() {
                    let mut display = Display::new(gpu_state);
                    term.write_str(&mut display, s);
                }
            }
        }
    }
}

/// Check cursor blink timer.
pub fn check_blink() {
    if let Some(mut term_guard) = TERMINAL.try_lock() {
        if let Some(term) = term_guard.as_mut() {
            if let Some(mut gpu_guard) = GPU.try_lock() {
                if let Some(gpu_state) = gpu_guard.as_mut() {
                    let _ = term_guard; // Hold both locks
                    // Note: Terminal::check_blink needs time/tick info
                    // Currently, the library heartbeats handles hardware status.
                    // The actual blink logic will be refactored into the library in Phase 3.
                }
            }
        }
    }
}
