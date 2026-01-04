# Phase 3: Implementation — GPU Terminal & Display System

**Feature**: GPU Refinement — Extended Scope (Phase 6, Task 6.2)
**Team**: TEAM_058
**Depends on**: `phase-2.md`

---

## Implementation Steps

### Step 1: Update QEMU Display Configuration

**Files**: `run.sh`, `run-pixel6.sh`

#### 1.1 Update `run.sh`

Change:
```bash
-display none \
-device virtio-gpu-device \
```

To:
```bash
-display gtk \
-device virtio-gpu-device,xres=1280,yres=800 \
```

#### 1.2 Update `run-pixel6.sh`

Change:
```bash
-display none \
-device virtio-gpu-device \
```

To:
```bash
-display gtk \
-device virtio-gpu-device,xres=2400,yres=1080 \
```

---

### Step 2: Add GPU Resolution Helper

**File**: `kernel/src/gpu.rs`

Add static method to get resolution:

```rust
impl GpuState {
    /// Get current screen resolution
    /// Returns None if GPU not initialized
    pub fn get_resolution() -> Option<(u32, u32)> {
        GPU.lock().as_ref().map(|s| (s.width, s.height))
    }
}
```

---

### Step 3: Create Terminal Module

**File**: `kernel/src/terminal.rs` (NEW)

```rust
//! GPU Terminal Emulator
//!
//! TEAM_058: Terminal emulation for Phase 6 GPU Refinement
//!
//! ## Behaviors
//! - [TERM1] Character rendering at cursor position
//! - [TERM2] Cursor advances after each character
//! - [TERM3] Newline moves to start of next line
//! - [TERM4] Screen scrolls when cursor exceeds rows
//! - [TERM5] Carriage return moves to start of current line
//! - [TERM6] Tab advances to next 8-column boundary
//! - [TERM7] Clear fills screen with background color
//! - [TERM8] Backspace moves cursor left (no erase)
//! - [TERM9] Resolution adapts to screen dimensions

use crate::gpu::{Display, GpuState, GPU};
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

/// Default terminal colors
pub const DEFAULT_FG: Rgb888 = Rgb888::new(204, 204, 204);  // Light gray
pub const DEFAULT_BG: Rgb888 = Rgb888::new(0, 0, 0);        // Black

/// Terminal emulator state
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
    /// [TERM9] Create terminal with auto-calculated dimensions
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        let cols = screen_width / FONT_WIDTH;
        let rows = screen_height / (FONT_HEIGHT + LINE_SPACING);

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
            '\n' => self.newline(display),           // [TERM3]
            '\r' => self.cursor_col = 0,             // [TERM5]
            '\t' => self.tab(display),               // [TERM6]
            '\x08' => self.backspace(),              // [TERM8]
            c if c >= ' ' => {
                // [TERM2] Check if we need to wrap
                if self.cursor_col >= self.cols {
                    self.newline(display);
                }

                // [TERM1] Render character
                let x = (self.cursor_col * FONT_WIDTH) as i32;
                let y = (self.cursor_row * (FONT_HEIGHT + LINE_SPACING)) as i32;

                let style = MonoTextStyle::new(&FONT_10X20, self.fg_color);
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);

                // Text baseline is at bottom of character
                let _ = Text::new(s, Point::new(x, y + FONT_HEIGHT as i32), style)
                    .draw(display);

                // [TERM2] Advance cursor
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

    /// [TERM3] [TERM4] Move to next line, scroll if needed
    pub fn newline(&mut self, display: &mut Display) {
        self.cursor_col = 0;
        self.cursor_row += 1;

        // [TERM4] Scroll if we've exceeded screen
        if self.cursor_row >= self.rows {
            self.scroll_up(display);
            self.cursor_row = self.rows - 1;
        }
    }

    /// [TERM6] Tab to next 8-column boundary
    fn tab(&mut self, display: &mut Display) {
        let next_tab = ((self.cursor_col / 8) + 1) * 8;
        if next_tab >= self.cols {
            self.newline(display);
        } else {
            self.cursor_col = next_tab;
        }
    }

    /// [TERM8] Move cursor left (no erase)
    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    /// [TERM7] Clear screen and reset cursor
    pub fn clear(&mut self, display: &mut Display) {
        let _ = Rectangle::new(
            Point::zero(),
            Size::new(self.screen_width, self.screen_height),
        )
        .into_styled(PrimitiveStyle::with_fill(self.bg_color))
        .draw(display);

        self.cursor_col = 0;
        self.cursor_row = 0;
    }

    /// [TERM4] Scroll screen up by one line
    fn scroll_up(&mut self, display: &mut Display) {
        let line_height = FONT_HEIGHT + LINE_SPACING;

        // Access framebuffer directly for efficient scroll
        let mut guard = GPU.lock();
        if let Some(state) = guard.as_mut() {
            let fb = state.framebuffer();
            let bytes_per_pixel = 4; // RGBA
            let row_bytes = self.screen_width as usize * bytes_per_pixel;
            let scroll_bytes = line_height as usize * row_bytes;

            // Copy everything up by one line
            if scroll_bytes < fb.len() {
                fb.copy_within(scroll_bytes.., 0);

                // Clear bottom line with background color
                let clear_start = fb.len() - scroll_bytes;
                for i in (clear_start..fb.len()).step_by(4) {
                    fb[i] = self.bg_color.r();
                    fb[i + 1] = self.bg_color.g();
                    fb[i + 2] = self.bg_color.b();
                    fb[i + 3] = 255;
                }
            }

            // Flush to display
            state.gpu.flush().ok();
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
```

---

### Step 4: Update main.rs

**File**: `kernel/src/main.rs`

#### 4.1 Add module declaration

After `mod net;`:
```rust
mod terminal;
```

#### 4.2 Replace graphics demo with terminal demo

Replace the "Verify Graphics" section (lines ~477-498) with:

```rust
// Initialize Terminal
verbose!("Initializing terminal...");
let mut display = gpu::Display;
let (width, height) = {
    let guard = gpu::GPU.lock();
    guard.as_ref().map(|s| (s.width, s.height)).unwrap_or((1280, 800))
};

let mut term = terminal::Terminal::new(width, height);
term.clear(&mut display);

// Display boot banner
term.write_str(&mut display, "LevitateOS Terminal v0.1\n");
term.write_str(&mut display, "========================\n\n");
let (cols, rows) = term.size();
// Note: Using separate print calls to avoid format! in early boot
term.write_str(&mut display, "Resolution: ");
// We'll print dimensions via UART for now
println!("Terminal: {}x{} chars at {}x{} pixels", cols, rows, width, height);
term.write_str(&mut display, "Type to see characters on screen...\n\n");
verbose!("Terminal initialized.");
```

#### 4.3 Update main loop to echo to terminal

In the main loop, update UART echo:
```rust
// Echo UART input to both serial and GPU terminal
if let Some(c) = levitate_hal::console::read_byte() {
    print!("{}", c as char);
    term.write_char(&mut display, c as char);
}
```

---

### Step 5: Update Behavior Inventory

**File**: `docs/testing/behavior-inventory.md`

Add Group 10: GPU Terminal section with TERM1-TERM9 behaviors.

---

## Implementation Order

1. **Step 1**: Update QEMU configs (5 min)
2. **Step 2**: Add GPU resolution helper (5 min)
3. **Step 3**: Create terminal.rs (30 min)
4. **Step 4**: Update main.rs (15 min)
5. **Step 5**: Update behavior inventory (10 min)
6. **Build & Test** (15 min)

**Total**: ~1.5 hours

---

## Verification Checklist

- [x] QEMU window appears (not headless)
- [x] Resolution detected correctly
- [x] Boot banner displays on screen
- [x] Characters type at cursor position
- [x] Newlines work correctly (TEAM_059: Fixed UART CR mapping)
- [x] Text wraps at screen edge
- [x] Screen scrolls when full
- [x] No panics during operation
- [x] Mouse cursor preserves text (TEAM_059: Fixed save/restore)

---

## Next Phase

Proceed to **Phase 4: Integration & Testing** after implementation.
