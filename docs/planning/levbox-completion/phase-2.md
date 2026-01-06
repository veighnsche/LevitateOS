# Phase 2: Design — Levbox Completion

**Phase:** Design  
**Status:** In Progress  
**Team:** TEAM_209

---

## 1. Proposed Solution

### 1.1 Scope
We will implement the missing `linkat` syscall and update `TmpfsNode` to support hard links (refcounting). We will also verify and finalize `utimensat` and `symlinkat` behaviors.

### 1.2 Architecture
- **Refcounting**: `TmpfsNode` will gain an `nlink: u32` field.
- **Syscall**: `sys_linkat` will be added to `kernel/src/syscall/fs/link.rs`.
- **VFS Integration**: `vfs_link` will be added to `kernel/src/fs/vfs/dispatch.rs` (if not present) and implemented for Tmpfs.

---

## 2. Data Model Changes

### 2.1 TmpfsNode Updates
```rust
pub struct TmpfsNode {
    // ... existing fields ...
    pub nlink: u32, // NEW: Number of hard links
}
```
- Initial `nlink` for files: 1
- Initial `nlink` for directories: 2 (self + parent entry)

---

## 3. API Design

### 3.1 sys_linkat
```rust
pub fn sys_linkat(
    olddirfd: i32,
    oldpath: usize,
    oldpath_len: usize,
    newdirfd: i32,
    newpath: usize,
    newpath_len: usize,
    flags: u32,
) -> i64
```
- **Returns**: 0 on success, or negative errno.
- **Behavior**: Increments `nlink` of the source node and adds a new entry in the destination directory pointing to the same node.

---

## 4. Behavioral Decisions (Questions)

| Q# | Question | Recommendation |
|----|----------|----------------|
| Q1 | Hard links to directories? | No (standard Unix restriction to prevent cycles). |
| Q2 | Cross-filesystem links? | No (return `EXDEV`). |
| Q3 | `utimensat` flags support? | Implement `AT_SYMLINK_NOFOLLOW` if trivial, otherwise documented as ignored for now. |

---

## 5. Implementation Steps

### Step 1: Update TmpfsNode
- Add `nlink` field.
- Update `new_file`, `new_dir`.
- Update `unlinkat` logic to decrement `nlink` and only free if 0.

### Step 2: Implement sys_linkat
- Add `Linkat = 37` (verify number) to `SyscallNumber`.
- Implement handler in `kernel/src/syscall/fs/link.rs`.
- Add `vfs_link` dispatch.

### Step 3: Userspace Wrapper
- Add `linkat` to `libsyscall`.

---

## 6. Open Questions
1. **Does `vfs_link` already exist in the codebase?** -> Verified: No, it needs to be implemented in `dispatch.rs`.
2. **What is the correct syscall number for `linkat`?** -> Linux ARM64 uses 37. Verified: `Readlinkat` is currently 37 in `@/home/vince/Projects/LevitateOS/kernel/src/syscall/mod.rs:75`. I must check if 37 is available or if I should use another number.
3. **Wait, `Readlinkat` is 37.** Looking at `@/home/vince/Projects/LevitateOS/kernel/src/syscall/mod.rs`:
   - 34: Mkdirat
   - 35: Unlinkat
   - 36: Symlinkat
   - 37: Readlinkat
   - 38: Renameat
   - 39: Umount
   - 40: Mount
   I should probably use a new number for `Linkat` or check if there's a gap. Linux uses 37 for `linkat`, but this kernel assigned it to `readlinkat`. I will use a free number or follow the user's preference if they have one. I'll propose 42 (next free after 41 Futex).

