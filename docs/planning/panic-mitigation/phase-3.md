# Phase 3: Filesystem Safety

**Status**: Ready for execution
**TEAM**: TEAM_415
**Dependencies**: Phase 2

---

## Purpose

Fix `Tmpfs::root()` to return `Option` or `Result` instead of panicking when called before initialization.

---

## Target Design

### Current (Unsafe)

```rust
// src/fs/tmpfs/superblock.rs:113
impl Tmpfs {
    pub fn root(&self) -> Arc<Inode> {
        self.vfs_root.as_ref().expect("Tmpfs::root called before vfs_root was initialized").clone()
    }
}
```

### New (Safe)

```rust
impl Tmpfs {
    /// Returns the root inode, or None if not yet initialized.
    pub fn root(&self) -> Option<Arc<Inode>> {
        self.vfs_root.clone()
    }
}
```

---

## Migration Strategy

### Breaking Change (Rule 5)

Per Rule 5, prefer clean breaks over compatibility hacks:

1. Change `Tmpfs::root()` signature to return `Option<Arc<Inode>>`
2. Let compiler find all call sites
3. Fix each call site to handle `None` appropriately
4. Remove old API

### Call Sites to Migrate

Search for all uses of `Tmpfs::root()` and update them to:
- Use `?` operator if in a function returning `Result`/`Option`
- Use `.expect("context")` if truly impossible to be None
- Use match/if-let for conditional handling

---

## Steps

### Step 1: Change Tmpfs::root() Signature

**File**: `phase-3-step-1.md`

1. Modify `src/fs/tmpfs/superblock.rs`
2. Change `root()` to return `Option<Arc<Inode>>`
3. Compile - collect all errors

**UoW size**: 1 file change + error collection.

### Step 2: Migrate Call Sites

**File**: `phase-3-step-2.md`

Fix all call sites identified by compiler errors.

**UoW size**: Depends on number of call sites.

---

## Exit Criteria

- [ ] `Tmpfs::root()` returns `Option`
- [ ] All call sites migrated
- [ ] Build passes
- [ ] No panics in filesystem code paths
