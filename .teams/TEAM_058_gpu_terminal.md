# TEAM_058: GPU Terminal & Text Rendering

**Date**: 2026-01-04
**Status**: Partial — Handoff to Next Team
**Feature**: Phase 6 Task 6.2 — GPU Refinement (Extended Scope)

## Objective

Implement comprehensive GPU terminal emulation with text rendering, proper resolution handling, and display configuration for both QEMU development and Pixel 6 target hardware.

## Extended Scope (Beyond Original Task 6.2)

Original scope: "Text rendering or terminal emulation on GPU framebuffer"

**Extended to include**:
1. Resolution configuration and detection
2. Display output mode configuration (fix headless issue)
3. Font selection for readability at target PPI
4. Terminal emulator with cursor, newlines, scrolling
5. ANSI color support (basic)
6. Integration with kernel println! (optional GPU output)

## Planning Location

`docs/planning/gpu-terminal/`

## Progress Log

- [x] Phase 1: Discovery — Complete
- [x] Phase 2: Design — Complete
- [x] Phase 3: Implementation — Complete ✅
- [x] Phase 4: Integration & Testing — Awaiting USER visual verification
- [ ] Phase 5: Polish & Handoff

## Implementation Complete

**Files created/modified**:
- `kernel/src/terminal.rs` — NEW (283 lines)
- `kernel/src/gpu.rs` — Added `get_resolution()`, `flush()`
- `kernel/src/main.rs` — Added `mod terminal`, terminal integration
- `run.sh` — Changed `-display none` → `-display gtk`, added resolution
- `run-pixel6.sh` — Changed `-display none` → `-display gtk`, added resolution

**Build status**: ✅ Compiles
**Tests status**: ✅ All 34 pass

---

## HANDOFF TO NEXT TEAM — CRITICAL BUGS REMAINING

### Bug 1: Newline Does Not Work Correctly (CRITICAL)

**Symptom**: After pressing Enter, the next character typed overlaps the previous line instead of appearing on the new line.

**UART Log shows**:
```
[TERM] newline at col=24, row=0 -> col=0, row=1
[TERM] write_char('=') at col=0, row=1, pixel=(0, 42)  # This looks correct...
```

But visually, text overlaps on same row.

**Suspected Issue**: The `newline()` function sets `cursor_row += 1` but there may be:
1. A race condition with the lock
2. The variable not being persisted correctly
3. Display coordinates not matching cursor state

**File to investigate**: `kernel/src/terminal.rs` lines 165-180 (`newline` function)

**How to reproduce**:
1. Run `./run.sh`
2. Type some characters in the terminal (stdio)
3. Press Enter
4. Type more characters
5. Observe: new characters overlap previous line

### Bug 2: Mouse Cursor Erases Text (MINOR)

**Symptom**: When mouse cursor moves over text, it erases it (draws black over the text).

**Cause**: `cursor.rs` erases previous position with BLACK, which overwrites any text underneath.

**Fix needed**: Save and restore the pixels under the cursor, OR use XOR drawing, OR disable cursor tracking over text areas.

**File**: `kernel/src/cursor.rs`

---

## Required Reading for Next Team

1. **Planning docs**: `docs/planning/gpu-terminal/phase-1.md` through `phase-4.md`
2. **Success criteria**: `phase-1.md` lines 180-289 (granular checkpoints)
3. **Terminal implementation**: `kernel/src/terminal.rs`
4. **Cursor implementation**: `kernel/src/cursor.rs`
5. **Main integration**: `kernel/src/main.rs` lines 478-514

## What Works ✅

- SC1.1-SC1.6: QEMU display configuration
- SC2.1-SC2.5: Resolution detection (1280x800)
- SC3.1-SC3.7: Terminal module creation
- SC4.1-SC4.6: Font rendering (FONT_10X20)
- SC10.1-SC10.5: Clear screen
- SC14.1: Boot banner displays
- SC14.2: Dimensions printed to UART
- SC14.3: UART input echoes
- Cursor trails mostly fixed (but erases text)

## What Does NOT Work ❌

- SC6.1-SC6.5: Newline handling (BUG - overlaps)
- SC8.1-SC8.5: Line wrapping (untested due to newline bug)
- SC11.1-SC11.8: Scrolling (untested due to newline bug)
- Mouse cursor erases text underneath

## Verification Commands

```bash
# Build kernel
cargo build --release --target aarch64-unknown-none -p levitate-kernel

# Run QEMU with GTK display
./run.sh

# Type in the TERMINAL (stdio), not QEMU window
# Watch UART output for [TERM] debug logs
```

## Target Hardware

| Platform | Resolution | PPI | Font Recommendation |
|----------|------------|-----|---------------------|
| QEMU Dev | 1280×800 | ~96 | FONT_10X20 |
| Pixel 6 Oriole | 2400×1080 | 411 | FONT_10X20 or larger |
