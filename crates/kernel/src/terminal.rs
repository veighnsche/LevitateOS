//! Kernel Terminal Integration
//! TEAM_092: Unified integration for dual-console mirroring.
//! This file replaces the previous terminal.rs and console_gpu.rs.
//!
//! TEAM_158: Behavior IDs [GPU1]-[GPU7] for traceability.

use los_hal::IrqSafeLock;
use los_term::Terminal;

pub static TERMINAL: IrqSafeLock<Option<Terminal>> = IrqSafeLock::new(None);

/// [GPU1] Initialize the global GPU terminal with correct dimensions.
/// [GPU2] Terminal calculates cols/rows from pixel dimensions.
/// Mirroring is enabled after this by calling los_hal::console::set_secondary_output.
pub fn init() {
    if let Some(mut gpu_guard) = crate::gpu::GPU.try_lock() {
        if let Some(gpu_state) = gpu_guard.as_mut() {
            // [GPU1] Use dimensions() for correct GPU framebuffer size
            let (width, height) = gpu_state.dimensions();
            // [GPU2] Terminal calculates cols/rows from pixel dimensions
            let term = Terminal::new(width, height);
            // TEAM_100: Log for golden file compatibility
            log::debug!(
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

/// [GPU3] Mirror console output to the GPU terminal (renders text to framebuffer).
/// [GPU4] write_str advances cursor position.
/// [GPU5] Newline triggers GPU flush.
/// [GPU7] Terminal output visible on VNC.
/// Called via the secondary output callback in levitate-hal.
/// TEAM_115: Changed from try_lock to lock to ensure output is never lost.
/// TEAM_143: Only flush on newline for performance. Timer provides 20Hz flush for responsiveness.
pub fn write_str(s: &str) {
    let mut term_guard = TERMINAL.lock();
    if let Some(term) = term_guard.as_mut() {
        let mut gpu_guard = crate::gpu::GPU.lock();
        if let Some(gpu_state) = gpu_guard.as_mut() {
            // [GPU3][GPU4] Render text and advance cursor
            let mut display = gpu_state.as_display();
            term.write_str(&mut display, s); // [GPU3] renders, [GPU4] advances cursor
            // [GPU5] Only flush on newline for performance
            // [GPU6] Timer interrupt flushes at 20Hz for responsiveness between newlines
            if s.contains('\n') {
                let _ = gpu_state.flush(); // [GPU5] newline triggers flush
            }
        }
    }
}
