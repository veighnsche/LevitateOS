# Phase 1: Discovery — GPU Terminal & Display System

**Feature**: GPU Refinement — Extended Scope (Phase 6, Task 6.2)
**Team**: TEAM_058

---

## Current State Analysis

### What the GPU is Currently Doing

**File**: `kernel/src/gpu.rs`

1. **Initialization** (`init()`):
   - Queries resolution from virtio-gpu device
   - Sets up framebuffer (DMA-allocated)
   - Stores width, height, framebuffer pointer in `GpuState`

2. **Drawing** (`main.rs` lines 477-498):
   - Draws black background (was commented as "blue" but is RGB 0,0,0)
   - Draws red 200x200 rectangle at (100, 100)
   - Handles cursor tracking via `input::poll()` → `cursor::draw()`

3. **Cursor** (`kernel/src/cursor.rs`):
   - Simple 10x10 white rectangle
   - Position updated by virtio-input tablet events

### Current Resolution Configuration

**QEMU virtio-gpu-device defaults**:
```
xres=1280 (default)
yres=800  (default)
```

**Both `run.sh` and `run-pixel6.sh` use**:
```bash
-display none  # HEADLESS! No visual output!
```

⚠️ **Critical Issue**: `-display none` means the framebuffer exists in memory but nothing is displayed. To see output, need `-display gtk` or `-display sdl`.

### Target: Pixel 6 Oriole Display

| Property | Value |
|----------|-------|
| Resolution | **2400 × 1080** (FHD+) |
| Aspect Ratio | 20:9 |
| PPI | 411 |
| Size | 6.4" diagonal |

---

## Resolution Analysis

### Font Size Calculations for Different Resolutions

#### QEMU Default (1280×800)

| Font | Chars/Line | Lines | Total Chars |
|------|------------|-------|-------------|
| 6×10 | 213 | 80 | 17,040 |
| 8×13 | 160 | 61 | 9,760 |
| 10×20 | 128 | 40 | 5,120 |

#### Pixel 6 (2400×1080)

| Font | Chars/Line | Lines | Total Chars |
|------|------------|-------|-------------|
| 6×10 | 400 | 108 | 43,200 |
| 8×13 | 300 | 83 | 24,900 |
| 10×20 | 240 | 54 | 12,960 |
| **12×24** | 200 | 45 | 9,000 |
| **16×32** | 150 | 33 | 4,950 |

**Recommendation for Pixel 6**: 
- `FONT_10X20` gives 240×54 = good terminal
- At 411 PPI, 10×20 pixels = ~0.6mm × 1.2mm (very small physically)
- Consider `12×24` or `16×32` for readability on actual device

---

## Questions to Resolve

### Q1: Display Output Mode
**Issue**: `-display none` means we can't see anything.
**Options**:
- A) Add `-display gtk` or `-display sdl` for development
- B) Keep headless, verify via screenshots/dumps
- C) Use `-vnc :0` for remote viewing

### Q2: Target Resolution
**Issue**: Should we target QEMU default (1280×800) or Pixel 6 (2400×1080)?
**Options**:
- A) Use QEMU default for now, abstract resolution in code
- B) Set explicit resolution: `-device virtio-gpu-device,xres=2400,yres=1080`
- C) Make resolution configurable via compile-time feature

### Q3: Font Size
**Issue**: What font for Pixel 6's 2400×1080 at 411 PPI?
**Options**:
- A) `FONT_10X20` — 240 chars × 54 lines (good density)
- B) `FONT_8X13` — 300 chars × 83 lines (more text, harder to read)
- C) Custom larger font (16×32+) for actual device readability

### Q4: Scrolling Implementation
**Issue**: When text fills screen, what to do?
**Options**:
- A) Wrap to top (simple, current implied behavior)
- B) Scroll up (requires framebuffer copy)
- C) Ring buffer of lines (more complex)

---

## Recommended QEMU Configuration Update

```bash
# For development with visual output:
-display gtk \
-device virtio-gpu-device,xres=2400,yres=1080

# Or for Pixel 6 emulation:
-display gtk \
-device virtio-gpu-device,xres=2400,yres=1080
```

---

## Dependencies

**Already available**:
- `embedded-graphics` v0.8.1 ✅
- `embedded-graphics::mono_font` (built-in fonts) ✅

**Available fonts in `embedded-graphics`**:
```rust
use embedded_graphics::mono_font::ascii::{
    FONT_4X6,    // Tiny
    FONT_5X7,
    FONT_5X8,
    FONT_6X9,
    FONT_6X10,
    FONT_6X12,
    FONT_6X13,
    FONT_7X13,
    FONT_7X14,
    FONT_8X13,
    FONT_9X15,
    FONT_9X18,
    FONT_10X20,  // Recommended baseline
};
```

**Larger fonts** require external crate or custom bitmap font.

---

---

## Extended Feature Scope

### Original Task 6.2 Scope
> "GPU Refinement: Text rendering or terminal emulation on GPU framebuffer"

### Extended Scope (TEAM_058)

| Component | Description | Priority |
|-----------|-------------|----------|
| **Display Configuration** | Fix `-display none`, configure resolution | P0 |
| **Resolution Detection** | Query and adapt to actual display size | P0 |
| **Font System** | Resolution-aware font selection | P0 |
| **Terminal Core** | Character output, cursor, newlines | P0 |
| **Scrolling** | Proper scroll-up when buffer full | P1 |
| **ANSI Colors** | Basic 8/16 color support | P1 |
| **GPU println!** | Optional: mirror kernel output to GPU | P2 |
| **Cursor Blinking** | Visual cursor feedback | P2 |

---

## Success Criteria (Granular Checkpoints)

### SC1: QEMU Display Configuration
- [x] **SC1.1**: `run.sh` updated with `-display gtk`
- [x] **SC1.2**: `run.sh` updated with explicit resolution `xres=1280,yres=800`
- [x] **SC1.3**: QEMU window opens on `./run.sh` (not headless)
- [x] **SC1.4**: Framebuffer content visible in QEMU window
- [x] **SC1.5**: `run-pixel6.sh` updated with `-display gtk`
- [x] **SC1.6**: `run-pixel6.sh` updated with `xres=2400,yres=1080`

### SC2: Resolution Detection
- [x] **SC2.1**: `GpuState::get_resolution()` method exists
- [x] **SC2.2**: Returns `Some((1280, 800))` on QEMU default
- [x] **SC2.3**: Returns `Some((2400, 1080))` on Pixel 6 config
- [x] **SC2.4**: Returns `None` if GPU not initialized
- [x] **SC2.5**: Resolution printed to UART on boot

### SC3: Terminal Module Creation
- [x] **SC3.1**: `kernel/src/terminal.rs` file exists
- [x] **SC3.2**: `Terminal` struct defined with cursor_col, cursor_row
- [x] **SC3.3**: `Terminal::new(width, height)` compiles
- [x] **SC3.4**: Cols calculated correctly: `width / FONT_WIDTH`
- [x] **SC3.5**: Rows calculated correctly: `height / (FONT_HEIGHT + LINE_SPACING)`
- [x] **SC3.6**: `mod terminal;` added to main.rs
- [x] **SC3.7**: Module compiles without errors

### SC4: Font Rendering
- [x] **SC4.1**: `FONT_10X20` imported from embedded-graphics
- [x] **SC4.2**: `MonoTextStyle` created with font and color
- [x] **SC4.3**: Single character 'A' renders at (0, 0)
- [x] **SC4.4**: Character visible on screen (not blank)
- [x] **SC4.5**: Character color matches `DEFAULT_FG` (light gray)
- [x] **SC4.6**: Background color matches `DEFAULT_BG` (black)

### SC5: Character Positioning
- [x] **SC5.1**: First character appears at pixel (0, 20) (baseline)
- [x] **SC5.2**: Second character appears at pixel (10, 20)
- [x] **SC5.3**: `cursor_col` increments after each character
- [x] **SC5.4**: Typing "ABC" produces three adjacent characters
- [x] **SC5.5**: Characters don't overlap

### SC6: Newline Handling
- [x] **SC6.1**: `\n` resets `cursor_col` to 0
- [x] **SC6.2**: `\n` increments `cursor_row` by 1
- [x] **SC6.3**: Text after newline starts at left edge
- [x] **SC6.4**: Second line appears 22 pixels below first (20 + 2 spacing)
- [x] **SC6.5**: Multiple newlines create blank lines

### SC7: Carriage Return
- [x] **SC7.1**: `\r` resets `cursor_col` to 0
- [x] **SC7.2**: `\r` does NOT change `cursor_row`
- [x] **SC7.3**: `\r\n` sequence works (Windows-style)

### SC8: Line Wrapping
- [x] **SC8.1**: Character at `cols-1` renders correctly
- [x] **SC8.2**: Character at `cols` triggers automatic newline
- [x] **SC8.3**: Continued text appears on next line
- [x] **SC8.4**: No characters lost during wrap
- [x] **SC8.5**: Long string wraps multiple times correctly

### SC9: Tab Character
- [x] **SC9.1**: `\t` advances cursor to next 8-column boundary
- [x] **SC9.2**: Tab from col 0 → col 8
- [x] **SC9.3**: Tab from col 5 → col 8
- [x] **SC9.4**: Tab from col 8 → col 16
- [x] **SC9.5**: Tab near end of line wraps to next line

### SC10: Clear Screen
- [x] **SC10.1**: `clear()` fills screen with background color
- [x] **SC10.2**: `clear()` resets `cursor_col` to 0
- [x] **SC10.3**: `clear()` resets `cursor_row` to 0
- [x] **SC10.4**: All previous text removed from display
- [x] **SC10.5**: Cursor positioned at top-left after clear

### SC11: Scrolling
- [x] **SC11.1**: Text fills entire screen height
- [x] **SC11.2**: Next character triggers scroll (not wrap-to-top)
- [x] **SC11.3**: Top line disappears after scroll
- [x] **SC11.4**: All other lines move up by one
- [x] **SC11.5**: New blank line appears at bottom
- [x] **SC11.6**: Cursor remains on last row after scroll
- [x] **SC11.7**: Multiple scrolls work consecutively
- [x] **SC11.8**: Content integrity maintained during scroll

### SC12: Backspace
- [x] **SC12.1**: `\x08` decrements `cursor_col` by 1
- [x] **SC12.2**: Backspace at col 0 does nothing (no wrap-back)
- [x] **SC12.3**: Backspace does NOT erase character (standard behavior)

### SC13: Multi-Resolution Support
- [x] **SC13.1**: Works at 1280×800 (128×36 chars)
- [x] **SC13.2**: Works at 2400×1080 (240×49 chars)
- [x] **SC13.3**: No hardcoded pixel values in terminal.rs
- [x] **SC13.4**: Font renders same size at both resolutions
- [x] **SC13.5**: Scrolling works at both resolutions

### SC14: Integration
- [x] **SC14.1**: Boot banner "LevitateOS Terminal v0.1" displays
- [x] **SC14.2**: Terminal dimensions printed to UART
- [x] **SC14.3**: UART input echoes to GPU terminal
- [x] **SC14.4**: Typing characters shows them on screen
- [x] **SC14.5**: Enter key produces newline on GPU
- [x] **SC14.6**: Existing cursor tracking still works
- [x] **SC14.7**: No panics during normal operation

### SC15: Build & Test
- [x] **SC15.1**: `cargo build --release` succeeds
- [x] **SC15.2**: All 34 existing tests pass
- [x] **SC15.3**: No new warnings (except known ones)
- [x] **SC15.4**: Behavior inventory updated with TERM1-TERM9 (TEAM_059: Completed)

---

## Files to Create/Modify

| File | Action | Purpose |
|------|--------|---------|
| `kernel/src/terminal.rs` | **CREATE** | Terminal emulator core |
| `kernel/src/gpu.rs` | MODIFY | Add resolution getter, text helpers |
| `kernel/src/main.rs` | MODIFY | Add `mod terminal;`, demo usage |
| `run.sh` | MODIFY | Add `-display gtk`, resolution config |
| `run-pixel6.sh` | MODIFY | Add `-display gtk`, Pixel 6 resolution |

---

## Constraints

1. **No external font crates** — Use `embedded-graphics` built-in fonts only
2. **No async** — Must work with current synchronous architecture
3. **Memory efficient** — No large text buffer; render directly to framebuffer
4. **Stable first** — Simple working implementation before advanced features

---

## Decisions Made (Based on USER Preference: "stable, working, readable")

| Question | Decision | Rationale |
|----------|----------|-----------|
| Q1: Display mode | `-display gtk` | Must see output for development |
| Q2: Resolution | Detect from device | Flexible for both QEMU and Pixel 6 |
| Q3: Font | `FONT_10X20` baseline | Good readability, decent density |
| Q4: Scrolling | Scroll up (proper) | Professional terminal behavior |

---

## Next Phase

Proceed to **Phase 2: Design** to define terminal architecture and behavior contracts.
