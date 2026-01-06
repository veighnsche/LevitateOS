# Phase 3: Implementation — Remaining Levbox Syscalls

**Phase:** Implementation  
**Status:** Ready  
**Team:** TBD

---

## Prerequisites

- [ ] Phase 2 design reviewed
- [ ] Kernel builds cleanly

---

## Step 1: Add Levbox Utilities to Initramfs (P0)

**Priority:** Highest — enables testing of existing functionality

### Tasks

1. Read `scripts/make_initramfs.sh` to understand build process
2. Add levbox binaries to initramfs:
   - `ls`, `mkdir`, `rmdir`, `rm`, `mv`, `cp`, `pwd`
3. Rebuild initramfs
4. Test in QEMU

### Files
- `scripts/make_initramfs.sh`
- `userspace/levbox/Cargo.toml`

### Exit Criteria
- `ls` command works in shell
- `mkdir /tmp/test` creates directory
- `rm /tmp/test` removes it

---

## Step 2: Add Timestamps to TmpfsNode (P1a)

### Tasks

1. Add `atime`, `mtime`, `ctime` fields to `TmpfsNode`
2. Update `new_file()` and `new_dir()` to set creation time
3. Update `write_file()` to update mtime
4. Update `sys_fstat` to include timestamps in Stat struct

### Files
- `kernel/src/fs/tmpfs.rs`
- `kernel/src/syscall/fs.rs`
- `kernel/src/syscall/mod.rs` (Stat struct)

### Exit Criteria
- TmpfsNode has timestamp fields
- Timestamps set on creation

---

## Step 3: Implement sys_utimensat (P1b)

### Tasks

1. Add `Utimensat = 88` to SyscallNumber enum
2. Add dispatch in syscall_dispatch
3. Implement `sys_utimensat` in fs.rs:
   - Validate path
   - Route to tmpfs if `/tmp/*`
   - Update atime/mtime on node
4. Add wrapper in libsyscall

### Files
- `kernel/src/syscall/mod.rs`
- `kernel/src/syscall/fs.rs`
- `userspace/libsyscall/src/lib.rs`

### Exit Criteria
- Syscall compiles and is dispatched
- Can update timestamps on tmpfs files

---

## Step 4: Create touch Utility (P1c)

### Tasks

1. Create `userspace/levbox/src/bin/touch.rs`
2. Implement options: `-a`, `-c`, `-m`, `--help`, `--version`
3. Use `openat` with O_CREAT to create file
4. Use `utimensat` to update timestamps
5. Add to Cargo.toml

### Files
- `userspace/levbox/src/bin/touch.rs` (new)
- `userspace/levbox/Cargo.toml`

### Exit Criteria
- `touch /tmp/file` creates empty file
- `touch -c /tmp/nonexistent` returns without creating

---

## Step 5: Add Symlink Support to Tmpfs (P2a)

### Tasks

1. Add `Symlink` variant to `TmpfsNodeType`
2. Add `new_symlink(name, target)` constructor
3. Add `create_symlink(path, target)` to Tmpfs
4. Add `read_link()` to get symlink target
5. Update `sys_fstat` to return symlink mode

### Files
- `kernel/src/fs/tmpfs.rs`
- `kernel/src/syscall/fs.rs`

### Exit Criteria
- Can create symlink nodes in tmpfs
- Can read symlink target

---

## Step 6: Implement sys_symlinkat (P2b)

### Tasks

1. Add `Symlinkat = 36` to SyscallNumber enum
2. Add dispatch in syscall_dispatch
3. Implement `sys_symlinkat` in fs.rs
4. Add wrapper in libsyscall

### Files
- `kernel/src/syscall/mod.rs`
- `kernel/src/syscall/fs.rs`
- `userspace/libsyscall/src/lib.rs`

### Exit Criteria
- `symlinkat("target", dirfd, "link")` creates symlink

---

## Step 7: Create ln Utility (P2c)

### Tasks

1. Create `userspace/levbox/src/bin/ln.rs`
2. Implement options: `-s`, `-f`, `--help`, `--version`
3. Use `symlinkat` for `-s` flag
4. Return error for hard links (not implemented)
5. Add to Cargo.toml

### Files
- `userspace/levbox/src/bin/ln.rs` (new)
- `userspace/levbox/Cargo.toml`

### Exit Criteria
- `ln -s target link` creates symbolic link
- `ln target link` returns "not implemented" error

---

## Verification

After each step:
1. Kernel compiles
2. Tests pass
3. Feature works in QEMU

---

## UoW Breakdown

If steps are too large, split:
- `phase-3-step-1-uow-1.md` — Update make_initramfs.sh
- `phase-3-step-3-uow-1.md` — Add syscall number
- `phase-3-step-3-uow-2.md` — Implement handler
- etc.
