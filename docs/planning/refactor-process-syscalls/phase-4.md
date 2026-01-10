# Phase 4: Cleanup

## Rule 6: Remove All Dead Code

### Items to Verify and Remove

After extraction, check for:

1. **Unused imports in old process.rs** - File will be deleted entirely
2. **Unused imports in new modules** - Each module should only import what it uses
3. **Duplicate constants** - Ensure no duplication of CLONE_*, RLIMIT_*, etc.
4. **Orphaned helper functions** - All helpers should have callers

### Dead Code Candidates

From static analysis of current file:

1. `MAX_SYMLINK_DEPTH` - Used only by `resolve_initramfs_executable`
2. `UserArgvEntry` - Used only by `sys_spawn_args`
3. `MAX_ARGC`, `MAX_ARG_LEN` - Used only by `sys_spawn_args`
4. `ECHILD` - Used only by `sys_waitpid`

All are actually used - keep them in their respective modules.

## Remove Temporary Adapters

**Rule 5 Compliance**: No compatibility shims were created.

If any were accidentally created:
- Search for `#[deprecated]` annotations
- Search for "temporary", "compat", "shim" in comments
- Remove any wrapper functions that just forward calls

## Tighten Encapsulation

### Make Private What Can Be Private

| Item | Current | Target | Rationale |
|------|---------|--------|-----------|
| resolve_initramfs_executable | pub(crate) | fn | Only used in lifecycle.rs |
| clone_fd_table_for_child | fn | fn | Only used in lifecycle.rs |
| register_spawned_process | fn | fn | Only used in lifecycle.rs |
| write_exit_status | fn | fn | Only used in lifecycle.rs |
| UserArgvEntry | pub(crate) | struct (private) | Only used in lifecycle.rs |
| str_to_array | fn | fn | Only used in identity.rs |
| Rlimit64 | struct | struct (private) | Only used in resources.rs |
| arch_prctl_codes | pub mod | pub(crate) mod | Only used by arch_prctl |

### Module Visibility

```rust
// process/mod.rs
mod lifecycle;      // private
mod identity;       // private
mod groups;         // private
mod thread;         // private
mod resources;      // private
mod arch_prctl;     // private

// Public API via re-exports only
pub use lifecycle::{sys_exit, sys_spawn, ...};
pub use identity::{sys_getuid, ...};
// etc.
```

## File Size Check

### Target Sizes (after refactor)

| File | Lines | Status |
|------|-------|--------|
| process/mod.rs | ~50 | OK (<500) |
| process/lifecycle.rs | ~200 | OK (<500) |
| process/identity.rs | ~80 | OK (<500) |
| process/groups.rs | ~100 | OK (<500) |
| process/thread.rs | ~150 | OK (<500) |
| process/resources.rs | ~180 | OK (<500) |
| process/arch_prctl.rs | ~120 | OK (<500) |

### Verification Command

```bash
wc -l crates/kernel/src/syscall/process/*.rs
```

Expected total: ~880 lines (down from 1090, better organized)

## Final Cleanup Checklist

- [ ] Delete `crates/kernel/src/syscall/process.rs` (old monolithic file)
- [ ] Run `cargo clippy` to find unused code warnings
- [ ] Remove any `#[allow(dead_code)]` that's no longer needed
- [ ] Verify no `pub` items that should be private
- [ ] Verify all `use` statements are necessary
- [ ] Run `cargo fmt` on all new files
- [ ] Both architectures build cleanly
