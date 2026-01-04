# Phase 4: Integration & Testing — GPU Terminal & Display System

**Feature**: GPU Refinement — Extended Scope (Phase 6, Task 6.2)
**Team**: TEAM_058
**Depends on**: `phase-3.md`

---

## Testing Strategy

### Test Categories

| Category | Method | Coverage |
|----------|--------|----------|
| **Unit** | Not applicable (hardware-dependent) | - |
| **Runtime** | Visual verification during boot | TERM1-TERM9 |
| **Behavior** | Golden file comparison | Boot output |

---

## Runtime Verification Tests

### Test 1: Display Visibility

**Steps**:
1. Run `./run.sh`
2. Verify QEMU GTK window appears

**Expected**: Window opens with framebuffer content visible

**Pass Criteria**: Not headless, window renders

---

### Test 2: Resolution Detection

**Steps**:
1. Run with default config (1280×800)
2. Check terminal size calculation

**Expected** (1280×800):
- Cols: 128 (1280 / 10)
- Rows: 36 (800 / 22)

**Expected** (2400×1080):
- Cols: 240 (2400 / 10)
- Rows: 49 (1080 / 22)

**Pass Criteria**: `Terminal: NxM chars` output matches calculation

---

### Test 3: Character Rendering (TERM1)

**Steps**:
1. Boot kernel
2. Type characters via UART

**Expected**: Characters appear on GPU display at cursor position

**Pass Criteria**: Visual verification of typed text

---

### Test 4: Cursor Advancement (TERM2)

**Steps**:
1. Type "ABC"
2. Observe cursor position

**Expected**: Each character appears to the right of previous

**Pass Criteria**: Text flows left-to-right

---

### Test 5: Newline (TERM3)

**Steps**:
1. Type "Hello" then Enter
2. Type "World"

**Expected**:
```
Hello
World
```

**Pass Criteria**: Second line starts at column 0

---

### Test 6: Line Wrap (TERM2 + TERM3)

**Steps**:
1. Type a string longer than screen width

**Expected**: Text wraps to next line automatically

**Pass Criteria**: No characters lost, text continues on next line

---

### Test 7: Scrolling (TERM4)

**Steps**:
1. Fill screen with text (type many lines)
2. Continue typing past bottom

**Expected**: 
- Top line scrolls off screen
- New text appears at bottom
- Cursor stays on last line

**Pass Criteria**: Content scrolls up, not wrap-to-top

---

### Test 8: Clear Screen (TERM7)

**Steps**:
1. Fill screen with text
2. Trigger clear (if exposed via command)

**Expected**: Screen becomes blank, cursor at (0, 0)

**Pass Criteria**: All text removed

---

### Test 9: Tab Character (TERM6)

**Steps**:
1. Type "A\tB" (A, tab, B)

**Expected**: B appears at column 8 (next tab stop)

**Pass Criteria**: Tab alignment correct

---

### Test 10: Multi-Resolution (TERM9)

**Steps**:
1. Run `./run.sh` (1280×800)
2. Verify terminal works
3. Run `./run-pixel6.sh` (2400×1080)
4. Verify terminal works

**Expected**: Both resolutions render correctly

**Pass Criteria**: No hardcoded dimensions causing issues

---

## Behavior Inventory Update

Add to `docs/testing/behavior-inventory.md`:

```markdown
## Group 10: GPU Terminal — Behavior Inventory

TEAM_058: GPU Terminal for Phase 6

### File Groups
- `kernel/src/terminal.rs` (Terminal emulator)

### Terminal (terminal.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| TERM1 | Character renders at cursor position | ⚠️ | Runtime (visual) |
| TERM2 | Cursor advances after character | ⚠️ | Runtime (visual) |
| TERM3 | Newline moves to next line start | ⚠️ | Runtime (visual) |
| TERM4 | Screen scrolls when cursor exceeds rows | ⚠️ | Runtime (visual) |
| TERM5 | Carriage return resets column | ⚠️ | Runtime (visual) |
| TERM6 | Tab advances to 8-column boundary | ⚠️ | Runtime (visual) |
| TERM7 | Clear fills with background color | ⚠️ | Runtime (visual) |
| TERM8 | Backspace moves cursor left | ⚠️ | Runtime (visual) |
| TERM9 | Resolution adapts to screen size | ⚠️ | Runtime (visual) |

### Group 10 Summary
- **Terminal**: 9/9 behaviors documented
- **Runtime verified**: 9/9 ⚠️ (hardware-dependent)
```

---

## Integration Checklist

- [ ] QEMU display mode changed from `none` to `gtk`
- [ ] Resolution configurable via QEMU device params
- [ ] Terminal module compiles without errors
- [ ] Terminal initializes in main.rs
- [ ] Boot banner displays on GPU
- [ ] UART input echoes to GPU terminal
- [ ] Scrolling works when screen fills
- [ ] No regressions in existing functionality
- [ ] Behavior inventory updated
- [ ] All existing tests still pass

---

## Rollback Plan

If issues arise:
1. Revert `run.sh` to `-display none`
2. Remove `mod terminal;` from main.rs
3. Restore original graphics demo code

Terminal module can be kept but unused until issues resolved.

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Display visible | ✅ QEMU window appears |
| Resolution detected | ✅ Matches QEMU config |
| Text renders | ✅ Characters visible |
| Scrolling works | ✅ Proper scroll-up |
| No regressions | ✅ All 34 tests pass |

---

## Next Phase

After successful integration:
- **Phase 5**: Polish, documentation, ROADMAP update
- Mark Task 6.2 complete in ROADMAP
- Update team file with completion status
