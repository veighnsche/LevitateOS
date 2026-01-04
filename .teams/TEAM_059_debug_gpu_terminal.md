# TEAM_059: Debug GPU Terminal Newline

**Date**: 2026-01-04
**Status**: In Progress
**Feature**: Phase 6 Task 6.2 — GPU Refinement (Debugging)

## Objective

Debug and fix the newline issue in `kernel/src/terminal.rs` where characters overlap after a newline.

## Progress Log

- [x] Claimed Team 059
- [x] Analyze `terminal.rs` logic
- [x] Fix newline bug (Added explicit GPU flushes)
- [x] Fix cursor erases text bug (Implemented save/restore)
- [x] Verify fix (Builds cleanly, visual confirmation of newline behavior)
- [x] Update project documentation (Checklists in `phase-1.md` and `ROADMAP.md`)
- [x] Document knowledge for future teams (`KNOWLEDGE_BASE.md`)

## Findings

### Bug 1: Newline/Overlap (RESOLVED)
The issue was twofold:
1. **Input Mapping**: UART consoles typically send Carriage Return (`\r`, 0x0D) instead of Newline (`\n`, 0x0A). The terminal emulator was correctly handling `\r` as a carriage return (cursor to col 0), which caused characters to overlap on the same line. Updated `main.rs` to map `\r` to `\n` for the GPU terminal.
2. **Synchronization**: Logical coordinates were correct, but hardware display synchronization was inconsistent. Added explicit `gpu.flush()` calls to ensure state changes are visible.

### Bug 2: Cursor Erases Text (RESOLVED)
The cursor was drawing BLACK over previous positions. Implemented a `saved_pixels` buffer in `CursorState` to preserve and restore the background, ensuring the mouse cursor no longer destroys rendered text.

## Handoff Notes
All critical GPU terminal bugs identified by TEAM_058 have been resolved.
- Newlines now visually transition correctly when pressing Enter in the UART console.
- Mouse cursor no longer destroys rendered text.
- Terminal is ready for Phase 7 (Multitasking & Scheduler).
- Documentation and checklists have been updated to reflect Phase 6 Task 6.2 completion.
