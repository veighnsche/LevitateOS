# Phase 12: VFS Foundation (Prerequisites)

**Phase:** Discovery & Implementation  
**Status:** Not Started  
**Team:** TBD

---

## Objective

Build the infrastructure required before implementing the core VFS layer.

---

## Prerequisites Analysis

### ✅ Already Have

| Component | Location | Notes |
|-----------|----------|-------|
| `Arc<T>` | `alloc::sync` | Reference counting |
| `Spinlock<T>` | `los_utils` | Mutual exclusion |
| Error system | `los_error` | `define_kernel_error!` macro |
| Atomic types | `core::sync::atomic` | Lock-free counters |

### 🔴 Need to Build

| Component | Priority | Estimated Effort |
|-----------|----------|------------------|
| RwLock | P0 | 1-2 days |
| Path abstraction | P0 | 1 day |
| Mount table | P0 | 2-3 days |
| Extended Stat | P1 | 1 day |
| File mode constants | P1 | 0.5 day |

---

## Step 1: Implement RwLock

**Why**: Inodes need read-many/write-one access pattern. Spinlock is too coarse.

### Design

```rust
/// Readers-writer lock (writer-preferring)
pub struct RwLock<T> {
    /// State: bits 0-30 = reader count, bit 31 = writer flag
    state: AtomicU32,
    /// Writer waiting flag
    writer_waiting: AtomicBool,
    /// Protected data
    data: UnsafeCell<T>,
}

impl<T> RwLock<T> {
    pub fn read(&self) -> RwLockReadGuard<T>;
    pub fn write(&self) -> RwLockWriteGuard<T>;
    pub fn try_read(&self) -> Option<RwLockReadGuard<T>>;
    pub fn try_write(&self) -> Option<RwLockWriteGuard<T>>;
}
```

### Location
- `crates/utils/src/rwlock.rs` (new file)
- Export from `los_utils`

### Exit Criteria
- [ ] RwLock compiles
- [ ] Multiple readers can hold lock simultaneously
- [ ] Writer blocks readers
- [ ] No deadlocks in basic tests

---

## Step 2: Create Path Abstraction

**Why**: Currently path parsing is duplicated in tmpfs, initramfs, syscalls.

### Design

```rust
/// Normalized filesystem path
pub struct Path {
    /// Path components (no empty strings, no ".")
    components: Vec<String>,
    /// Whether path is absolute (starts with /)
    absolute: bool,
}

impl Path {
    pub fn new(s: &str) -> Self;
    pub fn join(&self, other: &Path) -> Path;
    pub fn parent(&self) -> Option<Path>;
    pub fn file_name(&self) -> Option<&str>;
    pub fn components(&self) -> impl Iterator<Item = &str>;
    pub fn is_absolute(&self) -> bool;
    pub fn starts_with(&self, prefix: &Path) -> bool;
    pub fn strip_prefix(&self, prefix: &Path) -> Option<Path>;
}
```

### Location
- `kernel/src/fs/path.rs` (new file)

### Exit Criteria
- [ ] Path::new parses correctly
- [ ] Handles `.` and `..` resolution
- [ ] Handles double slashes
- [ ] join() works correctly

---

## Step 3: Create Mount Table

**Why**: Currently `/tmp` is hardcoded. Need dynamic mount points.

### Design

```rust
/// A mounted filesystem
pub struct Mount {
    /// Mount point path
    pub mountpoint: Path,
    /// Filesystem superblock
    pub superblock: Arc<dyn Superblock>,
    /// Mount flags (read-only, noexec, etc.)
    pub flags: MountFlags,
}

/// Global mount table
pub struct MountTable {
    mounts: RwLock<Vec<Mount>>,
}

impl MountTable {
    pub fn mount(&self, path: &Path, sb: Arc<dyn Superblock>, flags: MountFlags) -> Result<()>;
    pub fn umount(&self, path: &Path) -> Result<()>;
    pub fn lookup(&self, path: &Path) -> Option<(Arc<dyn Superblock>, Path)>;
}
```

### Location
- `kernel/src/fs/mount.rs` (new file)

### Exit Criteria
- [ ] Can mount a filesystem at a path
- [ ] lookup() finds correct mount for path
- [ ] Longest-prefix matching works
- [ ] umount() removes mount

---

## Step 4: Extend Stat Structure

**Why**: Current Stat is minimal. POSIX requires more fields.

### Current

```rust
pub struct Stat {
    pub st_size: u64,
    pub st_mode: u32,
    pub _pad: u32,
    pub st_atime: u64,
    pub st_mtime: u64,
    pub st_ctime: u64,
}
```

### Target (POSIX-like)

```rust
#[repr(C)]
pub struct Stat {
    pub st_dev: u64,      // Device ID
    pub st_ino: u64,      // Inode number
    pub st_mode: u32,     // File type and permissions
    pub st_nlink: u32,    // Number of hard links
    pub st_uid: u32,      // Owner UID
    pub st_gid: u32,      // Owner GID
    pub st_rdev: u64,     // Device ID (if special file)
    pub st_size: u64,     // Size in bytes
    pub st_blksize: u64,  // Block size for I/O
    pub st_blocks: u64,   // Number of 512B blocks
    pub st_atime: u64,    // Access time
    pub st_atime_nsec: u64,
    pub st_mtime: u64,    // Modification time
    pub st_mtime_nsec: u64,
    pub st_ctime: u64,    // Status change time
    pub st_ctime_nsec: u64,
}
```

### Location
- `kernel/src/syscall/mod.rs`
- Update `userspace/libsyscall/src/lib.rs` to match

### Exit Criteria
- [ ] Kernel Stat matches userspace Stat
- [ ] fstat returns all fields
- [ ] Existing tests still pass

---

## Step 5: Add File Mode Constants

**Why**: Need standard POSIX file type and permission bits.

### Design

```rust
// File types (st_mode high bits)
pub const S_IFMT:   u32 = 0o170000;  // Mask for file type
pub const S_IFSOCK: u32 = 0o140000;  // Socket
pub const S_IFLNK:  u32 = 0o120000;  // Symbolic link
pub const S_IFREG:  u32 = 0o100000;  // Regular file
pub const S_IFBLK:  u32 = 0o060000;  // Block device
pub const S_IFDIR:  u32 = 0o040000;  // Directory
pub const S_IFCHR:  u32 = 0o020000;  // Character device
pub const S_IFIFO:  u32 = 0o010000;  // FIFO

// Permission bits
pub const S_ISUID:  u32 = 0o4000;    // Set UID
pub const S_ISGID:  u32 = 0o2000;    // Set GID
pub const S_ISVTX:  u32 = 0o1000;    // Sticky bit
pub const S_IRWXU:  u32 = 0o0700;    // Owner RWX
pub const S_IRUSR:  u32 = 0o0400;    // Owner read
pub const S_IWUSR:  u32 = 0o0200;    // Owner write
pub const S_IXUSR:  u32 = 0o0100;    // Owner execute
// ... etc for group and other
```

### Location
- `kernel/src/fs/mode.rs` (new file)

### Exit Criteria
- [ ] All POSIX mode constants defined
- [ ] Helper functions: `S_ISREG()`, `S_ISDIR()`, etc.
- [ ] Used by tmpfs and initramfs

---

## Verification

After completing Phase 12:
- [ ] Kernel builds
- [ ] All existing tests pass
- [ ] RwLock works correctly
- [ ] Path parsing handles edge cases
- [ ] Mount table can mount/lookup/umount
