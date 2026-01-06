# Phase 14: Filesystem Migration

**Phase:** Migration & Integration  
**Status:** Not Started  
**Team:** TBD  
**Depends On:** Phase 13 (Core VFS)

---

## Objective

Migrate existing filesystems (tmpfs, initramfs, FAT32, ext4) to implement the VFS InodeOps trait.

---

## Migration Order

| Priority | Filesystem | Current Implementation | Complexity |
|----------|------------|----------------------|------------|
| 1 | tmpfs | `TmpfsNode` + direct syscall dispatch | Medium |
| 2 | initramfs | CPIO index + FdType dispatch | Low |
| 3 | FAT32 | `embedded-sdmmc` wrapper | Medium |
| 4 | ext4 | `ext4-view` wrapper (read-only) | Low |

---

## Step 1: Migrate tmpfs

### Current Structure
```rust
pub struct TmpfsNode {
    pub ino: u64,
    pub name: String,
    pub node_type: TmpfsNodeType,
    pub data: Vec<u8>,
    pub children: Vec<Arc<Spinlock<TmpfsNode>>>,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
}
```

### Target Structure
```rust
/// Tmpfs-specific inode data
pub struct TmpfsInodeData {
    /// File content (for regular files)
    data: RwLock<Vec<u8>>,
    /// Children (for directories)
    children: RwLock<HashMap<String, Arc<Inode>>>,
    /// Symlink target (for symlinks)
    target: Option<String>,
}

/// Tmpfs inode operations
pub static TMPFS_FILE_OPS: TmpfsFileOps = TmpfsFileOps;
pub static TMPFS_DIR_OPS: TmpfsDirOps = TmpfsDirOps;
pub static TMPFS_SYMLINK_OPS: TmpfsSymlinkOps = TmpfsSymlinkOps;

impl InodeOps for TmpfsFileOps { ... }
impl InodeOps for TmpfsDirOps { ... }
impl InodeOps for TmpfsSymlinkOps { ... }
```

### Files to Modify
- `kernel/src/fs/tmpfs.rs` — Implement InodeOps
- `kernel/src/syscall/fs.rs` — Remove TmpfsFile/TmpfsDir dispatch

### Exit Criteria
- [ ] tmpfs implements InodeOps
- [ ] All tmpfs tests pass
- [ ] Can create/read/write/delete files in /tmp

---

## Step 2: Migrate initramfs

### Current Structure
- CPIO archive parsed at boot
- Files indexed by position in archive
- Read-only access via `FdType::InitramfsFile`

### Target Structure
```rust
/// Initramfs-specific inode data
pub struct InitramfsInodeData {
    /// Pointer to data in CPIO archive
    data: &'static [u8],
    /// Children for directories
    children: HashMap<String, Arc<Inode>>,
}

/// Initramfs inode operations (read-only)
pub struct InitramfsOps;

impl InodeOps for InitramfsOps {
    fn lookup(&self, ...) -> Result<Arc<Inode>> { ... }
    fn read(&self, ...) -> Result<usize> { ... }
    fn readdir(&self, ...) -> Result<Option<DirEntry>> { ... }
    
    // Write operations return EROFS
    fn create(&self, ...) -> Result<Arc<Inode>> { Err(VfsError::ReadOnlyFs) }
    fn mkdir(&self, ...) -> Result<Arc<Inode>> { Err(VfsError::ReadOnlyFs) }
    fn unlink(&self, ...) -> Result<()> { Err(VfsError::ReadOnlyFs) }
    // etc.
}
```

### Files to Modify
- `kernel/src/fs/initramfs.rs` — Implement InodeOps
- `kernel/src/syscall/fs.rs` — Remove InitramfsFile/InitramfsDir dispatch

### Exit Criteria
- [ ] initramfs implements InodeOps
- [ ] Can read files from initramfs via VFS
- [ ] cat, ls work on initramfs files

---

## Step 3: Migrate FAT32

### Current Structure
- Uses `embedded-sdmmc` crate
- Mounted at boot but not integrated with syscalls

### Target Structure
```rust
pub struct Fat32InodeData {
    /// Path in FAT32 filesystem
    path: String,
    /// File size (cached)
    size: u64,
    /// Is directory
    is_dir: bool,
}

pub struct Fat32Ops {
    /// Reference to embedded-sdmmc volume manager
    volume: Arc<Mutex<VolumeManager>>,
}

impl InodeOps for Fat32Ops { ... }
```

### Exit Criteria
- [ ] FAT32 implements InodeOps
- [ ] Can mount FAT32 at arbitrary path
- [ ] Can read files from FAT32 via VFS

---

## Step 4: Migrate ext4

### Current Structure
- Uses `ext4-view` crate (read-only)
- Not currently exposed to userspace

### Target Structure
```rust
pub struct Ext4InodeData {
    /// ext4 inode number
    ext4_ino: u64,
}

pub struct Ext4Ops {
    /// Reference to ext4 view
    fs: Arc<Ext4Fs>,
}

impl InodeOps for Ext4Ops {
    // Read-only implementation
}
```

### Exit Criteria
- [ ] ext4 implements InodeOps
- [ ] Can mount ext4 at arbitrary path
- [ ] Can read files from ext4 via VFS

---

## Step 5: Remove Legacy Dispatch

After all filesystems are migrated:

### Remove from FdType
```rust
// DELETE these variants:
InitramfsFile { file_index: usize, offset: usize },
InitramfsDir { dir_index: usize, offset: usize },
TmpfsFile { node: Arc<Spinlock<TmpfsNode>>, offset: usize },
TmpfsDir { node: Arc<Spinlock<TmpfsNode>>, offset: usize },
```

### Simplify Syscalls
- `sys_read` — Just calls `vfs_read(file)`
- `sys_write` — Just calls `vfs_write(file)`
- `sys_openat` — Just calls `vfs_open(path, flags)`
- etc.

### Exit Criteria
- [ ] No per-filesystem dispatch in syscall layer
- [ ] FdType only has Stdin/Stdout/Stderr/File
- [ ] All existing tests pass

---

## Step 6: Add mount/umount Syscalls

```rust
/// sys_mount(source, target, fstype, flags, data)
pub fn sys_mount(
    source: usize, source_len: usize,
    target: usize, target_len: usize,
    fstype: usize, fstype_len: usize,
    flags: u32,
    data: usize,
) -> i64;

/// sys_umount(target, flags)
pub fn sys_umount(target: usize, target_len: usize, flags: u32) -> i64;
```

### Exit Criteria
- [ ] Can mount tmpfs at arbitrary path
- [ ] Can unmount filesystems
- [ ] mount/umount update dentry cache

---

## Final Verification

- [ ] All filesystems work through VFS
- [ ] All levbox utilities work
- [ ] Boot sequence unchanged
- [ ] Performance acceptable
- [ ] No regressions in existing tests
