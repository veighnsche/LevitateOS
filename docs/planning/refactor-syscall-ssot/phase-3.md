# Phase 3 — Migration

**Refactor:** Syscall SSOT Consolidation  
**Team:** TEAM_418  
**Date:** 2026-01-10

---

## Migration Strategy

### Approach: Breaking Changes Over Fragile Compatibility (Rule 5)

1. **Update imports directly** - no shims or adapters
2. **Let the compiler find issues** - if imports break, fix them
3. **Remove temporary re-exports** once all call sites migrated

### Order of Migration
1. Internal syscall modules (time.rs, process/, etc.)
2. Arch modules (re-export removal)
3. FS modules (tty)
4. Any external consumers

---

## Call Site Inventory

### Types to Migrate

| Type | Old Import | New Import | Call Sites |
|------|------------|------------|------------|
| `Timeval` | `process::Timeval`, local in time.rs | `syscall::types::Timeval` | time.rs, process/resources.rs |
| `Timespec` | `crate::arch::Timespec` | `syscall::types::Timespec` | time.rs, syscall/mod.rs |
| `Stat` | `crate::arch::Stat` | `syscall::stat::Stat` (via arch re-export) | Multiple FS syscalls |

### Constants to Migrate

| Constant | Old Location | New Location | Call Sites |
|----------|--------------|--------------|------------|
| `CLONE_*` | `process/mod.rs` | `syscall::constants` | process/thread.rs |
| `PATH_MAX` | Magic `4096` | `syscall::constants::PATH_MAX` | 6+ files |
| `RLIMIT_*` | local in resources.rs | `syscall::constants` | resources.rs |
| `TCGETS`, etc. | `crate::arch::*` | `fs::tty::constants` | fs/tty/mod.rs |

---

## Phase 3 Steps

### Step 1 — Migrate Time Types

**UoW:** Single session task

**Tasks:**
1. Update `syscall/time.rs`:
   - Remove local `Timeval` definition from `sys_gettimeofday`
   - Import from `crate::syscall::types::Timeval`

2. Update `syscall/process/resources.rs`:
   - Remove `Timeval` definition
   - Import from `crate::syscall::types::Timeval`
   - Update `Rusage` to use imported `Timeval`

3. Update re-exports in `syscall/process/mod.rs`:
   - Change `pub use resources::Timeval` to `pub use crate::syscall::types::Timeval`

4. Update arch modules:
   - Remove `Timespec` definition
   - Re-export from `crate::syscall::types::Timespec`

**Exit Criteria:**
- [ ] No duplicate `Timeval` or `Timespec` definitions remain
- [ ] `cargo build` succeeds
- [ ] All tests pass

---

### Step 2 — Migrate Clone Flags and PATH_MAX

**UoW:** Single session task

**Tasks:**
1. Update `syscall/process/mod.rs`:
   - Remove `CLONE_*` constant definitions
   - Re-export from `crate::syscall::constants`

2. Update `syscall/process/thread.rs`:
   - Change `use super::{CLONE_*}` to `use crate::syscall::constants::{CLONE_*}`

3. Replace magic `4096` with `PATH_MAX`:
   - `syscall/fs/open.rs:17`
   - `syscall/helpers.rs:342, 394`
   - `syscall/fs/dir.rs:135, 162, 200`
   - `syscall/fs/fd.rs:368`

4. Update `syscall/process/resources.rs`:
   - Remove local `RLIMIT_*` constants
   - Import from `crate::syscall::constants`

**Exit Criteria:**
- [ ] No duplicate clone flags
- [ ] No magic `4096` for path buffers
- [ ] `cargo build` succeeds

---

### Step 3 — Migrate TTY Constants

**UoW:** Single session task

**Tasks:**
1. Update `fs/tty/mod.rs`:
   - Remove imports from `crate::arch`
   - Import from local `constants` module

2. Update arch modules:
   - Remove TTY constant definitions
   - Optionally re-export from `fs::tty::constants` for any external use

**Exit Criteria:**
- [ ] TTY constants have single definition
- [ ] `cargo build` succeeds

---

### Step 4 — Migrate Stat Struct

**UoW:** Single session task (may be complex)

**Tasks:**
1. Update arch modules to re-export from `syscall::stat`:
   ```rust
   // In arch/aarch64/mod.rs
   pub use crate::syscall::stat::Stat;
   ```

2. Verify all Stat usage still works:
   - FS syscalls (fstat, stat, etc.)
   - VFS layer
   - initramfs, tmpfs

**Exit Criteria:**
- [ ] Single `Stat` definition in `syscall/stat.rs`
- [ ] Arch modules re-export for backward compatibility
- [ ] All FS tests pass

---

### Step 5 — Remove Temporary Re-exports

**UoW:** Single session task

Once all call sites are migrated, remove re-exports that are no longer needed:

1. If arch modules only re-export (no arch-specific additions), consider removing re-exports
2. Update `syscall/mod.rs` to export from SSOT directly

**Exit Criteria:**
- [ ] Minimal re-export chain
- [ ] Clean import paths

---

## Rollback Plan

If migration causes issues:
1. Revert the specific step
2. Re-add re-exports to maintain old import paths
3. Investigate root cause before retrying

---

## Exit Criteria for Phase 3

- [ ] All types imported from SSOT locations
- [ ] No duplicate definitions remain (except intentional re-exports)
- [ ] All tests pass
- [ ] `cargo build` succeeds for both architectures
