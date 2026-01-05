//! TEAM_081: GPU Console Integration
//!
//! Provides a global terminal instance that can be used by the console
//! callback to mirror output to the GPU terminal.

// TEAM_086: Display no longer used directly - GpuState is borrowed from caller's lock
use crate::gpu::GPU;
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
/// TEAM_083: Simplified to avoid deadlocks - single lock, no nested GPU access
pub fn write_str(s: &str) {
    // Lock GPU first, then terminal - consistent order prevents deadlock
    let mut gpu_guard = GPU.lock();
    let mut term_guard = GPU_TERMINAL.lock();
    
    if let (Some(gpu_state), Some(term)) = (gpu_guard.as_mut(), term_guard.as_mut()) {
        // Copy dimensions before borrowing framebuffer
        let width = gpu_state.width;
        let height = gpu_state.height;
        let fb = gpu_state.framebuffer();
        
        // Simple direct pixel writing for each character
        for c in s.chars() {
            // For now, just advance cursor and track position
            // Real rendering will come later
            match c {
                '\n' => {
                    term.cursor_col = 0;
                    term.cursor_row += 1;
                    if term.cursor_row >= term.rows {
                        term.cursor_row = term.rows - 1;
                        // TODO: scroll
                    }
                }
                '\r' => {
                    term.cursor_col = 0;
                }
                c if c >= ' ' => {
                    // Simple character rendering - draw a block for now
                    let x = term.cursor_col * 10; // FONT_WIDTH
                    let y = term.cursor_row * 22; // FONT_HEIGHT + spacing
                    
                    // Draw character block (simplified - just marks position)
                    if x < width && y < height {
                        let offset = ((y * width + x) * 4) as usize;
                        if offset + 4 <= fb.len() {
                            // Draw white pixel to show something
                            fb[offset] = 0xCC;     // B
                            fb[offset + 1] = 0xCC; // G
                            fb[offset + 2] = 0xCC; // R
                            fb[offset + 3] = 0xFF; // A
                        }
                    }
                    
                    term.cursor_col += 1;
                    if term.cursor_col >= term.cols {
                        term.cursor_col = 0;
                        term.cursor_row += 1;
                    }
                }
                _ => {}
            }
        }
        
        // TEAM_087: Removed flush per-call - was causing performance issues
        // Flush is batched or done by caller if needed
        // if let Err(_) = gpu_state.gpu.flush() {
        //     levitate_hal::serial_println!("[GPU] flush error");
        // }
    }
}

/// TEAM_083: Dummy callback for debugging
pub fn dummy_write_str(_s: &str) {
    levitate_hal::serial_println!("DUMMY_CALLBACK");
}

/// Clear the GPU terminal and reset cursor.
/// TEAM_086: Refactored to use single lock scope with new Display API
#[allow(dead_code)]
pub fn clear() {
    // TEAM_086: Lock GPU first, then terminal - consistent order prevents deadlock
    let mut gpu_guard = GPU.lock();
    let mut term_guard = GPU_TERMINAL.lock();
    
    if let (Some(gpu_state), Some(term)) = (gpu_guard.as_mut(), term_guard.as_mut()) {
        term.clear(gpu_state);
        gpu_state.flush();
    }
}

/// Get terminal size (columns, rows) if initialized.
#[allow(dead_code)]
pub fn size() -> Option<(u32, u32)> {
    GPU_TERMINAL.lock().as_ref().map(|t| t.size())
}

/// TEAM_083: Get mutable access to the terminal for direct drawing
pub fn get_terminal() -> levitate_hal::IrqSafeLockGuard<'static, Option<Terminal>> {
    GPU_TERMINAL.lock()
}

/// Check cursor blink timer.
/// TEAM_086: Refactored to use single lock scope with new Display API
pub fn check_blink() {
    let mut gpu_guard = GPU.lock();
    let mut term_guard = GPU_TERMINAL.lock();
    
    if let (Some(gpu_state), Some(term)) = (gpu_guard.as_mut(), term_guard.as_mut()) {
        term.check_blink(gpu_state);
        gpu_state.flush();
    }
}
