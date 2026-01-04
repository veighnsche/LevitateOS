# Phase 2: Design — GPU Terminal & Display System

**Feature**: GPU Refinement — Extended Scope (Phase 6, Task 6.2)
**Team**: TEAM_058
**Depends on**: `phase-1.md`

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      Terminal API                           │
│  write_char() | write_str() | newline() | clear() | scroll()│
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Terminal State                           │
│  cursor_col | cursor_row | cols | rows | fg_color | bg_color│
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│               Font Renderer (embedded-graphics)             │
│  MonoTextStyle | FONT_10X20 | Text primitive               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    GPU Display                              │
│  GpuState | framebuffer | width | height | flush()         │
└─────────────────────────────────────────────────────────────┘
```

---

## Module Design: `kernel/src/terminal.rs`

### Data Structures

```rust
/// Terminal configuration (compile-time or runtime)
pub struct TerminalConfig {
    pub font_width: u32,
    pub font_height: u32,
    pub line_spacing: u32,
    pub fg_color: Rgb888,
    pub bg_color: Rgb888,
}

/// Terminal state
pub struct Terminal {
    cursor_col: u32,
    cursor_row: u32,
    cols: u32,           // chars per line
    rows: u32,           // lines on screen
    config: TerminalConfig,
    screen_width: u32,   // pixels
    screen_height: u32,  // pixels
}

/// Default colors
pub const DEFAULT_FG: Rgb888 = Rgb888::new(204, 204, 204);  // Light gray
pub const DEFAULT_BG: Rgb888 = Rgb888::new(0, 0, 0);        // Black
```

### Public API

```rust
impl Terminal {
    /// Create terminal with auto-detected resolution
    pub fn new(screen_width: u32, screen_height: u32) -> Self;
    
    /// Create with custom config
    pub fn with_config(width: u32, height: u32, config: TerminalConfig) -> Self;
    
    /// Write single character
    pub fn write_char(&mut self, display: &mut Display, c: char);
    
    /// Write string
    pub fn write_str(&mut self, display: &mut Display, s: &str);
    
    /// Move to next line
    pub fn newline(&mut self, display: &mut Display);
    
    /// Clear screen and reset cursor
    pub fn clear(&mut self, display: &mut Display);
    
    /// Scroll screen up by one line
    fn scroll_up(&mut self, display: &mut Display);
    
    /// Get terminal dimensions
    pub fn size(&self) -> (u32, u32);  // (cols, rows)
    
    /// Get cursor position
    pub fn cursor(&self) -> (u32, u32);  // (col, row)
}
```

---

## Behavioral Contracts

### [TERM1] Character Rendering
- Characters render at current cursor position
- Character uses `config.fg_color` on `config.bg_color`
- After rendering, cursor advances by 1 column

### [TERM2] Cursor Advancement
- After each character, `cursor_col += 1`
- If `cursor_col >= cols`, automatic newline (TERM3)

### [TERM3] Newline Handling
- `\n` triggers: `cursor_col = 0`, `cursor_row += 1`
- If `cursor_row >= rows`, trigger scroll (TERM4)

### [TERM4] Scrolling
- When `cursor_row >= rows`:
  1. Copy framebuffer up by `font_height + line_spacing` pixels
  2. Clear bottom line with `bg_color`
  3. Set `cursor_row = rows - 1`

### [TERM5] Carriage Return
- `\r` sets `cursor_col = 0` (no row change)

### [TERM6] Tab Character
- `\t` advances to next 8-column boundary
- If would exceed line, wrap to next line

### [TERM7] Clear Screen
- Fill entire framebuffer with `bg_color`
- Reset `cursor_col = 0`, `cursor_row = 0`

### [TERM8] Backspace
- `\x08` moves cursor left by 1 (if `cursor_col > 0`)
- Does NOT erase character (standard behavior)

### [TERM9] Resolution Adaptation
- Terminal calculates `cols` and `rows` from screen dimensions
- `cols = screen_width / font_width`
- `rows = screen_height / (font_height + line_spacing)`

---

## Scrolling Implementation

### Strategy: Direct Framebuffer Copy

```rust
fn scroll_up(&mut self, display: &mut Display) {
    let line_height = self.config.font_height + self.config.line_spacing;
    let bytes_per_pixel = 4;  // RGBA
    let row_bytes = self.screen_width as usize * bytes_per_pixel;
    let scroll_bytes = line_height as usize * row_bytes;
    
    let fb = display.framebuffer_mut();
    
    // Copy everything up by one line
    fb.copy_within(scroll_bytes.., 0);
    
    // Clear bottom line
    let clear_start = fb.len() - scroll_bytes;
    for i in (clear_start..fb.len()).step_by(4) {
        fb[i] = self.config.bg_color.r();
        fb[i + 1] = self.config.bg_color.g();
        fb[i + 2] = self.config.bg_color.b();
        fb[i + 3] = 255;
    }
}
```

### Why This Approach
- **Pros**: Simple, no extra memory, uses `copy_within` (efficient)
- **Cons**: Requires mutable framebuffer access
- **Alternative**: Ring buffer of line offsets (more complex, deferred)

---

## Display Configuration Updates

### `run.sh` Changes

```bash
# OLD:
-display none \
-device virtio-gpu-device \

# NEW:
-display gtk \
-device virtio-gpu-device,xres=1280,yres=800 \
```

### `run-pixel6.sh` Changes

```bash
# OLD:
-display none \
-device virtio-gpu-device \

# NEW:
-display gtk \
-device virtio-gpu-device,xres=2400,yres=1080 \
```

---

## Font Selection Logic

```rust
/// Select appropriate font based on screen width
pub fn select_font(screen_width: u32) -> &'static MonoFont<'static> {
    match screen_width {
        0..=800 => &FONT_6X10,      // Small screens
        801..=1280 => &FONT_8X13,   // Medium (QEMU default)
        1281..=1920 => &FONT_10X20, // Full HD
        _ => &FONT_10X20,           // 2K+ (Pixel 6)
    }
}
```

**Decision**: Use `FONT_10X20` as baseline for readability. Dynamic selection available but not required for MVP.

---

## Color Scheme

### Default Terminal Colors

```rust
pub const COLOR_BLACK: Rgb888 = Rgb888::new(0, 0, 0);
pub const COLOR_RED: Rgb888 = Rgb888::new(204, 0, 0);
pub const COLOR_GREEN: Rgb888 = Rgb888::new(0, 204, 0);
pub const COLOR_YELLOW: Rgb888 = Rgb888::new(204, 204, 0);
pub const COLOR_BLUE: Rgb888 = Rgb888::new(0, 0, 204);
pub const COLOR_MAGENTA: Rgb888 = Rgb888::new(204, 0, 204);
pub const COLOR_CYAN: Rgb888 = Rgb888::new(0, 204, 204);
pub const COLOR_WHITE: Rgb888 = Rgb888::new(204, 204, 204);

// Bright variants (P2 feature)
pub const COLOR_BRIGHT_WHITE: Rgb888 = Rgb888::new(255, 255, 255);
```

---

## GPU Module Updates

### Required Changes to `gpu.rs`

```rust
impl GpuState {
    // EXISTING
    pub fn framebuffer(&mut self) -> &mut [u8];
    pub fn dimensions(&self) -> (u32, u32);
    
    // NEW: Get resolution for terminal initialization
    pub fn resolution() -> Option<(u32, u32)> {
        GPU.lock().as_ref().map(|s| (s.width, s.height))
    }
}
```

---

## Integration with main.rs

### Demo Usage

```rust
// After GPU init, before main loop:
let (width, height) = gpu::GpuState::resolution().unwrap_or((1280, 800));
let mut terminal = terminal::Terminal::new(width, height);
let mut display = gpu::Display;

terminal.clear(&mut display);
terminal.write_str(&mut display, "LevitateOS Terminal v0.1\n");
terminal.write_str(&mut display, "========================\n\n");
terminal.write_str(&mut display, "Type something...\n");

// In main loop, echo UART to terminal:
if let Some(c) = levitate_hal::console::read_byte() {
    print!("{}", c as char);  // UART echo (existing)
    terminal.write_char(&mut display, c as char);  // GPU echo (new)
}
```

---

## Behavior IDs Summary

| ID | Behavior | Test Strategy |
|----|----------|---------------|
| TERM1 | Character rendering | Visual + coordinate check |
| TERM2 | Cursor advancement | State inspection |
| TERM3 | Newline handling | Visual + state check |
| TERM4 | Scrolling | Visual verification |
| TERM5 | Carriage return | State inspection |
| TERM6 | Tab character | Position calculation |
| TERM7 | Clear screen | Visual verification |
| TERM8 | Backspace | State inspection |
| TERM9 | Resolution adaptation | Multiple resolution test |

---

## Open Questions (Resolved)

| # | Question | Resolution |
|---|----------|------------|
| 1 | Display mode | `-display gtk` for development |
| 2 | Target resolution | Auto-detect from device |
| 3 | Font size | `FONT_10X20` baseline |
| 4 | Scrolling | Direct framebuffer copy |
| 5 | Color scheme | Classic terminal (gray on black) |

---

## Next Phase

Proceed to **Phase 3: Implementation** with step-by-step code changes.
