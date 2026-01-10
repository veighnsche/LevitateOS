# TEAM_370 — Proactive Bug Investigation

**Date:** 2026-01-10  
**Status:** ✅ COMPLETED

---

## Purpose

Proactively scan the codebase for bugs, especially in areas recently modified.

---

## Bugs Found & Fixed

### Bug 1: Dead Code — `RunCommands` enum (FIXED)

**Location:** `xtask/src/run.rs:12-50`
**Issue:** ~40 lines of dead code. `RunCommands` enum was defined but never used since main.rs uses `RunArgs` struct instead.
**Fix:** Removed the dead enum.

### Bug 2: Pointless Flag — `--iso` (FIXED)

**Location:** `xtask/src/main.rs` (RunArgs)
**Issue:** The `--iso` flag was completely useless:
- x86_64: Always uses ISO regardless of flag (`use_iso = args.iso || arch == "x86_64"`)
- aarch64: Cannot use ISO (`build_iso()` bails with "ISO build currently only supported for x86_64")
**Fix:** Removed the flag, simplified logic to `use_iso = arch == "x86_64"`.

### Bug 3: Unused Import (FIXED)

**Location:** `xtask/src/run.rs:8`
**Issue:** `use clap::Subcommand` was unused after RunCommands removal.
**Fix:** Removed the import.

---

## Files Modified

- `xtask/src/run.rs` — Removed dead `RunCommands` enum and unused import
- `xtask/src/main.rs` — Removed `--iso` flag, simplified use_iso logic

---

## Verification

- [x] Build compiles successfully
- [x] No new errors introduced

---

## Remaining Warnings (Lower Priority)

The build has 26 warnings, mostly:
- Unused functions in test modules (screenshot_alpine.rs, screenshot_levitate.rs)
- These are likely intentionally unused test helpers

