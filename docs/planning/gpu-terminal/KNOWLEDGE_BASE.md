# GPU Terminal Verification & Knowledge Base

**TEAM_059: Documentation for Future Teams**

## 1. Verification Technique: UART vs. GPU Sync

When debugging terminal behavior, always compare UART logs with visual GPU output.

### The "False Positive" Gotcha
If UART logs show `[TERM] newline...` but the GPU visually stays on the same line:
1. **Check Character Mapping**: UART consoles (like QEMU's default) often send `\r` (Carriage Return, 0x0D) instead of `\n`.
2. **Implementation Pattern**: Always map `\r` to the terminal emulator's `newline` logic in the main input loop if you want "Enter" to advance rows.
3. **Hardware Flush**: State changes that don't draw pixels (like `cursor_row += 1`) might not trigger a hardware update in some VirtIO implementations. Explicitly call `gpu.flush()` after cursor movements.

## 2. Graphics Pattern: Cursor Save/Restore

To prevent the mouse cursor from "erasing" text or UI elements:
- **Avoid**: Simply drawing the background color over the previous cursor position.
- **Pattern**: 
    1. Save the pixels at the *new* position into a buffer *before* drawing the cursor.
    2. Restore the pixels from the *previous* position when the cursor moves.
- **Implementation**: See `kernel/src/cursor.rs` for the `saved_pixels` implementation.

## 3. Coordinate System Baseline

- **Font Baseline**: In `embedded-graphics`, text positioning usually refers to the **baseline** (bottom of the character).
- **Calculation**: For a font of height `H` at row `R`, the baseline Y coordinate is `(R * (H + Spacing)) + H`.

## 4. Resource Locking

- **GPU Access**: The `GPU` is protected by a `Spinlock`. Since `Display::draw_iter` and `cursor::draw` both need this lock, ensure you never hold one and call the other, or you will deadlock.
- **Pattern**: Perform calculations outside the lock, then acquire the lock only for the final framebuffer write/flush.
