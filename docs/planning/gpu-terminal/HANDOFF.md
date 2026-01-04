# GPU Terminal Handoff Document

**TEAM_058** → **Next Debugging Team**
**Date**: 2026-01-04
**Status**: Partial Implementation — Critical Bugs Remaining

---

## Executive Summary

GPU Terminal feature is **partially working**. Text renders on screen, but newlines are broken. The next team needs to debug and fix the newline issue before other features (scrolling, line wrap) can be tested.

---

## Quick Start for Next Team

```bash
# 1. Read the team file
cat .teams/TEAM_058_gpu_terminal.md

# 2. Build the kernel
cargo build --release --target aarch64-unknown-none -p levitate-kernel

# 3. Run QEMU
./run.sh

# 4. Type in the TERMINAL window (not QEMU window) to test
# Watch [TERM] logs in UART output
```

---

## Architecture Overview

```
main.rs
   │
   ├── gpu::get_resolution() → (1280, 800)
   │
   ├── terminal::Terminal::new(width, height)
   │       │
   │       ├── cols = 1280/10 = 128
   │       └── rows = 800/22 = 36
   │
   ├── term.clear(&mut display)
   │
   ├── term.write_str(&mut display, "banner...")
   │
   └── loop {
           input::poll() → cursor::draw()
           console::read_byte() → term.write_char()
       }
```

---

## Critical Bug: Newline Not Working

### Symptoms
1. Press Enter → UART shows `[TERM] newline at col=X, row=Y -> col=0, row=Y+1`
2. Type character → UART shows `[TERM] write_char(...) at col=0, row=Y+1`
3. BUT visually, character appears on SAME line as before

### Theory 1: Terminal State Not Persisted
The `Terminal` struct is a local variable in `main()`. After `newline()` returns, the state should be preserved, but something may be wrong.

### Theory 2: Y Coordinate Calculation
```rust
let y = (self.cursor_row * (FONT_HEIGHT + LINE_SPACING)) as i32;
```
Maybe `cursor_row` is being used before it's incremented?

### Theory 3: Display Lock Contention
The `write_char` function and `newline` function both access state, but `Terminal` is not behind a lock — it's a local mutable variable. This should be fine, but worth checking.

### Debug Steps for Next Team
1. Add more logging in `write_char`:
   ```rust
   crate::println!("[TERM] write_char PRE: cursor_row={}", self.cursor_row);
   ```
2. Check if `cursor_row` actually changes after newline
3. Verify the pixel Y calculation matches expected row

---

## Files to Read

| Priority | File | What It Contains |
|----------|------|------------------|
| 1 | `kernel/src/terminal.rs` | Terminal implementation, newline bug is HERE |
| 2 | `kernel/src/main.rs:478-514` | Integration code |
| 3 | `kernel/src/cursor.rs` | Mouse cursor (erases text bug) |
| 4 | `kernel/src/gpu.rs` | GPU state, framebuffer access |
| 5 | `docs/planning/gpu-terminal/phase-1.md:180-289` | Success criteria checklist |

---

## What Works ✅

| Checkpoint | Description |
|------------|-------------|
| SC1.1-SC1.6 | QEMU display mode GTK, resolution 1280x800 |
| SC2.1-SC2.5 | Resolution detection from virtio-gpu |
| SC3.1-SC3.7 | Terminal module compiles, struct defined |
| SC4.1-SC4.6 | FONT_10X20 renders text correctly |
| SC10.1-SC10.5 | Clear screen works |
| SC14.1 | Boot banner "LevitateOS Terminal v0.1" displays |
| SC14.2 | Dimensions printed to UART |
| SC14.3 | UART input echoes to GPU |

---

## What Does NOT Work ❌

| Checkpoint | Description | Bug |
|------------|-------------|-----|
| SC6.1-SC6.5 | Newline handling | Characters overlap after Enter |
| SC8.1-SC8.5 | Line wrapping | Blocked by newline bug |
| SC11.1-SC11.8 | Scrolling | Blocked by newline bug |
| SC14.6 | Cursor tracking | Erases text underneath |

---

## Behavior Inventory Update Needed

After fixing bugs, update `docs/testing/behavior-inventory.md`:
- Change TERM1-TERM9 from ⚠️ to ✅ as they're verified

---

## ROADMAP Status

Phase 6 Task 6.2 (GPU Refinement) is **IN PROGRESS**:
- VirtIO Net: ✅ Complete (TEAM_057)
- GPU Terminal: 🚧 Partial (TEAM_058)
- 9P Filesystem: ⏸️ Deferred

---

## Contact Points

- Team file: `.teams/TEAM_058_gpu_terminal.md`
- Planning: `docs/planning/gpu-terminal/`
- Behavior testing rules: `.windsurf/rules/behavior-testing.md`
