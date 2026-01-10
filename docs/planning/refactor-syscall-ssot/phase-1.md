# Phase 1 — Discovery and Safeguards

**Refactor:** Syscall SSOT Consolidation  
**Team:** TEAM_418  
**Date:** 2026-01-10

---

## Refactor Summary

Consolidate duplicated type definitions and constants across the kernel syscall subsystem into a Single Source of Truth (SSOT). Currently, types like `Timeval`, `Timespec`, `Rusage`, and constants like `CLONE_*` flags are defined multiple times in different files.

### Pain Points
1. `Timeval` defined 3 times (resources.rs, time.rs, process.rs)
2. `Timespec` duplicated in both arch modules (identical!)
3. `Stat` struct duplicated in both arch modules (identical!)
4. Clone flags duplicated between `process.rs` and `process/mod.rs`
5. TTY/ioctl constants duplicated across architectures
6. `PATH_MAX` used as magic number `4096` in 6+ locations
7. Dead code: `syscall/process.rs` (37KB) is unused - directory takes precedence

### Motivation
- Reduce maintenance burden
- Prevent divergence between duplicated definitions
- Follow Rule 7 (Modular Refactoring) - each module owns its state
- Remove 37KB of dead code

---

## Success Criteria

### Before
```
syscall/
├── process.rs          # 37KB DEAD CODE (duplicates process/)
├── process/
│   ├── mod.rs          # CLONE_* flags
│   ├── resources.rs    # Timeval, Rusage
│   └── thread.rs       # imports from parent
├── time.rs             # local Timeval definition
└── ...

arch/
├── aarch64/mod.rs      # Timespec, Stat, TTY constants
└── x86_64/mod.rs       # Timespec, Stat, TTY constants (identical!)
```

### After
```
syscall/
├── types.rs            # NEW: Timeval, Timespec (SSOT)
├── constants.rs        # NEW: PATH_MAX, CLONE_*, RLIMIT_* (SSOT)
├── stat.rs             # NEW: Stat struct (SSOT, re-exported by arch)
├── process/            # uses syscall::types, syscall::constants
└── time.rs             # uses syscall::types
└── ...

fs/tty/
├── constants.rs        # NEW: TCGETS, TIOCGWINSZ, etc. (SSOT)
└── ...

(process.rs DELETED)
```

---

## Behavioral Contracts

### Public APIs That Must Remain Stable
1. `syscall::process::{Rusage, Timeval}` - re-exported types
2. `syscall::process::{CLONE_VM, CLONE_FS, ...}` - clone flag constants
3. `crate::arch::{Stat, Timespec}` - arch-specific types (keep re-exports)
4. `fs::tty::{TCGETS, TCSETS, ...}` - TTY constants

### ABI Compatibility
- `Timeval`, `Timespec`, `Stat`, `Rusage` are `#[repr(C)]` - layout must not change
- Syscall numbers are architecture-specific - NOT changing

---

## Golden/Regression Tests

### Tests That Must Pass
1. `cargo build` - kernel must compile for both x86_64 and aarch64
2. `cargo test` - all unit tests
3. Eyra behavior tests - syscall ABI unchanged
4. Golden logs (silver mode) - behavior unchanged

### Baseline Command
```bash
cargo xtask build
# Run eyra tests to verify syscall behavior unchanged
```

---

## Current Architecture Notes

### Dependency Graph
```
syscall/mod.rs
├── pub mod process;      → process/mod.rs (directory wins over .rs)
├── pub mod time;         → time.rs (has local Timeval)
└── pub use arch::{Stat, Timespec}  → arch re-exports

process/mod.rs
├── pub use resources::{Rusage, Timeval}
└── pub const CLONE_* 

arch/aarch64/mod.rs & arch/x86_64/mod.rs
├── pub struct Stat       (IDENTICAL)
├── pub struct Timespec   (IDENTICAL)
└── pub const TCGETS...   (IDENTICAL)
```

### Known Couplings
- `syscall/time.rs:sys_gettimeofday` uses local `Timeval` - must import from SSOT
- `process/thread.rs` imports `CLONE_*` from `super::` - import path changes
- `fs/tty/mod.rs` imports TTY constants from `crate::arch` - import path changes

---

## Constraints

1. **ABI unchanged** - `#[repr(C)]` struct layouts must be identical
2. **No behavioral changes** - pure refactor, same functionality
3. **Arch modules still re-export** - for backward compatibility during transition

---

## Open Questions

None - analysis in TEAM_418 was comprehensive.

---

## Phase 1 Steps

### Step 1 — Verify Current State
- [ ] Confirm `process.rs` is truly dead code
- [ ] List all imports of duplicated types
- [ ] Run baseline tests

### Step 2 — Lock in Baseline
- [ ] Run `cargo build` for both architectures
- [ ] Run eyra behavior tests
- [ ] Document baseline results

---

## Exit Criteria for Phase 1
- [ ] All baseline tests pass
- [ ] Import analysis complete
- [ ] No open questions remain
