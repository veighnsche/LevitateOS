# Phase 2 — Structural Extraction

**Refactor:** Syscall SSOT Consolidation  
**Team:** TEAM_418  
**Date:** 2026-01-10

---

## Target Design

### New Module Layout
```
crates/kernel/src/syscall/
├── types.rs              # NEW - Time types (Timeval, Timespec)
├── constants.rs          # NEW - PATH_MAX, clone flags  
├── stat.rs               # NEW - Stat struct (shared across archs)
├── mod.rs                # Updated imports/re-exports
└── ...

crates/kernel/src/fs/tty/
├── constants.rs          # NEW - TTY/ioctl constants
├── mod.rs                # Updated imports
└── ...
```

### Responsibility Assignment

| New File | Contains | Moved From |
|----------|----------|------------|
| `syscall/types.rs` | `Timeval`, `Timespec` | arch/*/mod.rs, process/resources.rs, time.rs |
| `syscall/constants.rs` | `PATH_MAX`, `CLONE_*`, `RLIMIT_*` | process/mod.rs, scattered magic numbers |
| `syscall/stat.rs` | `Stat` struct + constructors | arch/*/mod.rs |
| `fs/tty/constants.rs` | `TCGETS`, `TIOCGWINSZ`, etc. | arch/*/mod.rs |

---

## Extraction Strategy

### Order of Extraction
1. **First:** `syscall/types.rs` - time types (smallest, lowest risk)
2. **Second:** `syscall/constants.rs` - clone flags and PATH_MAX
3. **Third:** `fs/tty/constants.rs` - TTY constants
4. **Fourth:** `syscall/stat.rs` - Stat struct (most complex)

### Coexistence Strategy
During extraction, **old locations re-export from new SSOT**:
```rust
// In arch/aarch64/mod.rs (temporary)
pub use crate::syscall::types::Timespec;
pub use crate::syscall::stat::Stat;
```

This allows gradual migration without breaking existing imports.

---

## Modular Refactoring Rules (Rule 7)

- [ ] Each module owns its own state
- [ ] Fields remain private where appropriate
- [ ] No deep relative imports (use `crate::` paths)
- [ ] File sizes < 500 lines (ideal)

---

## Phase 2 Steps

### Step 1 — Create `syscall/types.rs`

**UoW:** Single session task

**Tasks:**
1. Create `syscall/types.rs` with:
   ```rust
   //! Common syscall type definitions (SSOT).
   //! TEAM_418: Consolidated from scattered definitions.
   
   /// Time value with microsecond precision (gettimeofday, rusage).
   #[repr(C)]
   #[derive(Clone, Copy, Default)]
   pub struct Timeval {
       pub tv_sec: i64,
       pub tv_usec: i64,
   }
   
   /// Time value with nanosecond precision (clock_gettime, nanosleep).
   #[repr(C)]
   #[derive(Debug, Clone, Copy, Default)]
   pub struct Timespec {
       pub tv_sec: i64,
       pub tv_nsec: i64,
   }
   ```

2. Add `pub mod types;` to `syscall/mod.rs`
3. Add re-export: `pub use types::{Timeval, Timespec};`

**Exit Criteria:**
- [ ] `cargo build` succeeds
- [ ] New types accessible as `crate::syscall::types::{Timeval, Timespec}`

---

### Step 2 — Create `syscall/constants.rs`

**UoW:** Single session task

**Tasks:**
1. Create `syscall/constants.rs` with:
   ```rust
   //! Common syscall constants (SSOT).
   //! TEAM_418: Consolidated from scattered definitions.
   
   /// Maximum path length (Linux standard).
   pub const PATH_MAX: usize = 4096;
   
   // Clone flags (Linux ABI)
   pub const CLONE_VM: u64 = 0x00000100;
   pub const CLONE_FS: u64 = 0x00000200;
   pub const CLONE_FILES: u64 = 0x00000400;
   pub const CLONE_SIGHAND: u64 = 0x00000800;
   pub const CLONE_THREAD: u64 = 0x00010000;
   pub const CLONE_SETTLS: u64 = 0x00080000;
   pub const CLONE_PARENT_SETTID: u64 = 0x00100000;
   pub const CLONE_CHILD_CLEARTID: u64 = 0x00200000;
   pub const CLONE_CHILD_SETTID: u64 = 0x01000000;
   
   // Resource limits
   pub const RLIMIT_CPU: u32 = 0;
   pub const RLIMIT_FSIZE: u32 = 1;
   pub const RLIMIT_DATA: u32 = 2;
   pub const RLIMIT_STACK: u32 = 3;
   pub const RLIMIT_CORE: u32 = 4;
   pub const RLIMIT_RSS: u32 = 5;
   pub const RLIMIT_NPROC: u32 = 6;
   pub const RLIMIT_NOFILE: u32 = 7;
   pub const RLIMIT_MEMLOCK: u32 = 8;
   pub const RLIMIT_AS: u32 = 9;
   pub const RLIM_INFINITY: u64 = u64::MAX;
   ```

2. Add `pub mod constants;` to `syscall/mod.rs`
3. Re-export clone flags for backward compatibility

**Exit Criteria:**
- [ ] `cargo build` succeeds
- [ ] Constants accessible as `crate::syscall::constants::*`

---

### Step 3 — Create `fs/tty/constants.rs`

**UoW:** Single session task

**Tasks:**
1. Create `fs/tty/constants.rs` with TTY/termios constants:
   ```rust
   //! TTY and termios constants (SSOT).
   //! TEAM_418: Consolidated from arch modules.
   
   // ioctl requests
   pub const TCGETS: u64 = 0x5401;
   pub const TCSETS: u64 = 0x5402;
   pub const TCSETSW: u64 = 0x5403;
   pub const TCSETSF: u64 = 0x5404;
   pub const TIOCGPTN: u64 = 0x80045430;
   pub const TIOCSPTLCK: u64 = 0x40045431;
   pub const TIOCGWINSZ: u64 = 0x5413;
   pub const TIOCSWINSZ: u64 = 0x5414;
   
   // termios flags (c_lflag)
   pub const ISIG: u32 = 0x0001;
   pub const ICANON: u32 = 0x0002;
   // ... (move all from arch modules)
   ```

2. Update `fs/tty/mod.rs` to use local constants

**Exit Criteria:**
- [ ] `cargo build` succeeds
- [ ] TTY module uses local constants

---

### Step 4 — Create `syscall/stat.rs`

**UoW:** Single session task (may split if complex)

**Tasks:**
1. Create `syscall/stat.rs` with:
   - `Stat` struct (identical for both archs)
   - Constructor methods (`new_device`, `new_pipe`, `new_file`, `new_dir`)

2. Keep arch modules re-exporting for now

**Exit Criteria:**
- [ ] `cargo build` succeeds for both architectures
- [ ] Stat constructors work identically

---

## Exit Criteria for Phase 2

- [ ] All 4 new SSOT files created
- [ ] Old locations re-export from new SSOT
- [ ] `cargo build` succeeds for x86_64 and aarch64
- [ ] All tests pass
- [ ] No functional changes - pure structural extraction
