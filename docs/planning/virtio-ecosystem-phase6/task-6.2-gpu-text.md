# Task 6.2: GPU Text Rendering / Terminal Emulator

## Phase 1: Discovery

### Feature Summary
- **Feature**: Text rendering on VirtIO GPU framebuffer
- **Problem Statement**: LevitateOS can only output text via UART serial. GPU displays graphics but no text, limiting visual feedback capabilities.
- **Benefits**: Visual terminal output, foundation for future shell/console, better debugging on graphical displays

### Success Criteria
- [ ] Render text string on GPU framebuffer
- [ ] Implement character-by-character output
- [ ] Handle newlines and cursor positioning
- [ ] Basic scrolling when buffer full (optional)

### Current State Analysis

**Existing GPU Driver** (`kernel/src/gpu.rs`):
```rust
pub struct GpuState {
    gpu: VirtIOGpu<VirtioHal, StaticMmioTransport>,
    fb_ptr: usize,
    fb_len: usize,
    width: u32,
    height: u32,
}

pub struct Display;
impl DrawTarget for Display { ... }  // Can draw pixels/shapes
```

**embedded-graphics Text Support** (already in Cargo.toml):
```rust
// Available built-in fonts
use embedded_graphics::mono_font::ascii::{
    FONT_6X10,   // Compact: 6px wide, 10px tall
    FONT_8X13,   // Medium: 8px wide, 13px tall  
    FONT_10X20,  // Large: 10px wide, 20px tall
};

use embedded_graphics::text::Text;
use embedded_graphics::mono_font::MonoTextStyle;
```

### Codebase Reconnaissance

**Files to create/modify**:
| File | Change |
|------|--------|
| `kernel/src/terminal.rs` | **NEW** — Terminal emulator |
| `kernel/src/main.rs` | Add `mod terminal;`, integrate output |

**No new dependencies needed** — `embedded-graphics` already supports text.

---

## Phase 2: Implementation Plan

### Step 1: Create `kernel/src/terminal.rs`

```rust
//! GPU Terminal Emulator
//!
//! Renders text to the VirtIO GPU framebuffer using embedded-graphics.

use crate::gpu::Display;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    text::Text,
    primitives::{Rectangle, PrimitiveStyle},
};

const FONT_WIDTH: u32 = 6;
const FONT_HEIGHT: u32 = 10;
const LINE_SPACING: u32 = 2;  // Extra pixels between lines

pub struct Terminal {
    cursor_col: u32,
    cursor_row: u32,
    cols: u32,
    rows: u32,
    fg_color: Rgb888,
    bg_color: Rgb888,
}

impl Terminal {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        let cols = screen_width / FONT_WIDTH;
        let rows = screen_height / (FONT_HEIGHT + LINE_SPACING);
        
        Terminal {
            cursor_col: 0,
            cursor_row: 0,
            cols,
            rows,
            fg_color: Rgb888::new(255, 255, 255),  // White
            bg_color: Rgb888::new(0, 0, 0),        // Black
        }
    }
    
    pub fn write_char(&mut self, display: &mut Display, c: char) {
        match c {
            '\n' => self.newline(display),
            '\r' => self.cursor_col = 0,
            c => {
                if self.cursor_col >= self.cols {
                    self.newline(display);
                }
                
                let x = (self.cursor_col * FONT_WIDTH) as i32;
                let y = (self.cursor_row * (FONT_HEIGHT + LINE_SPACING)) as i32;
                
                let style = MonoTextStyle::new(&FONT_6X10, self.fg_color);
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                
                let _ = Text::new(s, Point::new(x, y + FONT_HEIGHT as i32), style)
                    .draw(display);
                
                self.cursor_col += 1;
            }
        }
    }
    
    pub fn write_str(&mut self, display: &mut Display, s: &str) {
        for c in s.chars() {
            self.write_char(display, c);
        }
    }
    
    fn newline(&mut self, display: &mut Display) {
        self.cursor_col = 0;
        self.cursor_row += 1;
        
        if self.cursor_row >= self.rows {
            self.scroll(display);
            self.cursor_row = self.rows - 1;
        }
    }
    
    fn scroll(&mut self, _display: &mut Display) {
        // For now: just wrap to top (simple implementation)
        // TODO: Implement proper scrolling by copying framebuffer up
        self.cursor_row = 0;
        self.cursor_col = 0;
    }
    
    pub fn clear(&mut self, display: &mut Display) {
        let size = display.size();
        let _ = Rectangle::new(Point::zero(), size)
            .into_styled(PrimitiveStyle::with_fill(self.bg_color))
            .draw(display);
        
        self.cursor_col = 0;
        self.cursor_row = 0;
    }
}
```

### Step 2: Update `kernel/src/main.rs`

```rust
mod terminal;

// In kmain(), after GPU init:
let mut term = terminal::Terminal::new(1024, 768);
let mut display = gpu::Display;
term.clear(&mut display);
term.write_str(&mut display, "LevitateOS Terminal\n");
term.write_str(&mut display, "==================\n\n");
```

### Step 3: Optional — Integrate with println!

Create a GPU-backed print macro (future enhancement):
```rust
// Could replace or supplement UART output
macro_rules! gprintln {
    ($($arg:tt)*) => {
        // Write to GPU terminal
    };
}
```

---

## Phase 3: Font Options

| Font | Size | Characters/Line (1024px) | Lines (768px) |
|------|------|--------------------------|---------------|
| FONT_6X10 | 6×10 | 170 | 64 |
| FONT_8X13 | 8×13 | 128 | 51 |
| FONT_10X20 | 10×20 | 102 | 35 |

**Recommendation**: Use `FONT_10X20` for readability (per USER preference).

---

## Phase 4: Testing

### Manual Test
1. Build and run kernel
2. Verify text appears on QEMU GPU display
3. Test newline handling
4. Test wrap-around at screen edges

### Visual Verification
```
Expected output on GPU:
┌────────────────────────────────────┐
│ LevitateOS Terminal                │
│ ==================                 │
│                                    │
│ Hello, GPU World!                  │
│                                    │
└────────────────────────────────────┘
```

---

## Estimated Effort

| Task | Time |
|------|------|
| Create terminal.rs | 1 hour |
| Update main.rs | 15 min |
| Testing | 30 min |
| Font tuning | 15 min |
| **Total** | **~2 hours** |

---

## Future Enhancements (Not in Phase 6)

- [ ] Proper framebuffer scrolling
- [ ] ANSI escape code support (colors, cursor movement)
- [ ] Input handling (keyboard → terminal)
- [ ] Full VT100/ANSI terminal emulation
