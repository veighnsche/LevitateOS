# Phase 2: Design - Procfs and Sysfs Implementation

## Proposed Solution

### User-Facing Behavior

After implementation:
```bash
# Mount procfs
mount -t proc proc /proc

# View process list
ls /proc/
# Output: 1  2  3  self  meminfo  uptime  stat

# View current process
cat /proc/self/status
# Output:
# Name:   ash
# Pid:    3
# PPid:   1
# State:  R (running)
# VmSize: 1234 kB
# ...

# View memory info
cat /proc/meminfo
# Output:
# MemTotal:       524288 kB
# MemFree:        400000 kB
# ...

# View file descriptors
ls -la /proc/self/fd/
# Output:
# 0 -> /dev/console
# 1 -> /dev/console
# 2 -> /dev/console
```

### System Behavior

1. **Mount**: `mount -t proc proc /proc` creates a `ProcfsSuperblock` instance
2. **Readdir `/proc/`**: Enumerate PIDs from process table + static entries
3. **Lookup `/proc/123/stat`**: Create synthetic inode for pid=123, file="stat"
4. **Read**: Generate content on-demand from kernel state
5. **Close**: No cleanup needed (no persistent state)

## API Design

### New Filesystem Types

```rust
// vfs/src/mount.rs
pub enum FsType {
    Tmpfs,
    Initramfs,
    Fat32,
    Ext4,
    Procfs,    // NEW
    Sysfs,     // NEW (stub for now)
}
```

### Procfs Superblock

```rust
// fs/procfs/src/lib.rs
pub struct ProcfsSuperblock {
    /// Next inode number for synthetic inodes
    next_ino: AtomicU64,
    /// Cached VFS root inode
    vfs_root: Mutex<Option<Arc<Inode>>>,
}

impl Superblock for ProcfsSuperblock {
    fn root(&self) -> Result<Arc<Inode>, VfsError>;
    fn statfs(&self) -> Result<StatFs, VfsError>;
    fn fs_type(&self) -> FsType { FsType::Procfs }
}
```

### Procfs Inode Private Data

```rust
/// What this inode represents
pub enum ProcfsEntry {
    /// Root /proc/ directory
    Root,
    /// /proc/[pid]/ directory
    ProcessDir { pid: u32 },
    /// /proc/[pid]/stat file
    ProcessStat { pid: u32 },
    /// /proc/[pid]/status file
    ProcessStatus { pid: u32 },
    /// /proc/[pid]/cmdline file
    ProcessCmdline { pid: u32 },
    /// /proc/[pid]/maps file
    ProcessMaps { pid: u32 },
    /// /proc/[pid]/exe symlink
    ProcessExe { pid: u32 },
    /// /proc/[pid]/cwd symlink
    ProcessCwd { pid: u32 },
    /// /proc/[pid]/fd/ directory
    ProcessFdDir { pid: u32 },
    /// /proc/[pid]/fd/[fd] symlink
    ProcessFd { pid: u32, fd: u32 },
    /// /proc/self symlink
    SelfLink,
    /// /proc/meminfo file
    Meminfo,
    /// /proc/uptime file
    Uptime,
    /// /proc/stat file (system stats)
    SystemStat,
}
```

### InodeOps for Procfs

```rust
// Directory operations (for /proc/, /proc/[pid]/, /proc/[pid]/fd/)
pub struct ProcfsDirOps;
impl InodeOps for ProcfsDirOps {
    fn lookup(&self, inode: &Inode, name: &str) -> Result<Arc<Inode>, VfsError>;
    fn readdir(&self, inode: &Inode, offset: usize) -> Result<Vec<DirEntry>, VfsError>;
}

// File operations (for stat, status, maps, meminfo, etc.)
pub struct ProcfsFileOps;
impl InodeOps for ProcfsFileOps {
    fn read(&self, inode: &Inode, buf: &mut [u8], offset: u64) -> Result<usize, VfsError>;
}

// Symlink operations (for exe, cwd, fd/[n], self)
pub struct ProcfsSymlinkOps;
impl InodeOps for ProcfsSymlinkOps {
    fn readlink(&self, inode: &Inode) -> Result<String, VfsError>;
}
```

### Error Handling

| Condition | Error |
|-----------|-------|
| Process exited (pid not found) | `VfsError::NotFound` (ENOENT) |
| Invalid procfs path | `VfsError::NotFound` (ENOENT) |
| Permission denied | `VfsError::PermissionDenied` (EPERM) |
| Write to read-only file | `VfsError::PermissionDenied` (EPERM) |

## Data Model

### No Persistent State
Procfs has no persistent data - all content is generated from:
- `los_sched::process::PROCESS_TABLE`
- `TaskControlBlock` fields
- `los_mm` memory statistics
- `los_hal::timer::ticks()` for uptime

### Inode Number Encoding

To allow fast PID lookup from inode number:
```rust
// Encoding scheme:
// Bits 63-32: Entry type (0=root, 1=process, 2=meminfo, etc.)
// Bits 31-16: PID (for process entries)
// Bits 15-0:  Sub-entry (stat=0, status=1, maps=2, etc.)

fn encode_ino(entry: &ProcfsEntry) -> u64 {
    match entry {
        ProcfsEntry::Root => 1,
        ProcfsEntry::ProcessDir { pid } => 0x1_0000_0000 | (*pid as u64) << 16,
        ProcfsEntry::ProcessStat { pid } => 0x1_0000_0000 | (*pid as u64) << 16 | 1,
        // etc.
    }
}
```

## Behavioral Decisions

### Q1: What happens when a process exits while /proc/[pid]/ is open?

**Decision**: Return `ENOENT` on subsequent operations.

The inode remains valid (held by open file), but reads return empty or error because the process no longer exists in the process table.

### Q2: What format should /proc/[pid]/stat use?

**Decision**: Linux-compatible format (space-separated, specific field order).

```
pid (comm) state ppid pgrp session tty_nr tpgid flags minflt cminflt majflt cmajflt utime stime cutime cstime priority nice num_threads itrealvalue starttime vsize rss ...
```

We'll implement a subset of fields, using 0 for unimplemented ones.

### Q3: What format should /proc/[pid]/maps use?

**Decision**: Linux-compatible format:
```
address           perms offset  dev   inode   pathname
08048000-08056000 r-xp 00000000 00:00 0       [heap]
```

### Q4: Should /proc entries be cached?

**Decision**: No caching. Always generate fresh content.

Reasoning:
- Procfs content changes frequently
- Caching adds complexity
- Memory overhead not worth it for small reads
- Kernel state is the source of truth

### Q5: How to handle /proc/self?

**Decision**: `/proc/self` is a symlink that returns the current task's PID.

```rust
fn readlink(&self, inode: &Inode) -> Result<String, VfsError> {
    let pid = los_sched::current_task().id.0;
    Ok(format!("{}", pid))
}
```

### Q6: What permissions should procfs entries have?

**Decision**:
- Directories: 0555 (r-xr-xr-x) - readable by all
- Files: 0444 (r--r--r--) - readable by all
- Symlinks: 0777 (lrwxrwxrwx) - symlinks don't have meaningful permissions

Note: Full permission checks would require per-process credentials. For now, everything is readable.

### Q7: Should we implement /proc/[pid]/fd/ directory?

**Decision**: Yes, this is commonly used.

- `/proc/self/fd/` lets programs enumerate their open files
- `/proc/self/fd/[n]` symlinks to the actual file path
- Used by shells for `<(...)` process substitution

### Q8: What about sysfs?

**Decision**: Implement a minimal stub that mounts successfully but has minimal content.

- `/sys/` directory exists
- `/sys/class/` exists (empty)
- `/sys/devices/` exists (empty)

Full sysfs can be implemented later when device enumeration is needed.

## Design Alternatives Considered

### Alternative A: Single Ops Table with Type Dispatch
One `ProcfsOps` struct that checks `ProcfsEntry` type in each method.

**Rejected**: Violates single-responsibility principle, large match statements.

### Alternative B: Pre-populated Directory Tree
Create all inodes at mount time, refresh on readdir.

**Rejected**: Process table can be large, wastes memory for unaccessed entries.

### Alternative C: Filesystem-Level Caching
Cache generated content with TTL.

**Rejected**: Adds complexity, stale data risk, minimal performance benefit.

### Chosen: Lazy Inode Creation with Type-Specific Ops

- Inodes created on first access (lookup)
- Different `InodeOps` for dirs, files, symlinks
- `ProcfsEntry` enum stored in inode private data
- Content generated fresh each read

## Open Questions

### Q9: How to get process executable path?
The `TaskControlBlock` doesn't currently store the executable path. Options:
- Add `exe_path: String` field to TCB (set by execve)
- Return `/proc/[pid]/exe -> [unknown]` for now
- **Recommended**: Add field to TCB, set in execve

### Q10: How to get process command line?
The `TaskControlBlock` doesn't store original argv. Options:
- Add `cmdline: String` field to TCB (set by execve)
- Parse ELF auxiliary vector
- Return empty string for now
- **Recommended**: Add field to TCB, set in execve

### Q11: Should /proc/[pid]/fd/[n] show the actual file path?
Current VFS doesn't track file paths after open. Options:
- Store path in `File` struct
- Walk dentry cache to reconstruct path
- Return generic `socket:[...]` / `pipe:[...]` / `[unknown]`
- **Recommended**: Return `[unknown]` initially, improve later

### Q12: Thread visibility in /proc?
Linux shows threads in `/proc/[tid]/` and `/proc/[pid]/task/[tid]/`. Options:
- Show only main process PIDs
- Show all threads as separate entries
- **Recommended**: Show only main PIDs (simpler)

### Q13: What memory stats should /proc/meminfo show?
- `MemTotal`: Total physical RAM (from HAL)
- `MemFree`: Free pages * page size
- `Buffers`: 0 (no buffer cache)
- `Cached`: 0 (no page cache)
- **Recommended**: Implement MemTotal and MemFree from frame allocator

### Q14: What should /proc/uptime contain?
Format: `uptime_seconds idle_seconds`
- Uptime: `los_hal::timer::ticks() / ticks_per_second`
- Idle: 0 (we don't track idle time)
- **Recommended**: Implement uptime, return 0 for idle
