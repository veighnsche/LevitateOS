# Phase 3: Migration

## Migration Order

Each step must leave the kernel in a buildable, testable state.

### Step 1: Create `mm/` Crate (Memory Management)

**Duration**: ~2 work sessions

**Files to Move**:
| From | To | Notes |
|------|----|-------|
| `src/memory/mod.rs` | `mm/src/lib.rs` | Re-export public API |
| `src/memory/heap.rs` | `mm/src/heap.rs` | Kernel heap |
| `src/memory/vma.rs` | `mm/src/vma.rs` | Virtual memory areas |
| `src/memory/user.rs` | `mm/src/user.rs` + `cow.rs` + `mapping.rs` | Split 709-line file |

**Call Sites to Update**:
```
src/init.rs              - memory::init_heap()
src/task/mod.rs          - memory::user::*
src/syscall/mm.rs        - memory::user::*, memory::vma::*
src/loader/elf.rs        - memory::user::map_user_page
src/syscall/process/*.rs - memory::user::clone_address_space
```

**Verification**:
```bash
cargo build --target x86_64-unknown-none --release
cargo build --target aarch64-unknown-none --release
cargo xtask run --headless --timeout 10
```

### Step 2: Create `sched/` Crate (Scheduler)

**Duration**: ~2 work sessions

**Files to Move**:
| From | To | Notes |
|------|----|-------|
| `src/task/mod.rs` | `sched/src/task.rs` | Task struct |
| `src/task/process.rs` | `sched/src/process.rs` | Process struct |
| `src/task/thread.rs` | `sched/src/thread.rs` | Thread struct |
| `src/task/scheduler.rs` | `sched/src/scheduler.rs` | Run queue |
| `src/task/fd_table.rs` | `sched/src/fd_table.rs` | FD table |
| `src/task/process_table.rs` | `sched/src/process_table.rs` | Process registry |

**Call Sites to Update**:
```
src/init.rs              - task::spawn_init(), task::current_task()
src/syscall/*.rs         - task::current_task(), task::Task
src/arch/*/task.rs       - task::TaskContext
src/fs/tty/*.rs          - task::current_task()
```

**Critical**: Process/Thread separation
```rust
// Before (coupled)
pub struct Task {
    pub pid: u32,
    pub tid: u32,
    pub mm: Option<Arc<AddressSpace>>,
    pub fd_table: Arc<Mutex<FdTable>>,
    // ... everything
}

// After (separated)
pub struct Process {
    pub pid: Pid,
    pub mm: Arc<AddressSpace>,
    pub fd_table: Arc<Mutex<FdTable>>,
    pub cwd: Arc<Dentry>,
}

pub struct Thread {
    pub tid: Tid,
    pub process: Arc<Process>,
    pub context: Context,
    pub state: ThreadState,
}
```

### Step 3: Create `vfs/` Crate (Virtual Filesystem)

**Duration**: ~2 work sessions

**Files to Move**:
| From | To | Notes |
|------|----|-------|
| `src/fs/vfs/mod.rs` | `vfs/src/lib.rs` | VFS core |
| `src/fs/vfs/inode.rs` | `vfs/src/inode.rs` | Inode |
| `src/fs/vfs/dentry.rs` | `vfs/src/dentry.rs` | Dentry cache |
| `src/fs/vfs/file.rs` | `vfs/src/file.rs` | Open file |
| `src/fs/vfs/dispatch.rs` | `vfs/src/dispatch.rs` | Operation dispatch |
| `src/fs/vfs/ops.rs` | `vfs/src/ops.rs` | File operations trait |
| `src/fs/vfs/error.rs` | `vfs/src/error.rs` | VFS errors |
| `src/fs/path.rs` | `vfs/src/path.rs` + `resolve.rs` | Split path logic |
| `src/fs/mount.rs` | `vfs/src/mount.rs` | Mount table |

**Call Sites to Update**:
```
src/syscall/fs/*.rs      - fs::vfs::* → vfs::*
src/init.rs              - fs::mount_root() → vfs::mount_root()
src/task/process.rs      - fs::vfs::Dentry → vfs::Dentry
```

### Step 4: Create `syscall/` Crate

**Duration**: ~3 work sessions

**Files to Move**:
| From | To | Notes |
|------|----|-------|
| `src/syscall/mod.rs` | `syscall/src/lib.rs` | Dispatcher |
| `src/syscall/fs/*.rs` | `syscall/src/file.rs`, `io.rs`, `dir.rs` | Reorganize by function |
| `src/syscall/process/*.rs` | `syscall/src/process.rs`, `thread.rs` | Merge small files |
| `src/syscall/mm.rs` | `syscall/src/mm.rs` | Memory syscalls |
| `src/syscall/sync.rs` | `syscall/src/sync.rs` | Futex |
| `src/syscall/signal.rs` | `syscall/src/signal.rs` | Signals |

**Reorganization**:
```
syscall/fs/read.rs    \
syscall/fs/write.rs    > syscall/io.rs (read, write, pread, pwrite, readv, writev)
syscall/fs/fd.rs      /

syscall/fs/open.rs    \
syscall/fs/stat.rs     > syscall/file.rs (open, close, stat, fstat, lseek)
syscall/fs/statx.rs   /

syscall/process/*.rs  → syscall/process.rs + syscall/thread.rs
```

### Step 5: Extract `arch/` Crates

**Duration**: ~2 work sessions

**Files to Move**:
| From | To | Notes |
|------|----|-------|
| `src/arch/aarch64/*` | `arch/aarch64/src/*` | Split mod.rs |
| `src/arch/x86_64/*` | `arch/x86_64/src/*` | Split mod.rs |
| `src/arch/mod.rs` | Remove | Replaced by crate imports |

**Platform Trait Implementation**:
```rust
// Each arch crate implements:
impl Platform for Aarch64 {
    type PageTable = Aarch64PageTable;
    type Context = Aarch64Context;

    fn kernel_base() -> usize { 0xFFFF_8000_0000_0000 }
    fn page_size() -> usize { 4096 }
    // ...
}
```

### Step 6: Slim Down `kernel/` Binary

**Duration**: ~1 work session

**Target**: `kernel/src/main.rs` < 100 lines
**Target**: `kernel/src/init.rs` < 200 lines

**Move Out**:
- Driver initialization → `drivers/*/init.rs`
- Device discovery → `drivers/pci/scan.rs`
- Console setup → `drivers/console/init.rs`

## Call Site Inventory

### High-Traffic Symbols

| Symbol | Current Location | New Location | Call Sites |
|--------|------------------|--------------|------------|
| `current_task()` | `task::current_task` | `sched::current` | ~50 |
| `validate_user_buffer` | `memory::user::*` | `mm::validate_user_buffer` | ~30 |
| `Task` struct | `task::Task` | `sched::Thread` | ~40 |
| `vfs_read/write` | `fs::vfs::dispatch` | `vfs::read/write` | ~15 |
| `SyscallResult` | `syscall::SyscallResult` | `syscall::Result` | ~100 |

### Import Pattern Changes

**Before**:
```rust
use crate::task::{current_task, Task};
use crate::memory::user::{validate_user_buffer, copy_to_user};
use crate::fs::vfs::dispatch::{vfs_read, vfs_write};
```

**After**:
```rust
use los_sched::{current, Thread};
use los_mm::{validate_user_buffer, copy_to_user};
use los_vfs::{read, write};
```

## Rollback Plan

Each migration step is atomic. If a step fails:

1. **Git revert**: Each step is one commit (or PR)
   ```bash
   git revert HEAD  # Undo last migration step
   ```

2. **Branch strategy**:
   - `master`: Stable, all tests pass
   - `refactor/mm`: Memory crate migration
   - `refactor/sched`: Scheduler crate migration
   - etc.

3. **Merge only when**:
   - All existing tests pass
   - Both architectures build
   - Boot test succeeds

4. **Abort criteria**:
   - Build broken for > 4 hours
   - Test regressions not fixable in same session
   - Circular dependency discovered

## Migration Checklist Template

For each step:
- [ ] Create new crate with Cargo.toml
- [ ] Move files (git mv to preserve history)
- [ ] Update imports in moved files
- [ ] Update all call sites in other files
- [ ] Delete old location
- [ ] `cargo build --target x86_64-unknown-none`
- [ ] `cargo build --target aarch64-unknown-none`
- [ ] `cargo xtask test unit`
- [ ] `cargo xtask run --headless --timeout 10`
- [ ] Commit with descriptive message
- [ ] Update team file with changes
