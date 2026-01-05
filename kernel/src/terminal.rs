//! Kernel Terminal Integration
//! TEAM_092: Unified integration for dual-console mirroring.
//! This file replaces the previous terminal.rs and console_gpu.rs.

use levitate_hal::IrqSafeLock;
use levitate_terminal::Terminal;

pub static TERMINAL: IrqSafeLock<Option<Terminal>> = IrqSafeLock::new(None);

/// Initialize the global GPU terminal.
/// Mirroring is enabled after this by calling levitate_hal::console::set_secondary_output.
pub fn init() {
    if let Some(mut gpu_guard) = crate::gpu::GPU.try_lock() {
        if let Some(gpu_state) = gpu_guard.as_mut() {
            // TEAM_100: Use dimensions() for old GpuState API
            let (width, height) = gpu_state.dimensions();
            let term = Terminal::new(width, height);
            // TEAM_100: Log for golden file compatibility
            crate::println!(
                "[TERM] Terminal::new({}x{}) -> {}x{} chars (font {}x{}, spacing {})",
                width,
                height,
                term.cols,
                term.rows,
                10,
                20,
                2
            );
            *TERMINAL.lock() = Some(term);
        }
    }
}

/// Mirror console output to the GPU terminal.
/// Called via the secondary output callback in levitate-hal.
/// TEAM_115: Changed from try_lock to lock to ensure output is never lost.
/// TEAM_129: Added explicit flush after write to make output immediately visible.
pub fn write_str(s: &str) {
    let mut term_guard = TERMINAL.lock();
    if let Some(term) = term_guard.as_mut() {
        let mut gpu_guard = crate::gpu::GPU.lock();
        if let Some(gpu_state) = gpu_guard.as_mut() {
            // TEAM_100: Use Display wrapper for DrawTarget
            let mut display = crate::gpu::Display::new(gpu_state);
            term.write_str(&mut display, s);
            // TEAM_129: Flush GPU after every write to ensure output is visible
            // The timer-based flush uses try_lock which fails when we hold the lock
            let _ = gpu_state.flush();
        }
    }
}
