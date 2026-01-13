# Phase 1: Discovery - Procfs and Sysfs Implementation

## Feature Summary

### Problem Statement
LevitateOS currently fails to mount `/proc` and `/sys` filesystems with "Invalid argument" (EINVAL). These pseudo-filesystems expose kernel state to userspace and are required by many Unix programs:

- **procfs** (`/proc`): Process information, system statistics, kernel parameters
- **sysfs** (`/sys`): Device/driver hierarchy, hardware information

Without these filesystems, tools like `ps`, `top`, `htop`, `free`, and many shell scripts fail.

### Who Benefits
- Users running standard Unix utilities (ps, top, free, uptime)
- Shell scripts that read `/proc/self/fd` or `/proc/$$/`
- Programs that query system information
- Developers debugging process state

### Success Criteria

#### Acceptance Tests
1. `mount -t proc proc /proc` succeeds
2. `mount -t sysfs sysfs /sys` succeeds
3. `ls /proc/` shows PID directories and system files
4. `cat /proc/self/status` shows current process info
5. `cat /proc/meminfo` shows memory statistics
6. `ls /proc/self/fd/` shows open file descriptors
7. `readlink /proc/self/exe` returns executable path
8. Basic sysfs structure exists at `/sys/`

#### Non-Goals (Phase 1)
- Full Linux procfs compatibility (thousands of files)
- Writable proc entries (`/proc/sys/*`)
- Full sysfs device tree
- cgroups or namespaces

## Current State Analysis

### Why Mount Fails
The `FsType` enum in `vfs/src/mount.rs` only supports:
- `Tmpfs`
- `Initramfs`
- `Fat32`
- `Ext4`

When `mount("proc", ...)` is called, `FsType::try_from("proc")` returns `Err(UnsupportedFsType)`, which maps to `EINVAL`.

### Existing Pseudo-Filesystem Patterns

**Devtmpfs** (`crates/kernel/fs/devtmpfs/`) provides a model:
- Pre-populated directory structure at creation
- Devices have `InodeOps` that handle read/write specially
- Device dispatch via `(major, minor)` numbers
- Static content (device list doesn't change)

**Tmpfs** (`crates/kernel/fs/tmpfs/`) provides another model:
- Dynamic directory creation
- File content stored in `TmpfsNode::data: Vec<u8>`
- Standard file I/O semantics

### Gap: Dynamic Content Generation

Neither existing filesystem supports **content generated at read time**. Procfs needs:
- `/proc/uptime` returns different value each read
- `/proc/meminfo` reflects current memory state
- `/proc/[pid]/stat` changes as process runs
- Directory listings change as processes start/exit

## Codebase Reconnaissance

### Modules Affected

| Module | Changes Required |
|--------|------------------|
| `vfs/src/mount.rs` | Add `FsType::Procfs`, `FsType::Sysfs` |
| `vfs/src/ops.rs` | Possibly extend `InodeOps` for dynamic content |
| `fs/` | New `procfs/` and `sysfs/` crates |
| `syscall/src/fs/mount.rs` | Wire up new filesystem types |

### APIs to Use

| API | Purpose |
|-----|---------|
| `los_sched::process::iter_all()` | List all processes |
| `los_sched::process::get_process(pid)` | Get task by PID |
| `los_sched::current_task()` | Get current task |
| `TaskControlBlock.vmas` | Process memory maps |
| `TaskControlBlock.fd_table` | Open file descriptors |
| `TaskControlBlock.cwd` | Current working directory |

### Tests Affected
- Behavior tests (golden boot log will change with mount success)
- New unit tests needed for procfs content generation

## Constraints

### Technical Constraints
1. **No `unwrap()` or `panic!()`** - All errors must be `Result`
2. **Thread-safe** - Multiple readers may access proc files concurrently
3. **No heap allocation in read path** (where possible) - Generate into provided buffer
4. **Linux ABI compatibility** - Format must match what programs expect

### Resource Constraints
1. Procfs inodes are synthetic - don't count against file limits
2. No persistent storage - all content generated on-demand
3. Memory usage should be minimal (no caching generated content)

### Compatibility Constraints
| File | Format Requirements |
|------|---------------------|
| `/proc/[pid]/stat` | Space-separated fields, specific order |
| `/proc/[pid]/status` | Key: Value lines |
| `/proc/[pid]/maps` | Specific hex format for addresses |
| `/proc/meminfo` | "MemTotal: %lu kB" format |

## Open Questions (Phase 1)

1. **Q1**: Should we start with procfs only, or implement both procfs and sysfs together?
   - Procfs is more critical for Unix compatibility
   - Sysfs mainly needed for device enumeration

2. **Q2**: What's the minimum viable set of `/proc` entries?
   - `/proc/self`, `/proc/[pid]/` directories
   - `stat`, `status`, `cmdline`, `maps`, `exe`, `cwd`, `fd/`
   - System-wide: `meminfo`, `uptime`, `stat`
