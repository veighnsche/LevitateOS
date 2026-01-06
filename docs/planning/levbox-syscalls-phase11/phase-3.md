# Phase 3: Implementation — Writable Filesystem for Levbox

**Phase:** Implementation  
**Status:** ✅ COMPLETE  
**Team:** TEAM_194

---

## Prerequisites

- [x] All Phase 2 questions answered ✅
- [x] Design finalized based on answers ✅
- [x] Kernel builds cleanly before starting ✅

---

## Implementation Steps

### Step 1: Create tmpfs Module Skeleton

**Files:**
- `kernel/src/fs/tmpfs.rs` (new)
- `kernel/src/fs/mod.rs` (update)

**Tasks:**
1. Create `kernel/src/fs/tmpfs.rs` with module structure
2. Add `pub mod tmpfs;` to `kernel/src/fs/mod.rs`
3. Define `TmpfsNode` and `TmpfsNodeType` enums
4. Define `Tmpfs` struct with `new()` constructor
5. Add `pub static TMPFS: Mutex<Option<Tmpfs>>` global

**Exit Criteria:**
- Kernel compiles with empty tmpfs module

---

### Step 2: Implement TmpfsNode Operations

**Tasks:**
1. Implement `TmpfsNode::new_file(name: &str) -> Self`
2. Implement `TmpfsNode::new_dir(name: &str) -> Self`
3. Implement `Tmpfs::lookup(path: &str) -> Option<Arc<Mutex<TmpfsNode>>>`
4. Implement path parsing and traversal

**Exit Criteria:**
- Can create nodes and look them up by path

---

### Step 3: Update FdType Enum

**File:** `kernel/src/task/fd_table.rs`

**Tasks:**
1. Add `TmpfsFile { node: ..., offset: usize }` variant
2. Add `TmpfsDir { node: ..., offset: usize }` variant
3. Update `FdTable::alloc()` if needed

**Exit Criteria:**
- FdType can represent tmpfs file handles

---

### Step 4: Update sys_openat for Path Routing

**File:** `kernel/src/syscall/fs.rs`

**Tasks:**
1. Add path prefix check: if starts with `/tmp/` → route to tmpfs
2. Implement `open_tmpfs_path()` helper
3. Handle O_CREAT flag to create new files
4. Handle O_TRUNC flag to truncate existing files
5. Return TmpfsFile/TmpfsDir fd type

**Exit Criteria:**
- `open("/tmp/foo", O_CREAT)` creates and opens tmpfs file

---

### Step 5: Implement sys_write for Tmpfs Files

**File:** `kernel/src/syscall/fs.rs`

**Tasks:**
1. Update `sys_write()` to handle `FdType::TmpfsFile`
2. Implement write-at-offset logic
3. Extend file data vector as needed
4. Update offset after write

**Exit Criteria:**
- Can write data to tmpfs files

---

### Step 6: Implement sys_mkdirat for Tmpfs

**File:** `kernel/src/syscall/fs.rs`

**Tasks:**
1. Update `sys_mkdirat()` to check path prefix
2. If `/tmp/*` → call `Tmpfs::create_dir()`
3. Handle EEXIST if directory exists
4. Handle ENOENT if parent doesn't exist

**Exit Criteria:**
- `mkdir /tmp/test` creates directory

---

### Step 7: Implement sys_unlinkat for Tmpfs

**File:** `kernel/src/syscall/fs.rs`

**Tasks:**
1. Update `sys_unlinkat()` to check path prefix
2. If `/tmp/*` → call `Tmpfs::remove()`
3. Handle AT_REMOVEDIR flag for directories
4. Handle ENOTEMPTY for non-empty directories

**Exit Criteria:**
- `rm /tmp/file` and `rmdir /tmp/dir` work

---

### Step 8: Implement sys_renameat for Tmpfs

**File:** `kernel/src/syscall/fs.rs`

**Tasks:**
1. Update `sys_renameat()` to check path prefixes
2. If both `/tmp/*` → call `Tmpfs::rename()`
3. If mixed → return EXDEV
4. Handle overwrite of existing target

**Exit Criteria:**
- `mv /tmp/a /tmp/b` works

---

### Step 9: Update sys_read for Tmpfs Files

**File:** `kernel/src/syscall/fs.rs`

**Tasks:**
1. Update `sys_read()` to handle `FdType::TmpfsFile`
2. Read from node's data vector at current offset
3. Update offset after read

**Exit Criteria:**
- `cat /tmp/file` reads content

---

### Step 10: Update sys_getdents for Tmpfs Directories

**File:** `kernel/src/syscall/fs.rs`

**Tasks:**
1. Handle `FdType::TmpfsDir` in `sys_getdents()`
2. Iterate over node's children
3. Return dirent64 records

**Exit Criteria:**
- `ls /tmp` shows tmpfs contents

---

## Verification

After each step:
1. Kernel compiles without errors
2. Existing tests still pass
3. New functionality works as expected

---

## UoW Breakdown (if needed)

If any step is too large, split into UoW files:
- `phase-3-step-4-uow-1.md` — Path routing logic
- `phase-3-step-4-uow-2.md` — O_CREAT implementation
- `phase-3-step-4-uow-3.md` — O_TRUNC implementation
