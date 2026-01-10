# Phase 3: Migration

## Migration Order

Execute in this order to minimize conflicts:

| Step | Module | Risk | Dependencies |
|------|--------|------|--------------|
| 1 | arch_prctl.rs | Low | None |
| 2 | identity.rs | Low | None |
| 3 | groups.rs | Low | None |
| 4 | resources.rs | Low | helpers.rs |
| 5 | thread.rs | Medium | Clone constants |
| 6 | lifecycle.rs | Medium | helpers, FD table |
| 7 | mod.rs cleanup | Low | All modules done |

## Call Site Inventory

### Primary Dispatcher
**File**: `crates/kernel/src/syscall/mod.rs`

The syscall dispatcher uses the syscall functions. After refactor:
- Current: `use process::*;` or explicit `process::sys_*`
- After: Same pattern works via re-exports from `process/mod.rs`

### Internal Cross-References

Within `process.rs`:
- `sys_exit_group` calls `sys_exit` - Keep in same module (identity.rs)
- `sys_getpgrp` calls `sys_getpgid` - Keep in same module (groups.rs)

### External Dependencies

These modules use types/functions from process.rs:

1. **syscall/mod.rs** - Dispatches to all process syscalls
2. **No other direct callers identified** - All access through syscall dispatch

## Detailed Migration Steps

### Step 1: Create process/ directory and mod.rs

```bash
mkdir -p crates/kernel/src/syscall/process
```

Create `process/mod.rs` with:
- Module declarations
- Clone flag constants
- Re-exports

### Step 2: Extract arch_prctl.rs

Move lines 612-734:
- `arch_prctl_codes` module
- `sys_arch_prctl` (both x86_64 and stub versions)

### Step 3: Extract identity.rs

Move lines 559-610 plus lines 830-933:
- `sys_getuid`, `sys_geteuid`, `sys_getgid`, `sys_getegid`
- `sys_gettid`, `sys_exit_group`
- `Utsname` struct, `str_to_array`, `sys_uname`, `sys_umask`

### Step 4: Extract groups.rs

Move lines 736-828:
- `sys_setpgid`, `sys_getpgid`, `sys_getpgrp`, `sys_setsid`

### Step 5: Extract resources.rs

Move lines 935-1090:
- `Rusage`, `Timeval` structs
- `Rlimit64` struct (can stay local)
- Resource limit constants
- `sys_getrusage`, `sys_prlimit64`

### Step 6: Extract thread.rs

Move lines 422-557:
- `sys_clone`
- `sys_set_tid_address`

### Step 7: Extract lifecycle.rs

Move remaining lines 1-420:
- Helper functions
- `sys_exit`, `sys_getpid`, `sys_getppid`, `sys_yield`
- `sys_spawn`, `sys_exec`, `sys_spawn_args`
- `sys_waitpid`
- `sys_set_foreground`, `sys_get_foreground`
- `UserArgvEntry`, constants

### Step 8: Update syscall/mod.rs

Change:
```rust
pub mod process;
```
to:
```rust
pub mod process;
```
(Same, since process/mod.rs re-exports everything)

### Step 9: Delete old process.rs

Remove `crates/kernel/src/syscall/process.rs` after verifying build.

## Rollback Plan

### Before Starting
- Commit current state with message: "checkpoint: before process.rs refactor"

### During Migration
- Each extracted module gets its own commit
- If build fails mid-extraction, revert to last good commit

### If Refactor Fails
```bash
git checkout HEAD~N -- crates/kernel/src/syscall/process.rs
git checkout HEAD~N -- crates/kernel/src/syscall/mod.rs
rm -rf crates/kernel/src/syscall/process/
```

### Verification at Each Step
```bash
cargo xtask build kernel --arch aarch64
cargo xtask build kernel --arch x86_64
```

Both must pass before proceeding to next step.
