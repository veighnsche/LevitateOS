# Phase 2: Structural Extraction

## Target Design

### New Module Layout

```
crates/kernel/src/syscall/
├── mod.rs                    # Dispatcher (existing)
├── process/
│   ├── mod.rs               # Re-exports, ~50 lines
│   ├── lifecycle.rs         # exit, spawn, exec, waitpid (~200 lines)
│   ├── identity.rs          # uid/gid/tid syscalls (~80 lines)
│   ├── groups.rs            # pgid/sid syscalls (~100 lines)
│   ├── thread.rs            # clone, set_tid_address (~150 lines)
│   ├── resources.rs         # getrusage, prlimit64 (~180 lines)
│   └── arch_prctl.rs        # x86_64-specific (~120 lines)
├── helpers.rs               # Existing helpers (unchanged)
└── ...                      # Other syscall modules
```

### Shared Types Location

Move structs to avoid circular dependencies:

```
crates/kernel/src/syscall/
├── types.rs                 # NEW: Utsname, Rusage, Timeval, Rlimit64, UserArgvEntry
└── process/...
```

Or keep structs local to their usage if not shared.

## Module Breakdown

### 1. `process/mod.rs` (~50 lines)
- Module declarations
- Re-exports of all public syscall functions
- Clone flag constants (CLONE_VM, etc.)

```rust
//! Process management syscalls.
//! TEAM_417: Refactored from monolithic process.rs

mod lifecycle;
mod identity;
mod groups;
mod thread;
mod resources;
mod arch_prctl;

// Re-export all syscall functions
pub use lifecycle::*;
pub use identity::*;
pub use groups::*;
pub use thread::*;
pub use resources::*;
pub use arch_prctl::*;

// Clone flags (Linux ABI)
pub const CLONE_VM: u64 = 0x00000100;
// ... etc
```

### 2. `process/lifecycle.rs` (~200 lines)
**Contents:**
- Helper functions: resolve_initramfs_executable, clone_fd_table_for_child, register_spawned_process, write_exit_status
- sys_exit, sys_getpid, sys_getppid, sys_yield
- sys_spawn, sys_exec, sys_spawn_args
- sys_waitpid
- sys_set_foreground, sys_get_foreground
- UserArgvEntry struct, MAX_ARGC, MAX_ARG_LEN constants

### 3. `process/identity.rs` (~80 lines)
**Contents:**
- sys_getuid, sys_geteuid
- sys_getgid, sys_getegid
- sys_gettid
- sys_exit_group (delegates to sys_exit)
- sys_uname, sys_umask
- Utsname struct, str_to_array helper

### 4. `process/groups.rs` (~100 lines)
**Contents:**
- sys_setpgid, sys_getpgid, sys_getpgrp
- sys_setsid

### 5. `process/thread.rs` (~150 lines)
**Contents:**
- sys_clone
- sys_set_tid_address

### 6. `process/resources.rs` (~180 lines)
**Contents:**
- Rusage struct
- Timeval struct
- sys_getrusage
- Rlimit64 struct (local)
- Resource limit constants (RLIMIT_*)
- sys_prlimit64

### 7. `process/arch_prctl.rs` (~120 lines)
**Contents:**
- arch_prctl_codes module
- sys_arch_prctl (x86_64 version)
- sys_arch_prctl (non-x86_64 stub)

## Extraction Strategy

### Order of Extraction

1. **arch_prctl.rs** - Most isolated, x86_64-specific only
2. **identity.rs** - Simple, no dependencies on other process syscalls
3. **groups.rs** - Simple process group handling
4. **resources.rs** - Self-contained resource limit handling
5. **thread.rs** - Depends on clone constants but otherwise isolated
6. **lifecycle.rs** - Most complex, depends on helpers

### Coexistence Strategy

**Rule 5: Breaking Changes > Fragile Compatibility**

- Do NOT create compatibility shims
- Extract one module at a time
- Update `syscall/mod.rs` dispatch table after each extraction
- Let compiler errors guide call site updates

### Step-by-Step Extraction

For each module:

1. Create new file `process/<module>.rs`
2. Move relevant functions and types
3. Add appropriate `use` statements
4. Update `process/mod.rs` with module declaration and re-export
5. Update `syscall/mod.rs` if dispatch changes are needed
6. Verify build for both architectures
7. Commit before next module

## Rule 7 Compliance Check

| Module | Estimated Lines | Status |
|--------|----------------|--------|
| mod.rs | ~50 | OK |
| lifecycle.rs | ~200 | OK |
| identity.rs | ~80 | OK |
| groups.rs | ~100 | OK |
| thread.rs | ~150 | OK |
| resources.rs | ~180 | OK |
| arch_prctl.rs | ~120 | OK |
| **Total** | ~880 | Split from 1090 |

All modules well under 500-line ideal.
