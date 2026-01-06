# Phase 2: Design — Remaining Levbox Syscalls

**Phase:** Design  
**Status:** ✅ COMPLETE  
**Team:** TEAM_196

---

## 1. Proposed Solution

### 1.1 Scope

| Feature | Priority | Complexity | Decision |
|---------|----------|------------|----------|
| `utimensat` | P1 | Medium | Implement |
| `symlinkat` | P2 | Medium | Implement |
| `linkat` | P3 | High | **Defer** |
| Add utilities to initramfs | P0 | Low | Implement first |

### 1.2 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Userspace                                 │
│  touch → utimensat()    ln -s → symlinkat()                 │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                    Kernel Syscalls                           │
│  sys_utimensat()        sys_symlinkat()                     │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                      Tmpfs                                   │
│  TmpfsNode { mtime, atime }    TmpfsNodeType::Symlink       │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Data Model Changes

### 2.1 TmpfsNode Extensions

```rust
pub enum TmpfsNodeType {
    File,
    Directory,
    Symlink,  // NEW
}

pub struct TmpfsNode {
    pub ino: u64,
    pub name: String,
    pub node_type: TmpfsNodeType,
    pub data: Vec<u8>,           // For files: content, for symlinks: target path
    pub children: Vec<...>,       // For directories
    pub atime: u64,              // NEW: Access time (seconds since epoch)
    pub mtime: u64,              // NEW: Modification time
    pub ctime: u64,              // NEW: Creation time
}
```

### 2.2 FdType (No changes needed)

Symlinks are not opened directly - they're resolved to their target.

---

## 3. API Design

### 3.1 sys_utimensat

```rust
/// Set file timestamps
/// 
/// # Arguments
/// * `dirfd` - Directory fd (AT_FDCWD for cwd)
/// * `path` - Path to file
/// * `path_len` - Length of path
/// * `times` - Pointer to [atime, mtime] timespec array (NULL = now)
/// * `flags` - AT_SYMLINK_NOFOLLOW (0x100) to not follow symlinks
///
/// # Returns
/// * 0 on success
/// * -ENOENT if file not found
/// * -EROFS if not in /tmp
pub fn sys_utimensat(
    dirfd: i32,
    path: usize,
    path_len: usize,
    times: usize,  // Pointer to [Timespec; 2] or 0 for "now"
    flags: u32,
) -> i64
```

### 3.2 sys_symlinkat

```rust
/// Create a symbolic link
///
/// # Arguments
/// * `target` - Target path the symlink points to
/// * `target_len` - Length of target
/// * `linkdirfd` - Directory fd for link path
/// * `linkpath` - Path for the new symlink
/// * `linkpath_len` - Length of link path
///
/// # Returns
/// * 0 on success
/// * -EEXIST if link already exists
/// * -EROFS if not in /tmp
pub fn sys_symlinkat(
    target: usize,
    target_len: usize,
    linkdirfd: i32,
    linkpath: usize,
    linkpath_len: usize,
) -> i64
```

---

## 4. Behavioral Decisions

### 4.1 utimensat Behavior

| Q# | Question | Decision |
|----|----------|----------|
| Q1 | What if times is NULL? | Use current time for both |
| Q2 | What if file doesn't exist? | Return ENOENT (don't create) |
| Q3 | Support UTIME_NOW/UTIME_OMIT? | Yes, for compatibility |
| Q4 | Update which timestamps? | atime and mtime as specified |

### 4.2 symlinkat Behavior

| Q# | Question | Decision |
|----|----------|----------|
| Q5 | Resolve symlinks on read? | No - just return target path |
| Q6 | Symlink to non-existent target? | Allow (dangling symlinks OK) |
| Q7 | Symlink target max length? | 256 bytes (same as path) |
| Q8 | Can symlink to outside /tmp? | Yes - target is just stored |

### 4.3 touch Utility Behavior

| Q# | Question | Decision |
|----|----------|----------|
| Q9 | `touch file` (no flags)? | Create if not exist, update mtime |
| Q10 | `touch -c file`? | Don't create if not exist |
| Q11 | `touch -a file`? | Update atime only |
| Q12 | `touch -m file`? | Update mtime only |

---

## 5. Implementation Steps

### Step 1: Add Levbox Utilities to Initramfs (P0)
- Update `scripts/make_initramfs.sh` to include ls, mkdir, rm, rmdir, mv, cp
- Build and test

### Step 2: Add Timestamps to TmpfsNode (P1a)
- Add atime, mtime, ctime fields
- Update create_file/create_dir to set times
- Update sys_fstat to return times

### Step 3: Implement sys_utimensat (P1b)
- Add syscall number and dispatch
- Add libsyscall wrapper
- Implement kernel handler

### Step 4: Create touch Utility (P1c)
- Basic implementation with -a, -c, -m flags
- Uses utimensat syscall

### Step 5: Add Symlink Support to Tmpfs (P2a)
- Add TmpfsNodeType::Symlink variant
- Add create_symlink method
- Store target in data field

### Step 6: Implement sys_symlinkat (P2b)
- Add syscall number and dispatch
- Add libsyscall wrapper
- Implement kernel handler

### Step 7: Create ln Utility (P2c)
- Basic implementation with -s flag
- Hard links return error (not implemented)

---

## 6. Questions Summary

**Status:** All design decisions confirmed by USER (2026-01-06).

| Q# | Decision | Confirmed |
|----|----------|-----------|
| Q1-Q4 | utimensat behavior | ✅ |
| Q5-Q8 | symlinkat behavior | ✅ |
| Q9-Q12 | touch utility behavior | ✅ |

> **Note:** Hard links (`linkat`) deferred — not needed for Phase 11 goals.

---

## Next Phase

Proceed to **Phase 3: Implementation** with steps in priority order.
