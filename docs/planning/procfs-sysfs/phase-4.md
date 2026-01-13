# Phase 4: Integration - Procfs and Sysfs

## Integration Points

### 1. VFS Mount System

**File**: `crates/kernel/vfs/src/mount.rs`

```rust
// Add to FsType enum
pub enum FsType {
    // existing...
    Procfs,
    Sysfs,
}

// Add to TryFrom impl
impl TryFrom<&str> for FsType {
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            // existing...
            "proc" => Ok(FsType::Procfs),
            "sysfs" => Ok(FsType::Sysfs),
            _ => Err(MountError::UnsupportedFsType),
        }
    }
}
```

### 2. Mount Syscall Dispatcher

**File**: `crates/kernel/syscall/src/fs/mount.rs`

```rust
// Add to superblock creation
let sb: Arc<dyn Superblock + Send + Sync> = match fstype {
    FsType::Tmpfs => los_fs_tmpfs::create_superblock(),
    FsType::Procfs => los_fs_procfs::create_superblock(),
    FsType::Sysfs => los_fs_sysfs::create_superblock(),
    // ...
};
```

### 3. Kernel Crate Dependencies

**File**: `crates/kernel/levitate/Cargo.toml`

```toml
[dependencies]
los_fs_procfs = { path = "../fs/procfs" }
los_fs_sysfs = { path = "../fs/sysfs" }
```

### 4. Scheduler Exports

**File**: `crates/kernel/sched/src/lib.rs`

Ensure these are public:
- `process::iter_all()` - iterate all processes
- `process::get_process(pid)` - get single process
- `TaskControlBlock` fields accessible

### 5. Memory Manager Exports

**File**: `crates/kernel/mm/src/lib.rs`

Need to expose:
- Frame allocator stats (total pages, free pages)
- Page size constant

## Test Strategy

### Unit Tests

```rust
// fs/procfs/src/lib.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_entry_mode() {
        assert_eq!(ProcfsEntry::Root.mode() & 0o777, 0o555);
        assert_eq!(ProcfsEntry::ProcessStat { pid: 1 }.mode() & 0o777, 0o444);
    }

    #[test]
    fn test_stat_format() {
        let stat = generators::process::format_stat(1, "init", 'S', 0);
        assert!(stat.starts_with("1 (init) S 0"));
    }
}
```

### Behavior Tests

**Golden log updates** expected for:
- Successful proc mount (removes "Invalid argument" error)
- Successful sysfs mount (removes "Invalid argument" error)

### Manual Integration Tests

```bash
# After boot, in shell:

# Test mount
mount -t proc proc /proc
echo $?  # Should be 0

# Test root listing
ls /proc/
# Should show: 1 2 self meminfo uptime

# Test self symlink
readlink /proc/self
# Should show current PID

# Test process status
cat /proc/self/status
# Should show process info

# Test meminfo
cat /proc/meminfo
# Should show memory stats

# Test process not found
cat /proc/99999/stat
# Should error with "No such file or directory"
```

### Edge Case Tests

| Test | Expected Result |
|------|-----------------|
| Mount proc twice at same point | EBUSY |
| Read /proc/1/stat for init | Valid stat output |
| Read /proc/[exited]/stat | ENOENT |
| List /proc/self/fd/ | Shows 0, 1, 2 (stdin/out/err) |
| Readlink /proc/self/exe | Returns executable path or [unknown] |

## Impact Analysis

### Affected Subsystems

| Subsystem | Impact | Risk |
|-----------|--------|------|
| VFS | Add 2 filesystem types | Low - additive change |
| Mount syscall | Wire up new types | Low - pattern exists |
| Scheduler | Export process iteration | Low - read-only access |
| Memory | Export stats | Low - read-only access |

### Performance Impact

- **Mount**: One-time cost, negligible
- **Readdir /proc/**: O(n) process table iteration
- **Read files**: O(1) for fixed files, O(n) for maps/fd
- **No caching**: Each read regenerates content

### Memory Impact

- **Per-mount**: ~100 bytes for superblock
- **Per-open**: ~200 bytes for inode + file struct
- **No content caching**: Zero persistent memory

### Breaking Changes

None expected. This is purely additive functionality.

## Rollback Plan

If issues arise:
1. Remove `los_fs_procfs` dependency from levitate
2. Remove `FsType::Procfs` handling (mount returns EINVAL again)
3. Revert to previous behavior

Procfs is isolated - no other subsystem depends on it.
