# TEAM_193: Opus Review after Iteration 27

**Date:** 2026-02-04
**Status:** Complete

## Scope Reviewed

Last 3 iterations' commits:

- **AcornOS** (3 commits): UKI building (Phase 5), ISO_LABEL constant fix, APK --usermode removal
- **IuppiterOS** (3 commits): sg3_utils binaries, APK --usermode removal, refurbishment tools (hdparm, smartctl, smartd, partx)

## Verification Results

- `cargo check --workspace`: Clean (only pre-existing leviso warnings)
- `cargo test -p acornos --lib`: 34 tests pass
- `cargo test -p iuppiteros`: 22 tests pass (after fix)
- `cargo test -p distro-builder`: 60+ tests pass
- `cargo test -p distro-spec`: 73+ tests pass (after fix)

## Bugs Found and Fixed

### 1. IuppiterOS test asserts sdparm presence, but sdparm not in Alpine v3.23 (FAILING TEST)

**Severity:** Test failure (CI-blocking)

Haiku correctly removed sdparm from `packages.rhai` (iteration 27, commit 620fce9) after discovering it's not available in Alpine v3.23 repos. However, haiku forgot to update the corresponding test in `IuppiterOS/src/lib.rs:121` which still asserted `"sdparm"` must be present.

Additionally, `distro-spec/src/iuppiter/packages.rs` still listed sdparm in `REFURBISHMENT_PACKAGES`. Verified via HTTP that sdparm is absent from both Alpine v3.23 main and community x86_64 repos.

**Fix:**
- Removed sdparm from distro-spec `REFURBISHMENT_PACKAGES` (commit e00ddaa)
- Removed sdparm from IuppiterOS lib.rs test assertion (commit f7b16a5)

## Code Quality Observations (No Action Needed)

1. **AcornOS uki.rs**: Clean implementation. Uses `ISO_LABEL` constant throughout (previous review's hardcoded label fix is in place). Tests verify entries and cmdline format correctly.

2. **IuppiterOS uki.rs**: Clean. Serial-only cmdline (no VGA_CONSOLE), correct distro-spec::iuppiter imports. Good test coverage including serial console verification for all entries.

3. **IuppiterOS packages.rhai try/catch pattern**: Haiku wrapped all `shell(apk_cmd + ...)` calls in try/catch blocks that log warnings instead of failing. This is pragmatic — APK sometimes returns exit code 1 for non-fatal issues (e.g., package already installed). The install() verification function still validates key binaries exist, so actual failures will be caught.

4. **IuppiterOS packages.rhai install() multi-path lookup**: The install function now checks multiple paths for each binary (e.g., `bin/bash` and `usr/bin/bash`). This handles merged-usr vs non-merged layouts correctly.

5. **AcornOS APK --usermode removal**: Correct. The `--usermode` flag doesn't exist in apk-tools-static. The APK database directory initialization that replaced it is the right approach.

6. **No remaining copy-paste issues**: Grep for `distro_spec::acorn` and `AcornOS` in IuppiterOS/src/ shows zero illegitimate references. Previous opus reviews' fixes are holding.

## Files Modified

- `distro-spec/src/iuppiter/packages.rs` — removed sdparm
- `IuppiterOS/src/lib.rs` — removed sdparm from test

## No Blocked Tasks

All PRD tasks through 7.3 are correctly marked [x]. No tasks were marked BLOCKED by haiku.
