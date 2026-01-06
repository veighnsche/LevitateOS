# Phase 2: Design — Writable Filesystem for Levbox

**Phase:** Design (Question-Heavy)  
**Status:** ✅ COMPLETE  
**Team:** TEAM_193

---

## 1. Proposed Solution

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Userspace                             │
│  levbox utilities → libsyscall → svc #0                     │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                    Kernel Syscalls                           │
│  sys_openat, sys_read, sys_write, sys_mkdirat, etc.         │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                      VFS Layer                               │
│  Route operations based on path prefix                       │
│  /tmp/* → Tmpfs    |    /* → Initramfs                      │
└────────────┬────────────────────────┬───────────────────────┘
             │                        │
┌────────────▼───────────┐ ┌─────────▼────────────────────────┐
│       Tmpfs            │ │       Initramfs                   │
│  In-memory writable    │ │  Read-only CPIO archive           │
└────────────────────────┘ └───────────────────────────────────┘
```

### 1.2 User-Facing Behavior

| Operation | Behavior |
|-----------|----------|
| `mkdir /tmp/foo` | Creates directory in tmpfs |
| `touch /tmp/bar` | Creates empty file in tmpfs |
| `echo "x" > /tmp/bar` | Writes to tmpfs file |
| `cat /tmp/bar` | Reads from tmpfs file |
| `rm /tmp/bar` | Deletes tmpfs file |
| `cp /init /tmp/init` | Copies from initramfs to tmpfs |
| `mkdir /etc/foo` | Returns `EROFS` (initramfs is read-only) |

### 1.3 System Behavior

1. At boot, kernel initializes tmpfs with empty `/tmp` directory
2. Path resolution checks prefix:
   - `/tmp/*` → route to tmpfs
   - Everything else → route to initramfs
3. Syscalls dispatch to appropriate filesystem
4. FdType extended to track tmpfs file handles

---

## 2. Data Model

### 2.1 TmpfsNode

```rust
pub enum TmpfsNodeType {
    File,
    Directory,
    Symlink,
}

pub struct TmpfsNode {
    /// Unique inode number
    pub ino: u64,
    /// Node name (not full path)
    pub name: String,
    /// Node type
    pub node_type: TmpfsNodeType,
    /// File content (for files)
    pub data: Vec<u8>,
    /// Child nodes (for directories)
    pub children: Vec<Arc<Mutex<TmpfsNode>>>,
    /// Creation time (optional)
    pub ctime: u64,
    /// Modification time (optional)
    pub mtime: u64,
}
```

### 2.2 Tmpfs State

```rust
pub struct Tmpfs {
    /// Root directory node
    root: Arc<Mutex<TmpfsNode>>,
    /// Next inode number
    next_ino: AtomicU64,
    /// Total bytes used
    bytes_used: AtomicUsize,
    /// Max bytes allowed (optional limit)
    max_bytes: Option<usize>,
}
```

### 2.3 FdType Extension

```rust
pub enum FdType {
    Stdin,
    Stdout,
    Stderr,
    InitramfsFile { file_index: usize, offset: usize },
    InitramfsDir { dir_index: usize, offset: usize },
    // NEW:
    TmpfsFile { node: Arc<Mutex<TmpfsNode>>, offset: usize },
    TmpfsDir { node: Arc<Mutex<TmpfsNode>>, offset: usize },
}
```

---

## 3. API Design

### 3.1 Tmpfs Internal API

```rust
impl Tmpfs {
    /// Create a new tmpfs instance
    pub fn new() -> Self;
    
    /// Lookup a node by path
    pub fn lookup(&self, path: &str) -> Option<Arc<Mutex<TmpfsNode>>>;
    
    /// Create a file
    pub fn create_file(&self, path: &str) -> Result<Arc<Mutex<TmpfsNode>>, Errno>;
    
    /// Create a directory
    pub fn create_dir(&self, path: &str) -> Result<Arc<Mutex<TmpfsNode>>, Errno>;
    
    /// Remove a file or empty directory
    pub fn remove(&self, path: &str) -> Result<(), Errno>;
    
    /// Rename/move a node
    pub fn rename(&self, old_path: &str, new_path: &str) -> Result<(), Errno>;
}
```

### 3.2 Syscall Changes

| Syscall | Change |
|---------|--------|
| `sys_openat` | Check path prefix, route to tmpfs or initramfs |
| `sys_read` | Handle TmpfsFile fd type |
| `sys_write` | Handle TmpfsFile fd type (NEW: write to files) |
| `sys_mkdirat` | Route to tmpfs if `/tmp/*` prefix |
| `sys_unlinkat` | Route to tmpfs if `/tmp/*` prefix |
| `sys_renameat` | Route to tmpfs (both paths must be in tmpfs) |
| `sys_fstat` | Handle TmpfsFile/TmpfsDir fd types |
| `sys_getdents` | Handle TmpfsDir fd type |

---

## 4. Behavioral Decisions (QUESTIONS)

### 4.1 Path Handling

| Q# | Question | Options | Recommendation |
|----|----------|---------|----------------|
| Q1 | Where should tmpfs be mounted? | (a) `/tmp` only (b) `/` overlay (c) configurable | **(a) /tmp only** — simplest, clear separation |
| Q2 | How to handle `..` and `.` in paths? | (a) Resolve in kernel (b) Require clean paths | **(a) Resolve in kernel** — match Linux |
| Q3 | Case sensitivity? | (a) Case-sensitive (b) Case-insensitive | **(a) Case-sensitive** — match Linux |

### 4.2 File Operations

| Q# | Question | Options | Recommendation |
|----|----------|---------|----------------|
| Q4 | Max file size? | (a) Unlimited (b) 1MB (c) 16MB (d) configurable | **(c) 16MB** — reasonable for temp files |
| Q5 | Max total tmpfs size? | (a) Unlimited (b) 64MB (c) configurable | **(b) 64MB** — prevent OOM |
| Q6 | What happens when tmpfs is full? | (a) ENOSPC (b) Evict old files | **(a) ENOSPC** — predictable |
| Q7 | O_CREAT flag for openat? | (a) Implement now (b) Defer | **(a) Implement now** — needed for touch/cp |
| Q8 | O_TRUNC flag for openat? | (a) Implement now (b) Defer | **(a) Implement now** — needed for > redirect |

### 4.3 Directory Operations

| Q# | Question | Options | Recommendation |
|----|----------|---------|----------------|
| Q9 | rmdir on non-empty dir? | (a) ENOTEMPTY (b) Recursive delete | **(a) ENOTEMPTY** — match POSIX |
| Q10 | mkdir -p (recursive)? | (a) Kernel support (b) Userspace loop | **(b) Userspace loop** — simpler kernel |

### 4.4 Special Cases

| Q# | Question | Options | Recommendation |
|----|----------|---------|----------------|
| Q11 | Rename across filesystems? | (a) EXDEV error (b) Copy+delete | **(a) EXDEV** — match Linux |
| Q12 | Hard links? | (a) Support (b) Defer (EOPNOTSUPP) | **(b) Defer** — complex for v1 |
| Q13 | Symlinks? | (a) Support (b) Defer | **(b) Defer** — not critical for Phase 11 |

### 4.5 Concurrency

| Q# | Question | Options | Recommendation |
|----|----------|---------|----------------|
| Q14 | Locking granularity? | (a) Global lock (b) Per-node lock | **(a) Global lock** — simpler, sufficient for now |
| Q15 | Multiple writers to same file? | (a) Last-write-wins (b) ETXTBSY | **(a) Last-write-wins** — match Linux |

---

## 5. Design Alternatives Considered

### 5.1 VFS Trait Abstraction

**Considered:** Define a `Filesystem` trait and have both initramfs and tmpfs implement it.

**Decision:** Defer. For Phase 11, simple path-prefix routing is sufficient. A full VFS abstraction can come later (Phase 14+).

### 5.2 Overlay Filesystem

**Considered:** Mount tmpfs as an overlay on top of initramfs, so writes to any path go to tmpfs.

**Decision:** Defer. This adds complexity (copy-on-write, whiteout files). `/tmp`-only is simpler.

### 5.3 Block-Backed Storage

**Considered:** Use VirtIO block device for persistent storage.

**Decision:** Defer. tmpfs (RAM-only) is simpler and sufficient for Phase 11.

---

## 6. Open Questions Summary

**✅ All questions answered by USER (2026-01-06) — recommendations accepted.**

| Priority | Q# | Question | Answer |
|----------|-----|----------|--------|
| P0 | Q1 | Mount point: `/tmp` only? | ✅ Yes |
| P0 | Q4 | Max file size: 16MB? | ✅ Yes |
| P0 | Q5 | Max tmpfs size: 64MB? | ✅ Yes |
| P1 | Q7 | Implement O_CREAT now? | ✅ Yes |
| P1 | Q8 | Implement O_TRUNC now? | ✅ Yes |
| P2 | Q12 | Defer hard links (EOPNOTSUPP)? | ✅ Yes |
| P2 | Q13 | Defer symlinks? | ✅ Yes |

---

## 7. Phase 2 Steps

### Step 1: Draft Initial Design ✓
- Architecture diagram
- Data structures defined

### Step 2: Define Behavioral Contracts ✓
- Listed all edge cases as questions
- Documented recommendations

### Step 3: Review Against Architecture ✓
- Fits existing syscall model
- Uses existing allocator
- Follows Rule 5 (clean design over hacks)

### Step 4: Finalize After Questions Answered
- [ ] Waiting for user answers to Q1-Q15
- [ ] Update design based on answers

---

## Next Phase

After questions are answered, proceed to **Phase 3: Implementation** with:
- Step 1: Create tmpfs module skeleton
- Step 2: Implement TmpfsNode and Tmpfs structs
- Step 3: Update FdType enum
- Step 4: Update sys_openat for path routing
- Step 5: Implement sys_write for tmpfs files
- Step 6: Implement sys_mkdirat for tmpfs
- Step 7: Implement sys_unlinkat for tmpfs
- Step 8: Implement sys_renameat for tmpfs
- Step 9: Update sys_getdents for tmpfs directories
