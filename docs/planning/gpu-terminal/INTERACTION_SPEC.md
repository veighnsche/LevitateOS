# Boot Console Interaction Specification

This document defines the rigorous behavioral contracts for the LevitateOS Boot Console (GPU Terminal). 

## Input Handling (Interactive Session)

### [SPEC-1] Printable Characters
- **Behavior**: When a printable ASCII character (0x20-0x7E) is received:
  - **Action**: Draw the character at the current `cursor_col/row`.
  - **State**: Advance `cursor_col`. If `cursor_col >= cols`, trigger `newline`.
  - **Visual**: The blinking cursor moves to the new position.

### [SPEC-2] Destructive Backspace (ASCII 0x08)
- **Behavior**: When a backspace character is received:
  - **Case: col > 0**:
    - Move `cursor_col` left by 1.
    - Erase the character at the new position by filling the character cell with `bg_color`.
  - **Case: col == 0 and row > 0**:
    - Move `cursor_row` up by 1.
    - Set `cursor_col` to `cols - 1` (wrap back).
    - Erase the character at the new position.
  - **Case: col == 0 and row == 0**:
    - No action.
  - **Visual**: The character at the previous position vanishes; the cursor moves left/up.

### [SPEC-3] Enter / Newline (ASCII 0x0A / 0x0D)
- **Behavior**: When a newline or carriage return is received:
  - **Action**: Move `cursor_col` to 0.
  - **State**: Increment `cursor_row`. 
  - **Scroll**: If `cursor_row >= rows`, scroll screen up and set `cursor_row = rows - 1`.
  - **Visual**: Cursor moves to the start of the next line.

### [SPEC-4] Tab (ASCII 0x09)
- **Behavior**: Advance to the next 8-column boundary.
- **Action**: `cursor_col = (cursor_col / 8 + 1) * 8`.
- **Wrap**: If `new_col >= cols`, trigger `newline`.

## Rendering Invariants

### [SPEC-5] Cursor Non-Destruction
- **Constraint**: The blinking cursor block MUST NOT permanently overwrite character data.
- **Mechanism**:
  1. Before drawing the cursor block, save the underlying pixel data.
  2. When hiding or moving the cursor, restore the saved pixel data.
  3. Character rendering (`write_char`) MUST hide the cursor before drawing and show it after.

### [SPEC-6] Screen Scrolling
- **Mechanism**: When scrolling, the entire framebuffer content is shifted up by `FONT_HEIGHT + LINE_SPACING` pixels. The bottom line is filled with `bg_color`.
