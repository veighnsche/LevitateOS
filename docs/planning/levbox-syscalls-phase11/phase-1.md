# Phase 1: Discovery — Writable Filesystem for Levbox

**Phase:** Discovery  
**Status:** ✅ COMPLETE  
**Team:** TEAM_193

---

## 1. Feature Summary

### Problem Statement

Levbox utilities (`mkdir`, `rm`, `rmdir`, `mv`, `cp`, `touch`, `ln`) cannot function because:

1. The only filesystem is **initramfs** (read-only CPIO archive)
2. Kernel syscall handlers exist but return `EROFS` (read-only)
3. There is no writable storage layer

### Who Benefits

- **Users**: Can create, modify, and delete files
- **Shell**: Can have a working `/tmp` directory
- **Future features**: Pipes, process working directories, application data

### Solution Direction

Implement **tmpfs** — a simple in-memory filesystem that:
- Lives entirely in RAM (backed by kernel allocator)
- Supports basic operations: create, read, write, delete, rename
- Can be mounted at `/tmp` or overlay on root

---

## 2. Success Criteria

| Criteria | Verification |
|----------|--------------|
| Create directory | `mkdir /tmp/test` succeeds |
| Create file | `touch /tmp/file` or redirect creates file |
| Write to file | `echo "hello" > /tmp/file` works |
| Read file | `cat /tmp/file` returns content |
| Remove file | `rm /tmp/file` succeeds |
| Remove directory | `rmdir /tmp/test` succeeds |
| Rename/move | `mv /tmp/a /tmp/b` succeeds |
| Copy file | `cp /init /tmp/init` succeeds |

---

## 3. Current State Analysis

### 3.1 How the System Works Today

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
│                    Initramfs (CPIO)                          │
│  Read-only archive loaded at boot                            │
│  Parsed by los_utils::cpio                                   │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Current File Descriptor Model

From `kernel/src/task/fd_table.rs`:

```rust
pub enum FdType {
    Stdin,
    Stdout,
    Stderr,
    InitramfsFile { file_index: usize, offset: usize },
    InitramfsDir { dir_index: usize, offset: usize },
}
```

No concept of "writable file" or "tmpfs file" exists.

### 3.3 Current Syscall Handlers

| Syscall | Handler | Current Behavior |
|---------|---------|------------------|
| `openat` | `sys_openat` | Opens initramfs files read-only |
| `read` | `sys_read` | Reads from stdin or initramfs |
| `write` | `sys_write` | Writes to stdout/stderr only |
| `mkdirat` | `sys_mkdirat` | Returns `EROFS` |
| `unlinkat` | `sys_unlinkat` | Returns `EROFS` |
| `renameat` | `sys_renameat` | Returns `EROFS` |

### 3.4 What Users Do Instead

Currently impossible to:
- Create any files or directories
- Modify existing files
- Store application state

---

## 4. Codebase Reconnaissance

### 4.1 Files to Modify

| File | Purpose |
|------|---------|
| `kernel/src/fs/mod.rs` | Add tmpfs module, VFS trait |
| `kernel/src/fs/tmpfs.rs` | New: tmpfs implementation |
| `kernel/src/syscall/fs.rs` | Update syscall handlers |
| `kernel/src/task/fd_table.rs` | Add TmpfsFile fd type |

### 4.2 Files to Read (Reference)

| File | Purpose |
|------|---------|
| `kernel/src/fs/mod.rs` | Current INITRAMFS global |
| `kernel/src/syscall/fs.rs` | Current syscall implementations |
| `crates/utils/src/cpio.rs` | CPIO parsing (reference for entry structure) |
| `.external-kernels/redox-kernel/` | Reference tmpfs implementations |

### 4.3 Key Data Structures

**CpioEntry** (reference):
```rust
pub struct CpioEntry<'a> {
    pub name: &'a str,
    pub ino: u64,
    pub mode: u32,
    pub data: &'a [u8],
    pub entry_type: CpioEntryType,
}
```

**Proposed TmpfsNode**:
```rust
pub struct TmpfsNode {
    pub name: String,
    pub ino: u64,
    pub node_type: TmpfsNodeType,
    pub data: Vec<u8>,        // For files
    pub children: Vec<...>,   // For directories
}
```

### 4.4 Tests and Golden Files

| Test | Impact |
|------|--------|
| `tests/golden_boot.txt` | May need update if boot message changes |
| `tests/golden_shutdown.txt` | No impact expected |

---

## 5. Constraints

### 5.1 Memory

- tmpfs lives in RAM — size limited by available memory
- Need to decide max file size / total tmpfs size
- Should use existing kernel allocator (slab for nodes, pages for data)

### 5.2 Concurrency

- Multiple processes may access tmpfs simultaneously
- Need proper locking (Mutex/RwLock on tmpfs state)

### 5.3 Persistence

- tmpfs is **not persistent** — data lost on reboot
- This is acceptable for Phase 11 (scratch space)

### 5.4 Compatibility

- Should follow Linux syscall semantics where possible
- File descriptors should work with existing read/write/close

---

## 6. Open Questions (Phase 1)

These are discovery questions, not design questions:

| Q# | Question | Notes |
|----|----------|-------|
| Q1 | Where should tmpfs be mounted? | `/tmp` seems natural |
| Q2 | Should tmpfs be the only writable fs? | For now, yes |
| Q3 | Max file/directory limits? | TBD in design phase |

---

## 7. Phase 1 Steps

### Step 1: Capture Feature Intent ✓
- Problem: No writable filesystem
- Solution: Implement tmpfs

### Step 2: Analyze Current State ✓
- Documented current fd model
- Documented current syscall behavior

### Step 3: Source Code Reconnaissance ✓
- Identified files to modify
- Identified reference implementations

---

## Next Phase

Proceed to **Phase 2: Design** to define:
- VFS abstraction layer
- Tmpfs data structures
- Syscall integration points
- Behavioral edge cases (generates questions)
