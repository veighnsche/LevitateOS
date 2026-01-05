//! Platform-agnostic Terminal Emulator
//! TEAM_092: Extracted from kernel/src/terminal.rs

#![no_std]

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnsiState {
    None,
    Esc,
    CSI,
}

pub struct Terminal {
    pub cursor_col: u32,
    pub cursor_row: u32,
    pub cols: u32,
    pub rows: u32,
    config: TerminalConfig,
    screen_width: u32,
    screen_height: u32,
    cursor_visible: bool,
    last_blink: u64,
    saved_pixels: [[Rgb888; 10]; 20],
    has_saved: bool,
    ansi_state: AnsiState,
}

impl Terminal {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        let cols = screen_width / FONT_WIDTH;
        let rows = screen_height / (FONT_HEIGHT + LINE_SPACING);

        Self {
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

                let x = (self.cursor_col * FONT_WIDTH) as i32;
                let y = (self.cursor_row * (FONT_HEIGHT + LINE_SPACING)) as i32;

                let style = MonoTextStyle::new(&FONT_10X20, self.config.fg_color);
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);

                let _ = Text::new(s, Point::new(x, y + FONT_HEIGHT as i32), style).draw(target);
                self.cursor_col += 1;
            }
            _ => {}
        }
        self.show_cursor(target);
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

    fn scroll_up<D>(&mut self, _target: &mut D)
    where
        D: DrawTarget<Color = Rgb888>,
    {
        // TODO: Scrolling is hardware-specific (requires copy_within or similar)
        // For a generic DrawTarget, we might need a DrawTarget + Flushable + Scrollable trait
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

        // TODO: Cursor saving logic also needs direct framebuffer access or a ReadTarget trait
        // For now, we just draw the block
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

        // Just erase with background for now
        let _ = Rectangle::new(Point::new(x as i32, y as i32), Size::new(10, 20))
            .into_styled(PrimitiveStyle::with_fill(self.config.bg_color))
            .draw(target);

        self.cursor_visible = false;
    }
}
