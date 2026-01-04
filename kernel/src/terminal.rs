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
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};

/// Font dimensions (FONT_10X20) - SC3.4, SC3.5
const FONT_WIDTH: u32 = 10;
const FONT_HEIGHT: u32 = 20;
const LINE_SPACING: u32 = 2;

/// Default terminal colors - SC4.5, SC4.6
pub const DEFAULT_FG: Rgb888 = Rgb888::new(204, 204, 204); // Light gray
pub const DEFAULT_BG: Rgb888 = Rgb888::new(0, 0, 0); // Black

/// Terminal emulator state - SC3.2
pub struct Terminal {
    cursor_col: u32,
    cursor_row: u32,
    cols: u32,
    rows: u32,
    fg_color: Rgb888,
    bg_color: Rgb888,
    screen_width: u32,
    screen_height: u32,
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
            fg_color: DEFAULT_FG,
            bg_color: DEFAULT_BG,
            screen_width,
            screen_height,
        }
    }

    /// Get terminal dimensions (columns, rows)
    pub fn size(&self) -> (u32, u32) {
        (self.cols, self.rows)
    }

    /// Get cursor position (column, row)
    pub fn cursor(&self) -> (u32, u32) {
        (self.cursor_col, self.cursor_row)
    }

    /// [TERM1] [TERM2] Write single character at cursor position
    pub fn write_char(&mut self, display: &mut Display, c: char) {
        match c {
            '\n' => {
                // [TERM3] - SC6.1, SC6.2
                crate::println!(
                    "[TERM] newline at col={}, row={} -> col=0, row={}",
                    self.cursor_col,
                    self.cursor_row,
                    self.cursor_row + 1
                );
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
                // [TERM8] - SC12.1, SC12.2
                self.backspace();
            }
            c if c >= ' ' => {
                // [TERM2] Check if we need to wrap - SC8.2
                if self.cursor_col >= self.cols {
                    crate::println!(
                        "[TERM] line_wrap triggered at col={} (cols={})",
                        self.cursor_col,
                        self.cols
                    );
                    self.newline(display);
                }

                // [TERM1] Render character - SC4.3, SC5.1, SC5.2
                let cur_row = self.cursor_row;
                let cur_col = self.cursor_col;
                let x = (cur_col * FONT_WIDTH) as i32;
                let y = (cur_row * (FONT_HEIGHT + LINE_SPACING)) as i32;

                // Log every 20th character to avoid flooding UART
                if cur_col % 20 == 0 || c == 'A' {
                    crate::println!(
                        "[TERM] write_char('{}') at col={}, row={}, pixel=({}, {})",
                        c,
                        cur_col,
                        cur_row,
                        x,
                        y + FONT_HEIGHT as i32
                    );
                }

                let style = MonoTextStyle::new(&FONT_10X20, self.fg_color);
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
    }

    /// Write string to terminal
    pub fn write_str(&mut self, display: &mut Display, s: &str) {
        for c in s.chars() {
            self.write_char(display, c);
        }
    }

    /// [TERM3] [TERM4] Move to next line, scroll if needed - SC6.1, SC6.2
    /// TEAM_058 BREADCRUMB: SUSPECT - Newline visually doesn't work
    /// Characters after Enter overlap previous line. UART logs show correct state
    /// but display shows wrong position. Investigate Y coordinate calculation.
    pub fn newline(&mut self, display: &mut Display) {
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
    }

    /// [TERM6] Tab to next 8-column boundary - SC9.1-SC9.5
    fn tab(&mut self, display: &mut Display) {
        let old_col = self.cursor_col;
        let next_tab = ((self.cursor_col / 8) + 1) * 8;

        if next_tab >= self.cols {
            crate::println!(
                "[TERM] tab from col={} -> wrap (next_tab={} >= cols={})",
                old_col,
                next_tab,
                self.cols
            );
            self.newline(display);
        } else {
            crate::println!("[TERM] tab from col={} -> col={}", old_col, next_tab);
            self.cursor_col = next_tab;
        }

        // TEAM_059: Force flush after tab
        let mut guard = GPU.lock();
        if let Some(state) = guard.as_mut() {
            state.flush();
        }
    }

    /// [TERM8] Move cursor left (no erase) - SC12.1, SC12.2
    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            crate::println!("[TERM] backspace -> col={}", self.cursor_col);
        } else {
            crate::println!("[TERM] backspace at col=0, no action");
        }

        // TEAM_059: Force flush after backspace
        let mut guard = GPU.lock();
        if let Some(state) = guard.as_mut() {
            state.flush();
        }
    }

    /// [TERM7] Clear screen and reset cursor - SC10.1-SC10.5
    pub fn clear(&mut self, display: &mut Display) {
        crate::println!(
            "[TERM] clear() - filling {}x{} with bg color",
            self.screen_width,
            self.screen_height
        );

        let _ = Rectangle::new(
            Point::zero(),
            Size::new(self.screen_width, self.screen_height),
        )
        .into_styled(PrimitiveStyle::with_fill(self.bg_color))
        .draw(display);

        self.cursor_col = 0;
        self.cursor_row = 0;

        crate::println!("[TERM] clear() complete, cursor at (0, 0)");
    }

    /// [TERM4] Scroll screen up by one line - SC11.3-SC11.8
    fn scroll_up(&mut self, _display: &mut Display) {
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
                    fb[i] = self.bg_color.r();
                    fb[i + 1] = self.bg_color.g();
                    fb[i + 2] = self.bg_color.b();
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
    }

    /// Set foreground color
    pub fn set_fg(&mut self, color: Rgb888) {
        self.fg_color = color;
    }

    /// Set background color
    pub fn set_bg(&mut self, color: Rgb888) {
        self.bg_color = color;
    }
}
