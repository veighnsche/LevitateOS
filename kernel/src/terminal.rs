//! GPU Terminal Emulator
//!
//! TEAM_058: Terminal emulation for Phase 6 GPU Refinement
//!
//! ## Behaviors (with verbose UART logging for verification)
//! - [TERM1] Character rendering at cursor position
//! - [TERM2] Cursor advances after each character
//! - [TERM3] Newline moves to start of next line
//! - [TERM4] Screen scrolls when cursor exceeds rows
//! - [TERM5] Carriage return moves to start of current line
//! - [TERM6] Tab advances to next 8-column boundary
//! - [TERM7] Clear fills screen with background color
//! - [TERM8] Backspace moves cursor left (no erase)
//! - [TERM9] Resolution adapts to screen dimensions

use crate::gpu::{Display, GPU};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};

/// Font dimensions (FONT_10X20) - SC3.4, SC3.5
const FONT_WIDTH: u32 = 10;
const FONT_HEIGHT: u32 = 20;
const LINE_SPACING: u32 = 2;
const CHARACTER_SPACING: u32 = 0;

/// Default terminal colors - SC4.5, SC4.6
pub const DEFAULT_FG: Rgb888 = Rgb888::new(204, 204, 204); // Light gray
pub const DEFAULT_BG: Rgb888 = Rgb888::new(0, 0, 0); // Black

/// Terminal configuration (compile-time or runtime)
pub struct TerminalConfig {
    #[allow(dead_code)]
    pub font_width: u32,
    #[allow(dead_code)]
    pub font_height: u32,
    #[allow(dead_code)]
    pub line_spacing: u32,
    pub fg_color: Rgb888,
    pub bg_color: Rgb888,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            font_width: FONT_WIDTH,
            font_height: FONT_HEIGHT,
            line_spacing: LINE_SPACING,
            fg_color: DEFAULT_FG,
            bg_color: DEFAULT_BG,
        }
    }
}

/// Terminal emulator state - SC3.2
pub struct Terminal {
    cursor_col: u32,
    cursor_row: u32,
    cols: u32,
    rows: u32,
    config: TerminalConfig,
    screen_width: u32,  // pixels
    screen_height: u32, // pixels
    cursor_visible: bool,
    last_blink: u64,
    saved_pixels: [[Rgb888; 10]; 20],
    has_saved: bool,
    ansi_state: AnsiState, // TEAM_063: Track escape sequences
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnsiState {
    None,
    Esc,
    CSI,
}

impl Terminal {
    /// [TERM9] Create terminal with auto-calculated dimensions - SC3.3
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        // SC3.4: cols = width / FONT_WIDTH
        let cols = screen_width / FONT_WIDTH;
        // SC3.5: rows = height / (FONT_HEIGHT + LINE_SPACING)
        let rows = screen_height / (FONT_HEIGHT + LINE_SPACING);

        // SC2.5: Log resolution to UART for verification
        crate::println!(
            "[TERM] Terminal::new({}x{}) -> {}x{} chars (font {}x{}, spacing {})",
            screen_width,
            screen_height,
            cols,
            rows,
            FONT_WIDTH,
            FONT_HEIGHT,
            LINE_SPACING
        );

        Terminal {
            cursor_col: 0,
            cursor_row: 0,
            cols,
            rows,
            config: TerminalConfig::default(),
            screen_width,
            screen_height,
            cursor_visible: false,
            last_blink: 0,
            saved_pixels: [[Rgb888::BLACK; 10]; 20],
            has_saved: false,
            ansi_state: AnsiState::None,
        }
    }

    /// Get terminal dimensions (columns, rows)
    pub fn size(&self) -> (u32, u32) {
        (self.cols, self.rows)
    }

    /// Get cursor position (column, row)
    #[allow(dead_code)]
    pub fn cursor(&self) -> (u32, u32) {
        (self.cursor_col, self.cursor_row)
    }

    /// [TERM1] [TERM2] Write single character at cursor position
    pub fn write_char(&mut self, display: &mut Display, c: char) {
        self.hide_cursor(display);

        // TEAM_063: Basic ANSI VT100 state machine
        match self.ansi_state {
            AnsiState::None => {
                if c == '\x1b' {
                    self.ansi_state = AnsiState::Esc;
                    return;
                }
            }
            AnsiState::Esc => {
                if c == '[' {
                    self.ansi_state = AnsiState::CSI;
                } else {
                    self.ansi_state = AnsiState::None;
                }
                return;
            }
            AnsiState::CSI => {
                // Currently only support 'J' (Clear Screen) for VT100 compatibility
                if c == 'J' {
                    self.clear(display);
                }
                // Transition back after any command or if we hit a non-parameter char
                if (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') {
                    self.ansi_state = AnsiState::None;
                }
                return;
            }
        }

        match c {
            '\n' => {
                // [TERM3] - SC6.1, SC6.2
                self.newline(display);

                // TEAM_059: Force flush after newline
                let mut guard = GPU.lock();
                if let Some(state) = guard.as_mut() {
                    state.flush();
                }
            }
            '\r' => {
                // [TERM5] - SC7.1, SC7.2
                self.carriage_return(display);
            }
            '\t' => {
                // [TERM6] - SC9.1
                self.tab(display);
            }
            '\x08' => {
                // [TERM8] [SPEC-3] Interactive destructive backspace
                self.backspace(display);
            }
            c if c >= ' ' => {
                // [TERM2] Check if we need to wrap - SC8.2
                if self.cursor_col >= self.cols {
                    self.newline(display);
                }

                // [TERM1] Render character - SC4.3, SC5.1, SC5.2
                let cur_row = self.cursor_row;
                let cur_col = self.cursor_col;
                let x = (cur_col * FONT_WIDTH) as i32;
                let y = (cur_row * (FONT_HEIGHT + LINE_SPACING)) as i32;

                let style = MonoTextStyle::new(&FONT_10X20, self.config.fg_color);
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);

                // Text baseline is at bottom of character
                let _ = Text::new(s, Point::new(x, y + FONT_HEIGHT as i32), style).draw(display);

                // TEAM_059: Explicitly flush after writing a character
                let mut guard = GPU.lock();
                if let Some(state) = guard.as_mut() {
                    state.flush();
                }

                // [TERM2] Advance cursor - SC5.3
                self.cursor_col += 1;
            }
            _ => {} // Ignore other control characters
        }
        self.show_cursor(display);
    }

    /// Write string to terminal
    pub fn write_str(&mut self, display: &mut Display, s: &str) {
        for c in s.chars() {
            self.write_char(display, c);
        }
    }

    /// [TERM3] [TERM4] Move to next line, scroll if needed - SC6.1, SC6.2
    pub fn newline(&mut self, display: &mut Display) {
        self.hide_cursor(display);
        self.cursor_col = 0;
        self.cursor_row += 1;

        // [TERM4] Scroll if we've exceeded screen - SC11.2
        if self.cursor_row >= self.rows {
            crate::println!(
                "[TERM] scroll_up triggered: row={} >= rows={}",
                self.cursor_row,
                self.rows
            );
            self.scroll_up(display);
            self.cursor_row = self.rows - 1; // SC11.6
        }
    }

    // [TERM5] - SC7.1, SC7.2
    pub fn carriage_return(&mut self, _display: &mut Display) {
        self.hide_cursor(_display);
        crate::println!(
            "[TERM] carriage_return at col={}, row={} -> col=0",
            self.cursor_col,
            self.cursor_row
        );
        self.cursor_col = 0;

        // TEAM_059: Force flush after carriage return
        let mut guard = GPU.lock();
        if let Some(state) = guard.as_mut() {
            state.flush();
        }
        self.show_cursor(_display);
    }

    /// [TERM6] Tab to next 8-column boundary - SC9.1-SC9.5
    /// TEAM_065: Fixed to handle wrap-around when tab exceeds columns
    fn tab(&mut self, display: &mut Display) {
        self.hide_cursor(display);
        let next_tab = ((self.cursor_col / 8) + 1) * 8;

        // TEAM_065: Handle wrap-around if tab would exceed line width
        if next_tab >= self.cols {
            self.newline(display);
        } else {
            self.cursor_col = next_tab;
        }

        // TEAM_059: Force flush after tab
        let mut guard = GPU.lock();
        if let Some(state) = guard.as_mut() {
            state.flush();
        }
        self.show_cursor(display);
    }

    /// [SPEC-2] Destructive backspace (0x08)
    fn backspace(&mut self, display: &mut Display) {
        self.hide_cursor(display);

        let mut changed = false;
        if self.cursor_col > 0 {
            // [TERM8] Normal backspace
            self.cursor_col -= 1;
            changed = true;
        } else if self.cursor_row > 0 {
            // [SPEC-2] Wrap back to previous line
            self.cursor_row -= 1;
            self.cursor_col = self.cols - 1;
            changed = true;
        }

        if changed {
            // [SPEC-2] Erase character at new position
            let x = self.cursor_col * (FONT_WIDTH + CHARACTER_SPACING);
            let y = self.cursor_row * (FONT_HEIGHT + LINE_SPACING);

            let _ = Rectangle::new(
                Point::new(x as i32, y as i32),
                Size::new(FONT_WIDTH, FONT_HEIGHT + LINE_SPACING),
            )
            .into_styled(PrimitiveStyle::with_fill(self.config.bg_color))
            .draw(display);

            // TEAM_059: Force flush after erase
            let mut guard = GPU.lock();
            if let Some(state) = guard.as_mut() {
                state.flush();
            }
        }
        self.show_cursor(display);
    }

    /// [TERM7] Clear screen and reset cursor - SC10.1-SC10.5
    pub fn clear(&mut self, display: &mut Display) {
        self.hide_cursor(display);
        crate::println!(
            "[TERM] clear() - filling {}x{} with bg color",
            self.screen_width,
            self.screen_height
        );

        let _ = Rectangle::new(
            Point::zero(),
            Size::new(self.screen_width, self.screen_height),
        )
        .into_styled(PrimitiveStyle::with_fill(self.config.bg_color))
        .draw(display);

        self.cursor_col = 0;
        self.cursor_row = 0;

        crate::println!("[TERM] clear() complete, cursor at (0, 0)");
        self.show_cursor(display);
    }

    /// [TERM4] Scroll screen up by one line - SC11.3-SC11.8
    fn scroll_up(&mut self, _display: &mut Display) {
        self.hide_cursor(_display);
        let line_height = FONT_HEIGHT + LINE_SPACING;

        crate::println!(
            "[TERM] scroll_up: moving content up by {} pixels",
            line_height
        );

        // Access framebuffer directly for efficient scroll
        let mut guard = GPU.lock();
        if let Some(state) = guard.as_mut() {
            let fb = state.framebuffer();
            let bytes_per_pixel = 4; // RGBA
            let row_bytes = self.screen_width as usize * bytes_per_pixel;
            let scroll_bytes = line_height as usize * row_bytes;

            // SC11.4: Copy everything up by one line
            if scroll_bytes < fb.len() {
                fb.copy_within(scroll_bytes.., 0);

                // SC11.5: Clear bottom line with background color
                let clear_start = fb.len() - scroll_bytes;
                for i in (clear_start..fb.len()).step_by(4) {
                    fb[i] = self.config.bg_color.r();
                    fb[i + 1] = self.config.bg_color.g();
                    fb[i + 2] = self.config.bg_color.b();
                    fb[i + 3] = 255;
                }

                crate::println!(
                    "[TERM] scroll_up complete: copied {} bytes, cleared bottom line",
                    fb.len() - scroll_bytes
                );
            }

            // TEAM_058: Use public flush method
            state.flush();
        }
        self.show_cursor(_display);
    }

    /// Set foreground color
    #[allow(dead_code)]
    pub fn set_fg(&mut self, color: Rgb888) {
        self.config.fg_color = color;
    }

    /// Set background color
    #[allow(dead_code)]
    pub fn set_bg(&mut self, color: Rgb888) {
        self.config.bg_color = color;
    }

    /// [TEAM_060] Toggle cursor visibility based on timer
    pub fn check_blink(&mut self, display: &mut Display) {
        let now = crate::timer::uptime_seconds();
        if now != self.last_blink {
            self.last_blink = now;
            if self.cursor_visible {
                self.hide_cursor(display);
            } else {
                self.show_cursor(display);
            }
        }
    }

    fn show_cursor(&mut self, display: &mut Display) {
        if self.cursor_visible {
            return;
        }

        let x = self.cursor_col * (FONT_WIDTH + CHARACTER_SPACING);
        let y = self.cursor_row * (FONT_HEIGHT + LINE_SPACING);

        // Save pixels
        let mut guard = crate::gpu::GPU.lock();
        if let Some(gpu) = guard.as_mut() {
            let fb = gpu.framebuffer();
            let width = self.screen_width as usize;

            for dy in 0..20 {
                for dx in 0..10 {
                    let py = (y + dy) as usize;
                    let px = (x + dx) as usize;
                    if py < self.screen_height as usize && px < self.screen_width as usize {
                        let idx = (py * width + px) * 4;
                        self.saved_pixels[dy as usize][dx as usize] =
                            Rgb888::new(fb[idx], fb[idx + 1], fb[idx + 2]);
                    }
                }
            }
            self.has_saved = true;
        }
        drop(guard);

        // Draw block
        let _ = Rectangle::new(Point::new(x as i32, y as i32), Size::new(10, 20))
            .into_styled(PrimitiveStyle::with_fill(self.config.fg_color))
            .draw(display);

        self.cursor_visible = true;
    }

    fn hide_cursor(&mut self, _display: &mut Display) {
        if !self.cursor_visible || !self.has_saved {
            self.cursor_visible = false;
            return;
        }

        let x = self.cursor_col * (FONT_WIDTH + CHARACTER_SPACING);
        let y = self.cursor_row * (FONT_HEIGHT + LINE_SPACING);

        // Restore pixels
        let mut guard = crate::gpu::GPU.lock();
        if let Some(gpu) = guard.as_mut() {
            let fb = gpu.framebuffer();
            let width = self.screen_width as usize;

            for dy in 0..20 {
                for dx in 0..10 {
                    let py = (y + dy) as usize;
                    let px = (x + dx) as usize;
                    if py < self.screen_height as usize && px < self.screen_width as usize {
                        let idx = (py * width + px) * 4;
                        let color = self.saved_pixels[dy as usize][dx as usize];
                        fb[idx] = color.r();
                        fb[idx + 1] = color.g();
                        fb[idx + 2] = color.b();
                        fb[idx + 3] = 255;
                    }
                }
            }
            gpu.flush();
        }

        self.cursor_visible = false;
    }
}
