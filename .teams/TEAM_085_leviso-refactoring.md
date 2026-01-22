# TEAM_085: Leviso Codebase Refactoring

**Status:** Complete
**Started:** 2026-01-22
**Completed:** 2026-01-22

## Objective
Refactor leviso to eliminate duplication, clarify module boundaries, and improve maintainability while keeping the subsystem-based organization.

## Issues Being Addressed

| Issue | Severity | Count |
|-------|----------|-------|
| Duplicated `copy_dir_recursive` | HIGH | 3 implementations |
| Confusing naming (`binary.rs` vs `binaries.rs`) | HIGH | 2 files |
| Functions too long (100+ lines) | MEDIUM | 4 functions |
| Dead code (legacy `create_iso`) | MEDIUM | 138 lines |
| Test-only functions in production code | LOW | 5+ functions |
| `setup_dbus()` name collision | MEDIUM | 2 functions |

## Phases

### Phase 1: Consolidate Duplicated Code
- [x] Unify `copy_dir_recursive` (3 implementations → 1 in `common/binary.rs`)
- [x] Extract `create_symlink_if_missing` helper (added to `common/binary.rs`)

### Phase 2: Clarify Module Naming
- [x] Rename `build/binary.rs` → `build/libdeps.rs`
- [x] Rename `build/binaries.rs` → `build/binary_lists.rs`
- [x] Rename `setup_dbus()` → `copy_dbus_configs()` in systemd.rs

### Phase 3: Remove Dead Code
- [x] Delete legacy `create_iso()` function (138 lines removed)
- [SKIPPED] Move test-only functions to `#[cfg(test)]`
  - These functions (`create_var_symlinks`, `create_sh_symlink`, `create_shell_config`, `for_testing`) ARE used by integration tests in `tests/`
  - Integration tests are external crates and cannot access `#[cfg(test)]` items
  - Current approach with `#[allow(dead_code)]` is correct for test helpers

### Phase 4: Break Up Long Functions
- [x] Split `build_system()` into 5 phase helpers
- [x] Split `create_squashfs_iso()` into 5 stage functions + IsoPaths struct

## Verification
- `cargo build --release`: Pass
- `cargo clippy`: No warnings
- `cargo test`: All 46 tests pass

## Log
- 2026-01-22: Team file created, starting Phase 1
- 2026-01-22: All phases completed, verification passed
