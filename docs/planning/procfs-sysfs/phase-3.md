# Phase 3: Implementation - Procfs and Sysfs

## Implementation Overview

### Crate Structure

```
crates/kernel/fs/
├── procfs/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # ProcfsSuperblock, create_superblock()
│       ├── entries.rs       # ProcfsEntry enum
│       ├── dir_ops.rs       # ProcfsDirOps (lookup, readdir)
│       ├── file_ops.rs      # ProcfsFileOps (read)
│       ├── symlink_ops.rs   # ProcfsSymlinkOps (readlink)
│       └── generators/
│           ├── mod.rs
│           ├── process.rs   # stat, status, cmdline, maps
│           ├── system.rs    # meminfo, uptime, stat
│           └── fd.rs        # fd directory listing
└── sysfs/
    ├── Cargo.toml
    └── src/
        └── lib.rs           # SysfsSuperblock (minimal stub)
```

### Implementation Order

1. **VFS Extension** (30 min)
   - Add `FsType::Procfs`, `FsType::Sysfs` to enum
   - Add string mappings in `TryFrom`
   - Wire up in mount syscall

2. **Procfs Crate Setup** (20 min)
   - Create `Cargo.toml` with dependencies
   - Create `ProcfsSuperblock` skeleton
   - Implement `create_superblock()`

3. **Root Directory** (30 min)
   - Implement `/proc/` readdir (PIDs + static entries)
   - Implement `/proc/` lookup

4. **Process Directory** (45 min)
   - Implement `/proc/[pid]/` readdir
   - Implement `/proc/[pid]/` lookup
   - Return ENOENT for non-existent PIDs

5. **Process Files** (60 min)
   - `/proc/[pid]/stat` - process statistics
   - `/proc/[pid]/status` - human-readable status
   - `/proc/[pid]/cmdline` - command line (empty for now)
   - `/proc/[pid]/maps` - memory maps

6. **Process Symlinks** (30 min)
   - `/proc/[pid]/exe` - executable symlink
   - `/proc/[pid]/cwd` - working directory symlink
   - `/proc/self` - self symlink

7. **FD Directory** (45 min)
   - `/proc/[pid]/fd/` readdir
   - `/proc/[pid]/fd/[n]` symlinks

8. **System Files** (30 min)
   - `/proc/meminfo`
   - `/proc/uptime`
   - `/proc/stat` (minimal)

9. **Sysfs Stub** (15 min)
   - Mount successfully
   - Empty `/sys/class/` and `/sys/devices/`

## Design Reference

See `phase-2.md` for:
- API contracts
- Error handling rules
- Behavioral decisions (Q1-Q14)
- Data formats

## Dependencies

### Crate Dependencies

```toml
# fs/procfs/Cargo.toml
[dependencies]
los_vfs = { path = "../../vfs" }
los_sched = { path = "../../sched" }
los_mm = { path = "../../../mm" }
los_hal = { path = "../../../hal" }
linux-raw-sys = { version = "0.9", default-features = false, features = ["general", "errno"] }
alloc = { package = "alloc" }
log = "0.4"
```

### Kernel State Access

| Data Needed | Source | Access Pattern |
|-------------|--------|----------------|
| Process list | `los_sched::process::iter_all()` | Lock, iterate, release |
| Task info | `los_sched::current_task()` | Get Arc, read fields |
| Memory stats | `los_mm::frame_allocator()` | Query free/total pages |
| Uptime | `los_hal::timer::ticks()` | Direct call |
| VMAs | `task.vmas.lock()` | Lock, iterate, release |
| FDs | `task.fd_table.lock()` | Lock, iterate, release |

## File Templates

### lib.rs (Superblock)

```rust
//! TEAM_469: Procfs implementation

extern crate alloc;

use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};
use los_vfs::{Inode, Superblock, VfsError, StatFs};
use los_vfs::mount::FsType;

mod entries;
mod dir_ops;
mod file_ops;
mod symlink_ops;
mod generators;

pub use entries::ProcfsEntry;

pub struct ProcfsSuperblock {
    next_ino: AtomicU64,
    vfs_root: IrqSafeLock<Option<Arc<Inode>>>,
}

impl ProcfsSuperblock {
    pub fn new() -> Self {
        Self {
            next_ino: AtomicU64::new(2), // 1 is root
            vfs_root: IrqSafeLock::new(None),
        }
    }

    pub fn alloc_ino(&self) -> u64 {
        self.next_ino.fetch_add(1, Ordering::Relaxed)
    }
}

impl Superblock for ProcfsSuperblock {
    fn root(&self) -> Result<Arc<Inode>, VfsError> {
        // Create root inode lazily
        todo!()
    }

    fn statfs(&self) -> Result<StatFs, VfsError> {
        Ok(StatFs {
            f_type: 0x9fa0, // PROC_SUPER_MAGIC
            f_bsize: 4096,
            f_blocks: 0,
            f_bfree: 0,
            f_bavail: 0,
            f_files: 0,
            f_ffree: 0,
            f_namelen: 255,
        })
    }

    fn fs_type(&self) -> FsType {
        FsType::Procfs
    }
}

pub fn create_superblock() -> Arc<dyn Superblock + Send + Sync> {
    Arc::new(ProcfsSuperblock::new())
}
```

### entries.rs (Entry Types)

```rust
//! TEAM_469: Procfs entry type definitions

/// What a procfs inode represents
#[derive(Clone, Debug)]
pub enum ProcfsEntry {
    Root,
    ProcessDir { pid: u32 },
    ProcessStat { pid: u32 },
    ProcessStatus { pid: u32 },
    ProcessCmdline { pid: u32 },
    ProcessMaps { pid: u32 },
    ProcessExe { pid: u32 },
    ProcessCwd { pid: u32 },
    ProcessFdDir { pid: u32 },
    ProcessFd { pid: u32, fd: u32 },
    SelfLink,
    Meminfo,
    Uptime,
    SystemStat,
}

impl ProcfsEntry {
    /// Get the file mode for this entry type
    pub fn mode(&self) -> u32 {
        use linux_raw_sys::general::{S_IFDIR, S_IFREG, S_IFLNK};
        match self {
            Self::Root | Self::ProcessDir { .. } | Self::ProcessFdDir { .. } => {
                S_IFDIR | 0o555
            }
            Self::ProcessStat { .. }
            | Self::ProcessStatus { .. }
            | Self::ProcessCmdline { .. }
            | Self::ProcessMaps { .. }
            | Self::Meminfo
            | Self::Uptime
            | Self::SystemStat => S_IFREG | 0o444,
            Self::ProcessExe { .. }
            | Self::ProcessCwd { .. }
            | Self::ProcessFd { .. }
            | Self::SelfLink => S_IFLNK | 0o777,
        }
    }
}
```

## Unit of Work Breakdown

| # | Task | Est. Time | Files |
|---|------|-----------|-------|
| 1 | Add FsType variants | 10 min | vfs/src/mount.rs |
| 2 | Wire up mount syscall | 10 min | syscall/src/fs/mount.rs |
| 3 | Create procfs crate | 20 min | fs/procfs/Cargo.toml, lib.rs |
| 4 | Implement ProcfsEntry | 15 min | fs/procfs/src/entries.rs |
| 5 | Implement root dir ops | 30 min | fs/procfs/src/dir_ops.rs |
| 6 | Implement file ops | 30 min | fs/procfs/src/file_ops.rs |
| 7 | Implement symlink ops | 20 min | fs/procfs/src/symlink_ops.rs |
| 8 | Process stat generator | 30 min | fs/procfs/src/generators/process.rs |
| 9 | System info generators | 20 min | fs/procfs/src/generators/system.rs |
| 10 | FD directory generator | 30 min | fs/procfs/src/generators/fd.rs |
| 11 | Create sysfs stub | 15 min | fs/sysfs/ |
| 12 | Integration testing | 30 min | - |

**Total Estimated Time: ~4 hours**
