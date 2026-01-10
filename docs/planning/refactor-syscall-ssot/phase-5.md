# Phase 5 — Hardening and Handoff

**Refactor:** Syscall SSOT Consolidation  
**Team:** TEAM_418  
**Date:** 2026-01-10

---

## Purpose

Final verification, documentation updates, and clean handoff to future teams.

---

## Verification Checklist

### Build Verification
- [ ] `cargo build` succeeds for x86_64
- [ ] `cargo build` succeeds for aarch64
- [ ] No warnings related to refactored code

### Test Verification
- [ ] All unit tests pass
- [ ] Eyra behavior tests pass
- [ ] Golden logs unchanged (or updated in silver mode)

### ABI Verification
- [ ] `Timeval` layout unchanged (16 bytes: i64 + i64)
- [ ] `Timespec` layout unchanged (16 bytes: i64 + i64)
- [ ] `Stat` layout unchanged (128 bytes)
- [ ] `Rusage` layout unchanged

### Import Path Verification
```rust
// These should all work after refactor:
use crate::syscall::types::{Timeval, Timespec};
use crate::syscall::constants::{CLONE_VM, PATH_MAX};
use crate::syscall::stat::Stat;
use crate::fs::tty::constants::TCGETS;

// Backward compatibility (if kept):
use crate::arch::{Stat, Timespec};  // re-exports
use crate::syscall::process::{Timeval, Rusage};  // re-exports
```

---

## Documentation Updates

### Files to Update

1. **`docs/ARCHITECTURE.md`** (if exists)
   - Add section on SSOT modules
   - Document import conventions

2. **`crates/kernel/src/syscall/mod.rs`**
   - Update module-level doc comments
   - Document re-export strategy

3. **Team file: `.teams/TEAM_418_refactor_ssot_analysis.md`**
   - Mark as complete
   - Add final summary

### New Documentation

Add doc comments to new SSOT modules:

```rust
//! # syscall/types.rs
//! 
//! Single Source of Truth for common syscall type definitions.
//! 
//! ## Types
//! - `Timeval` - Time with microsecond precision (gettimeofday, rusage)
//! - `Timespec` - Time with nanosecond precision (clock_gettime)
//! 
//! ## Usage
//! ```rust
//! use crate::syscall::types::{Timeval, Timespec};
//! ```
```

---

## Phase 5 Steps

### Step 1 — Full Test Suite

**UoW:** Single session task

**Tasks:**
1. Run complete build for both architectures:
   ```bash
   cargo xtask build --arch x86_64
   cargo xtask build --arch aarch64
   ```

2. Run all tests:
   ```bash
   cargo test
   ```

3. Run eyra behavior tests (if available)

4. Verify golden logs (silver mode - update if needed)

**Exit Criteria:**
- [ ] All builds succeed
- [ ] All tests pass

---

### Step 2 — Documentation

**UoW:** Single session task

**Tasks:**
1. Add/update doc comments in new SSOT modules
2. Update team file with completion status
3. Update any architecture docs if needed

**Exit Criteria:**
- [ ] New modules have doc comments
- [ ] Team file marked complete

---

### Step 3 — Final Review and Handoff

**UoW:** Single session task

**Tasks:**
1. Review all changes:
   - New files created
   - Old files deleted
   - Import changes

2. Verify no TODOs left behind

3. Create summary in team file

4. Mark refactor complete

**Exit Criteria:**
- [ ] All phases complete
- [ ] Clean handoff documentation

---

## Handoff Notes Template

```markdown
## TEAM_418 Refactor Complete

### Summary
Consolidated duplicated syscall types and constants into SSOT modules.

### Changes Made
- Created `syscall/types.rs` (Timeval, Timespec)
- Created `syscall/constants.rs` (CLONE_*, PATH_MAX, RLIMIT_*)
- Created `syscall/stat.rs` (Stat struct)
- Created `fs/tty/constants.rs` (TTY/ioctl constants)
- Deleted `syscall/process.rs` (37KB dead code)
- Updated imports across codebase

### Files Affected
- crates/kernel/src/syscall/types.rs (NEW)
- crates/kernel/src/syscall/constants.rs (NEW)
- crates/kernel/src/syscall/stat.rs (NEW)
- crates/kernel/src/fs/tty/constants.rs (NEW)
- crates/kernel/src/syscall/process.rs (DELETED)
- crates/kernel/src/syscall/mod.rs (MODIFIED)
- crates/kernel/src/syscall/time.rs (MODIFIED)
- crates/kernel/src/syscall/process/mod.rs (MODIFIED)
- crates/kernel/src/syscall/process/resources.rs (MODIFIED)
- crates/kernel/src/syscall/process/thread.rs (MODIFIED)
- crates/kernel/src/arch/aarch64/mod.rs (MODIFIED)
- crates/kernel/src/arch/x86_64/mod.rs (MODIFIED)
- crates/kernel/src/fs/tty/mod.rs (MODIFIED)

### Remaining Work
None - refactor complete.

### For Future Teams
- Import time types from `crate::syscall::types`
- Import constants from `crate::syscall::constants`
- Do not add new type definitions to arch modules - use SSOT
```

---

## Exit Criteria for Phase 5

- [ ] All tests pass
- [ ] Documentation updated
- [ ] Team file complete
- [ ] No open TODOs
- [ ] Clean handoff ready

---

## Overall Refactor Exit Criteria

- [ ] Phase 1 complete (Discovery)
- [ ] Phase 2 complete (Extraction)
- [ ] Phase 3 complete (Migration)
- [ ] Phase 4 complete (Cleanup)
- [ ] Phase 5 complete (Hardening)
- [ ] 37KB dead code removed
- [ ] SSOT established for types and constants
- [ ] All tests pass
- [ ] Build succeeds for both architectures
