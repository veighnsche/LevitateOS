# Phase 13: Core VFS Implementation

**Phase:** Implementation  
**Status:** Not Started  
**Team:** TBD  
**Depends On:** Phase 12 (VFS Foundation)

---

## Objective

Implement the core Virtual Filesystem abstractions that provide a unified interface for all filesystems.

---

## Core Abstractions

### 1. Inode

Represents a file, directory, or other filesystem object.

```rust
/// Inode — in-memory representation of a file/directory
pub struct Inode {
    /// Unique identifier within filesystem
    pub ino: u64,
    /// File type and permissions
    pub mode: u32,
    /// Number of hard links
    pub nlink: AtomicU32,
    /// Owner user ID
    pub uid: u32,
    /// Owner group ID
    pub gid: u32,
    /// Size in bytes
    pub size: AtomicU64,
    /// Timestamps
    pub atime: AtomicU64,
    pub mtime: AtomicU64,
    pub ctime: AtomicU64,
    /// Filesystem-specific data
    pub private: Box<dyn Any + Send + Sync>,
    /// Operations table
    pub ops: &'static dyn InodeOps,
    /// Containing superblock
    pub sb: Weak<dyn Superblock>,
}
```

### 2. InodeOps Trait

Filesystem-specific operations.

```rust
pub trait InodeOps: Send + Sync {
    // Directory operations
    fn lookup(&self, inode: &Inode, name: &str) -> Result<Arc<Inode>, VfsError>;
    fn create(&self, inode: &Inode, name: &str, mode: u32) -> Result<Arc<Inode>, VfsError>;
    fn mkdir(&self, inode: &Inode, name: &str, mode: u32) -> Result<Arc<Inode>, VfsError>;
    fn unlink(&self, inode: &Inode, name: &str) -> Result<(), VfsError>;
    fn rmdir(&self, inode: &Inode, name: &str) -> Result<(), VfsError>;
    fn symlink(&self, inode: &Inode, name: &str, target: &str) -> Result<Arc<Inode>, VfsError>;
    fn link(&self, inode: &Inode, name: &str, target: &Inode) -> Result<(), VfsError>;
    fn rename(&self, old_dir: &Inode, old_name: &str, new_dir: &Inode, new_name: &str) -> Result<(), VfsError>;
    
    // File operations
    fn read(&self, inode: &Inode, offset: u64, buf: &mut [u8]) -> Result<usize, VfsError>;
    fn write(&self, inode: &Inode, offset: u64, buf: &[u8]) -> Result<usize, VfsError>;
    fn truncate(&self, inode: &Inode, size: u64) -> Result<(), VfsError>;
    
    // Symlink
    fn readlink(&self, inode: &Inode) -> Result<String, VfsError>;
    
    // Directory iteration
    fn readdir(&self, inode: &Inode, offset: usize) -> Result<Option<DirEntry>, VfsError>;
    
    // Metadata
    fn getattr(&self, inode: &Inode) -> Result<Stat, VfsError>;
    fn setattr(&self, inode: &Inode, attr: &SetAttr) -> Result<(), VfsError>;
}
```

### 3. Superblock

Represents a mounted filesystem instance.

```rust
pub trait Superblock: Send + Sync {
    /// Get the root inode
    fn root(&self) -> Arc<Inode>;
    
    /// Get filesystem statistics
    fn statfs(&self) -> Result<StatFs, VfsError>;
    
    /// Sync filesystem to disk
    fn sync(&self) -> Result<(), VfsError>;
    
    /// Filesystem type name
    fn fs_type(&self) -> &'static str;
}
```

### 4. File (Open File Handle)

```rust
/// An open file handle
pub struct File {
    /// The inode this file refers to
    pub inode: Arc<Inode>,
    /// Current position for read/write
    pub offset: AtomicU64,
    /// Open flags (O_RDONLY, O_WRONLY, O_RDWR, etc.)
    pub flags: u32,
    /// File-specific operations (may differ from inode ops)
    pub ops: Option<&'static dyn FileOps>,
}

pub trait FileOps: Send + Sync {
    fn read(&self, file: &File, buf: &mut [u8]) -> Result<usize, VfsError>;
    fn write(&self, file: &File, buf: &[u8]) -> Result<usize, VfsError>;
    fn seek(&self, file: &File, offset: i64, whence: SeekWhence) -> Result<u64, VfsError>;
    fn poll(&self, file: &File) -> PollEvents;
    fn ioctl(&self, file: &File, cmd: u32, arg: usize) -> Result<i32, VfsError>;
}
```

### 5. Dentry (Directory Entry Cache)

```rust
/// Cached path-to-inode mapping
pub struct Dentry {
    /// Name of this entry
    pub name: String,
    /// Parent dentry (None for root)
    pub parent: Option<Weak<Dentry>>,
    /// Inode this dentry points to
    pub inode: Option<Arc<Inode>>,  // None = negative dentry
    /// Children dentries
    pub children: RwLock<HashMap<String, Arc<Dentry>>>,
    /// Mount point (if something is mounted here)
    pub mounted: RwLock<Option<Arc<dyn Superblock>>>,
}
```

---

## Files to Create

| File | Contents |
|------|----------|
| `kernel/src/fs/inode.rs` | `Inode`, `InodeOps` trait |
| `kernel/src/fs/superblock.rs` | `Superblock` trait |
| `kernel/src/fs/file.rs` | `File`, `FileOps` trait |
| `kernel/src/fs/dentry.rs` | `Dentry`, dentry cache |
| `kernel/src/fs/vfs.rs` | VFS dispatch: `vfs_open`, `vfs_read`, etc. |
| `kernel/src/fs/error.rs` | `VfsError` enum |

---

## VFS Dispatch Layer

```rust
// kernel/src/fs/vfs.rs

/// Open a file by path
pub fn vfs_open(path: &Path, flags: u32, mode: u32) -> Result<Arc<File>, VfsError>;

/// Read from an open file
pub fn vfs_read(file: &File, buf: &mut [u8]) -> Result<usize, VfsError>;

/// Write to an open file
pub fn vfs_write(file: &File, buf: &[u8]) -> Result<usize, VfsError>;

/// Close a file
pub fn vfs_close(file: Arc<File>) -> Result<(), VfsError>;

/// Get file status
pub fn vfs_stat(path: &Path) -> Result<Stat, VfsError>;

/// Create a directory
pub fn vfs_mkdir(path: &Path, mode: u32) -> Result<(), VfsError>;

/// Remove a file
pub fn vfs_unlink(path: &Path) -> Result<(), VfsError>;

/// etc.
```

---

## FdType Simplification

### Before (Current)
```rust
pub enum FdType {
    Stdin,
    Stdout,
    Stderr,
    InitramfsFile { file_index: usize, offset: usize },
    InitramfsDir { dir_index: usize, offset: usize },
    TmpfsFile { node: Arc<Spinlock<TmpfsNode>>, offset: usize },
    TmpfsDir { node: Arc<Spinlock<TmpfsNode>>, offset: usize },
}
```

### After (VFS)
```rust
pub enum FdType {
    Stdin,
    Stdout,
    Stderr,
    File(Arc<File>),  // All regular files/dirs go through VFS
}
```

---

## Units of Work

- [ ] Create `VfsError` enum with error codes
- [ ] Define `InodeOps` trait
- [ ] Define `Superblock` trait
- [ ] Define `FileOps` trait
- [ ] Implement `Inode` struct
- [ ] Implement `File` struct
- [ ] Implement `Dentry` struct and cache
- [ ] Implement VFS dispatch functions
- [ ] Simplify `FdType` to use `Arc<File>`

---

## Exit Criteria

- [ ] All VFS types compile
- [ ] VFS dispatch layer works
- [ ] Can open/read/write/close files through VFS
- [ ] Dentry cache speeds up repeated lookups
- [ ] FdType simplified to use File
