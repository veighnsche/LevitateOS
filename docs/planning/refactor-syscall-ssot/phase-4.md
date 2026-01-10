# Phase 4 — Cleanup

**Refactor:** Syscall SSOT Consolidation  
**Team:** TEAM_418  
**Date:** 2026-01-10

---

## Purpose

Remove dead code, temporary adapters, and tighten encapsulation after migration is complete.

---

## Dead Code Removal (Rule 6)

### Critical: Delete `syscall/process.rs`

**File:** `crates/kernel/src/syscall/process.rs` (37KB)

**Status:** Dead code - Rust 2018+ prioritizes `process/mod.rs` over `process.rs`

**Contains duplicates of:**
- `CLONE_*` flags (lines 427-435)
- `Rusage` struct (lines 939-959)
- `Timeval` struct (lines 961-967)
- `sys_clone` function (entire implementation)
- Other syscall implementations already in `process/`

**Action:** Delete entire file

---

## Temporary Adapter Removal

After Phase 3 migration, remove any re-exports that were only kept for backward compatibility:

1. **Arch module re-exports** - If `Timespec` and `Stat` are now imported directly from SSOT, arch re-exports may be unnecessary
2. **process/mod.rs re-exports** - Verify which re-exports are still needed

---

## Encapsulation Tightening

### Review new SSOT modules:

| Module | Review Items |
|--------|--------------|
| `syscall/types.rs` | Fields should be `pub` (ABI requirement) |
| `syscall/constants.rs` | All `pub const` (intentional) |
| `syscall/stat.rs` | Fields `pub` (ABI), constructors `pub` |
| `fs/tty/constants.rs` | All `pub const` (intentional) |

### Remove unnecessary exports from `syscall/mod.rs`:
- Audit `pub use` statements
- Remove anything not used externally

---

## File Size Check (Rule 7)

Target: < 500 lines ideal, < 1000 lines acceptable

| File | Expected Size | Status |
|------|---------------|--------|
| `syscall/types.rs` | ~30 lines | ✓ Small |
| `syscall/constants.rs` | ~50 lines | ✓ Small |
| `syscall/stat.rs` | ~150 lines | ✓ Acceptable |
| `fs/tty/constants.rs` | ~80 lines | ✓ Small |

---

## Phase 4 Steps

### Step 1 — Delete Dead Code

**UoW:** Single session task

**Tasks:**
1. Delete `crates/kernel/src/syscall/process.rs`
2. Verify `cargo build` still works (process/ directory should be used)
3. Run all tests

**Exit Criteria:**
- [ ] `process.rs` deleted
- [ ] Build succeeds
- [ ] Tests pass

---

### Step 2 — Remove Unnecessary Re-exports

**UoW:** Single session task

**Tasks:**
1. Audit arch modules:
   - If nothing uses `crate::arch::Timespec`, remove re-export
   - If nothing uses `crate::arch::Stat` (besides VFS), consider direct import

2. Audit `syscall/mod.rs`:
   - Remove redundant `pub use` statements
   - Keep only externally-used exports

3. Audit `syscall/process/mod.rs`:
   - Verify re-exports match actual usage

**Exit Criteria:**
- [ ] Minimal re-export surface
- [ ] All imports are direct to SSOT where possible

---

### Step 3 — Verify Encapsulation

**UoW:** Single session task

**Tasks:**
1. Check that no internal implementation details are leaked
2. Verify struct fields have appropriate visibility
3. Run `cargo doc` to check public API surface

**Exit Criteria:**
- [ ] Clean public API
- [ ] No unintended exports

---

## Exit Criteria for Phase 4

- [ ] `process.rs` (37KB dead code) deleted
- [ ] No unnecessary re-exports
- [ ] All files under 500 lines
- [ ] Build succeeds for both architectures
- [ ] All tests pass
