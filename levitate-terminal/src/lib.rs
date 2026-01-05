//! Platform-agnostic Terminal Emulator
//! TEAM_092: Extracted from kernel/src/terminal.rs
//! TEAM_116: Added text buffer for proper scrolling support (heap-allocated)

#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};

/// Font dimensions (FONT_10X20)
const FONT_WIDTH: u32 = 10;
const FONT_HEIGHT: u32 = 20;
const LINE_SPACING: u32 = 2;
const CHARACTER_SPACING: u32 = 0;

pub const DEFAULT_FG: Rgb888 = Rgb888::new(204, 204, 204);
pub const DEFAULT_BG: Rgb888 = Rgb888::new(0, 0, 0);

pub struct TerminalConfig {
    pub font_width: u32,
    pub font_height: u32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum AnsiState {
    #[default]
    None,
    Esc,
    CSI,
}

/// TEAM_116: Text buffer for scrolling support
/// Uses heap allocation to avoid stack overflow
pub struct Terminal {
    pub cursor_col: u32,
    pub cursor_row: u32,
    pub cols: u32,
    pub rows: u32,
    config: TerminalConfig,
    screen_width: u32,
    screen_height: u32,
    cursor_visible: bool,
    ansi_state: AnsiState,
    /// Character buffer: flattened [row * cols + col]
    buffer: Vec<char>,
}

impl Terminal {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        let cols = screen_width / FONT_WIDTH;
        let rows = screen_height / (FONT_HEIGHT + LINE_SPACING);

        // Allocate buffer on heap
        let buffer_size = (cols * rows) as usize;
        let buffer = vec![' '; buffer_size];

        Self {
            cursor_col: 0,
            cursor_row: 0,
            cols,
            rows,
            config: TerminalConfig::default(),
            screen_width,
            screen_height,
            cursor_visible: false,
            ansi_state: AnsiState::None,
            buffer,
        }
    }

    /// Get character at position (bounds-checked)
    fn get_char(&self, col: u32, row: u32) -> char {
        if col < self.cols && row < self.rows {
            let idx = (row * self.cols + col) as usize;
            self.buffer.get(idx).copied().unwrap_or(' ')
        } else {
            ' '
        }
    }

    /// Set character at position (bounds-checked)
    fn set_char(&mut self, col: u32, row: u32, c: char) {
        if col < self.cols && row < self.rows {
            let idx = (row * self.cols + col) as usize;
            if let Some(cell) = self.buffer.get_mut(idx) {
                *cell = c;
            }
        }
    }

    pub fn write_char<D>(&mut self, target: &mut D, c: char)
    where
        D: DrawTarget<Color = Rgb888>,
    {
        self.hide_cursor(target);

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
                if c == 'J' {
                    self.clear(target);
                }
                if (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') {
                    self.ansi_state = AnsiState::None;
                }
                return;
            }
        }

        match c {
            '\n' => self.newline(target),
            '\r' => self.carriage_return(target),
            '\t' => self.tab(target),
            '\x08' => self.backspace(target),
            c if c >= ' ' => {
                if self.cursor_col >= self.cols {
                    self.newline(target);
                }

                // Store in buffer
                self.set_char(self.cursor_col, self.cursor_row, c);

                // Draw the character
                self.draw_char_at(target, c, self.cursor_col, self.cursor_row);
                self.cursor_col += 1;
            }
            _ => {}
        }
        self.show_cursor(target);
    }

    /// Draw a single character at the specified grid position
    fn draw_char_at<D>(&self, target: &mut D, c: char, col: u32, row: u32)
    where
        D: DrawTarget<Color = Rgb888>,
    {
        let x = (col * FONT_WIDTH) as i32;
        let y = (row * (FONT_HEIGHT + LINE_SPACING)) as i32;

        // Clear background first
        let _ = Rectangle::new(
            Point::new(x, y),
            Size::new(FONT_WIDTH, FONT_HEIGHT + LINE_SPACING),
        )
        .into_styled(PrimitiveStyle::with_fill(self.config.bg_color))
        .draw(target);

        // Draw character
        let style = MonoTextStyle::new(&FONT_10X20, self.config.fg_color);
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        let _ = Text::new(s, Point::new(x, y + FONT_HEIGHT as i32), style).draw(target);
    }

    pub fn write_str<D>(&mut self, target: &mut D, s: &str)
    where
        D: DrawTarget<Color = Rgb888>,
    {
        for c in s.chars() {
            self.write_char(target, c);
        }
    }

    pub fn newline<D>(&mut self, target: &mut D)
    where
        D: DrawTarget<Color = Rgb888>,
    {
        self.hide_cursor(target);
        self.cursor_col = 0;
        self.cursor_row += 1;

        if self.cursor_row >= self.rows {
            self.scroll_up(target);
            self.cursor_row = self.rows - 1;
        }
        self.show_cursor(target);
    }

    pub fn carriage_return<D>(&mut self, target: &mut D)
    where
        D: DrawTarget<Color = Rgb888>,
    {
        self.hide_cursor(target);
        self.cursor_col = 0;
        self.show_cursor(target);
    }

    fn tab<D>(&mut self, target: &mut D)
    where
        D: DrawTarget<Color = Rgb888>,
    {
        self.hide_cursor(target);
        let next_tab = ((self.cursor_col / 8) + 1) * 8;
        if next_tab >= self.cols {
            self.newline(target);
        } else {
            self.cursor_col = next_tab;
        }
        self.show_cursor(target);
    }

    fn backspace<D>(&mut self, target: &mut D)
    where
        D: DrawTarget<Color = Rgb888>,
    {
        self.hide_cursor(target);
        let mut changed = false;
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            changed = true;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cols - 1;
            changed = true;
        }

        if changed {
            // Clear character from buffer
            self.set_char(self.cursor_col, self.cursor_row, ' ');

            let x = self.cursor_col * (FONT_WIDTH + CHARACTER_SPACING);
            let y = self.cursor_row * (FONT_HEIGHT + LINE_SPACING);

            let _ = Rectangle::new(
                Point::new(x as i32, y as i32),
                Size::new(FONT_WIDTH, FONT_HEIGHT + LINE_SPACING),
            )
            .into_styled(PrimitiveStyle::with_fill(self.config.bg_color))
            .draw(target);
        }
        self.show_cursor(target);
    }

    pub fn clear<D>(&mut self, target: &mut D)
    where
        D: DrawTarget<Color = Rgb888>,
    {
        self.hide_cursor(target);

        // Clear buffer
        for cell in self.buffer.iter_mut() {
            *cell = ' ';
        }

        let _ = Rectangle::new(
            Point::zero(),
            Size::new(self.screen_width, self.screen_height),
        )
        .into_styled(PrimitiveStyle::with_fill(self.config.bg_color))
        .draw(target);
        self.cursor_col = 0;
        self.cursor_row = 0;
        self.show_cursor(target);
    }

    /// TEAM_116: Scroll up by shifting buffer contents and redrawing
    fn scroll_up<D>(&mut self, target: &mut D)
    where
        D: DrawTarget<Color = Rgb888>,
    {
        let rows = self.rows as usize;
        let cols = self.cols as usize;

        // Shift buffer up by one row (copy row N to row N-1)
        for row in 1..rows {
            for col in 0..cols {
                let src_idx = row * cols + col;
                let dst_idx = (row - 1) * cols + col;
                if let Some(&c) = self.buffer.get(src_idx) {
                    if let Some(cell) = self.buffer.get_mut(dst_idx) {
                        *cell = c;
                    }
                }
            }
        }

        // Clear the last row in buffer
        let last_row_start = (rows - 1) * cols;
        for col in 0..cols {
            if let Some(cell) = self.buffer.get_mut(last_row_start + col) {
                *cell = ' ';
            }
        }

        // Clear screen and redraw from buffer
        let _ = Rectangle::new(
            Point::zero(),
            Size::new(self.screen_width, self.screen_height),
        )
        .into_styled(PrimitiveStyle::with_fill(self.config.bg_color))
        .draw(target);

        // Redraw all characters from buffer
        for row in 0..rows {
            for col in 0..cols {
                let c = self.get_char(col as u32, row as u32);
                if c != ' ' {
                    self.draw_char_at(target, c, col as u32, row as u32);
                }
            }
        }
    }

    fn show_cursor<D>(&mut self, target: &mut D)
    where
        D: DrawTarget<Color = Rgb888>,
    {
        if self.cursor_visible {
            return;
        }

        let x = self.cursor_col * (FONT_WIDTH + CHARACTER_SPACING);
        let y = self.cursor_row * (FONT_HEIGHT + LINE_SPACING);

        let _ = Rectangle::new(Point::new(x as i32, y as i32), Size::new(10, 20))
            .into_styled(PrimitiveStyle::with_fill(self.config.fg_color))
            .draw(target);
        self.cursor_visible = true;
    }

    fn hide_cursor<D>(&mut self, target: &mut D)
    where
        D: DrawTarget<Color = Rgb888>,
    {
        if !self.cursor_visible {
            return;
        }
        let x = self.cursor_col * (FONT_WIDTH + CHARACTER_SPACING);
        let y = self.cursor_row * (FONT_HEIGHT + LINE_SPACING);

        // Redraw the character under the cursor (or space if empty)
        let c = self.get_char(self.cursor_col, self.cursor_row);

        if c != ' ' {
            self.draw_char_at(target, c, self.cursor_col, self.cursor_row);
        } else {
            let _ = Rectangle::new(Point::new(x as i32, y as i32), Size::new(10, 20))
                .into_styled(PrimitiveStyle::with_fill(self.config.bg_color))
                .draw(target);
        }

        self.cursor_visible = false;
    }
}
