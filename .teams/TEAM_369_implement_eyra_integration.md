# TEAM_369 — Implement Eyra Integration (Phase 4)

**Date:** 2026-01-10  
**Status:** ✅ COMPLETED

---

## Purpose

Implement phase-4 of the Eyra integration plan:
1. Add `--with-eyra` flag to `cargo xtask build all`
2. Modify `create_initramfs()` to copy Eyra binaries
3. Eyra binaries replace bare-metal versions (per user decision)

---

## Plan Reference

- `docs/planning/eyra-uutils-linker-fix/phase-4.md`

---

## Progress Log

### Entry 1 — Starting Implementation
- Read phase-4 plan
- Verified test baseline
- Beginning implementation

### Entry 2 — Implementation Complete
- Added `--with-eyra` flag to `BuildCommands::All`
- Updated `build_all()` to accept `with_eyra: bool` parameter
- Updated `create_initramfs()` to accept `with_eyra: bool` and copy Eyra binaries
- Updated all 12+ call sites across xtask codebase
- Verified build compiles and tests pass

---

## Files Modified

- `xtask/src/build/commands.rs` — Added --with-eyra flag, updated build_all() and create_initramfs()
- `xtask/src/main.rs` — Updated command handler to pass with_eyra flag
- `xtask/src/run.rs` — Updated build_all() calls (4 locations)
- `xtask/src/vm/exec.rs` — Updated build_all() call
- `xtask/src/vm/session.rs` — Updated build_all() call
- `xtask/src/tests/backspace.rs` — Updated build_all() call
- `xtask/src/tests/screenshot.rs` — Updated build_all() calls (2 locations)
- `xtask/src/tests/screenshot_levitate.rs` — Updated build_all() calls
- `xtask/src/tests/keyboard_input.rs` — Updated build_all() call

---

## Usage

```bash
# Build with Eyra coreutils (replaces bare-metal)
cargo xtask build all --with-eyra

# Build without Eyra (default, bare-metal only)
cargo xtask build all
```

---

## Handoff Checklist

- [x] --with-eyra flag added
- [x] build_all() calls build_eyra() when flag set
- [x] create_initramfs() copies Eyra binaries
- [x] Verified build compiles
- [x] Tests pass
- [x] Team file updated

---

## Next Steps (Phase 5)

Phase 5 requires testing Eyra binaries on the actual kernel:
1. Boot LevitateOS with `cargo xtask run --with-eyra` (needs flag propagation to run)
2. Test basic execution (true, false, echo, pwd)
3. Test file operations (cat, ls, touch, mkdir)

