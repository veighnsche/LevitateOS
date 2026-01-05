# Phase 4 — Cleanup

**Parent:** Userspace Architecture Refactor  
**File:** `docs/planning/userspace-refactor/phase-4.md`  
**Depends On:** Phase 3 complete

---

## 1. Dead Code Removal (Rule 6)

### 1.1 Files to Delete

| Path | Reason |
|------|--------|
| `userspace/hello/` | Duplicate shell, replaced by `init` + `shell` |
| `initrd_root/hello` | Old binary, replaced by `init` |

### 1.2 Validation Before Deletion

Before deleting `userspace/hello/`:
- [ ] `cargo xtask test all` passes with new structure
- [ ] VNC visual verification works
- [ ] No references to "hello" in kernel (grep confirms)

---

## 2. Temporary Adapter Removal

None expected — this refactor doesn't use adapters.

If any shims were added during Phase 2/3, remove them here.

---

## 3. Encapsulation Tightening

### 3.1 libsyscall Visibility

```rust
// libsyscall/src/lib.rs

// Public API
pub mod syscall;       // read, write, exit, getpid, sbrk
pub mod print;         // print!, println! macros
pub use syscall::*;    // Re-export for convenience

// Internal (not pub)
mod errno;             // Error constants (if separated)
```

### 3.2 Shell Module Cleanup

- Remove any `pub` visibility that's not needed
- Ensure `execute()`, `read_line()`, etc. are private

---

## 4. File Size Check (Rule 7)

Target: <500 lines per file (ideal), <1000 lines (acceptable)

| File | Expected Lines | Status |
|------|----------------|--------|
| `libsyscall/src/lib.rs` | ~100 | ✅ |
| `init/src/main.rs` | ~30 | ✅ |
| `shell/src/main.rs` | ~150 | ✅ |

---

## 5. Phase 4 Steps

### Step 1: Delete userspace/hello
```bash
rm -rf userspace/hello
```

### Step 2: Clean initrd_root
```bash
rm -f initrd_root/hello
# Keep: init, shell, hello.txt, test.txt
```

### Step 3: Update .gitignore if Needed
Ensure `userspace/target/` is ignored (it should be).

### Step 4: Final Grep for Dead References
```bash
grep -r "userspace/hello" .
grep -r '"hello"' kernel/  # Should find only comments/docs
```

---

## 6. Exit Criteria

- [ ] `userspace/hello/` deleted
- [ ] No references to old binary in kernel
- [ ] All tests pass
- [ ] File sizes within limits
- [ ] Build is clean (no warnings about unused code)
