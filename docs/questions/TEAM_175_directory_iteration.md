# TEAM_175: Directory Iteration Design Questions

**Feature:** Directory Iteration (`ReadDir`) for ulib Phase 10  
**Planning Doc:** `docs/planning/directory-iteration/phase-2.md`  
**Status:** Awaiting user decisions

---

## Questions Requiring Decisions

### Q1: ReadDir Internal Buffer Size

What should be the default buffer size for `ReadDir`?

| Option | Description |
|--------|-------------|
| **A** | 256 bytes (minimal, frequent syscalls) |
| **B** | 1024 bytes (balance) |
| **C** | 4096 bytes (one page, efficient) ⭐ Recommended |
| **D** | Configurable via `ReadDir::with_capacity()` |

**Recommendation:** C (4096 bytes) - one page handles most directories in single syscall.

---

### Q2: Handling "." and ".." Entries

Should `ReadDir` include `.` (current dir) and `..` (parent dir) entries?

| Option | Description |
|--------|-------------|
| **A** | Include them (matches raw syscall, POSIX behavior) |
| **B** | Filter them out (matches Rust `std::fs::read_dir`) ⭐ Recommended |
| **C** | Make configurable |

**Recommendation:** B - Rust std behavior filters them, less surprise for Rust developers.

---

### Q3: Error on Non-Directory

What error when user calls `read_dir()` on a regular file?

| Option | Description |
|--------|-------------|
| **A** | `ErrorKind::NotADirectory` (new error kind) ⭐ Recommended |
| **B** | `ErrorKind::InvalidArgument` (generic) |
| **C** | `ErrorKind::NotFound` (misleading but simple) |

**Recommendation:** A - explicit error is clearer for debugging.

---

### Q4: Empty Directory Behavior

How should empty directories behave?

| Option | Description |
|--------|-------------|
| **A** | Return iterator that immediately yields `None` |
| **B** | Return `Ok(ReadDir)` where first `next()` returns `None` ⭐ Recommended |
| **C** | Return error (empty dir is "not found") |

**Recommendation:** B - consistent with non-empty directories, empty is valid state.

---

### Q5: Initramfs Directory Implementation

How should we implement directory support in initramfs?

**Context:** CPIO format has `c_mode` field encoding file type, but current `CpioArchive` API lacks directory helpers.

| Option | Description |
|--------|-------------|
| **A** | Enhance `CpioArchive` with `is_directory()` and `list_directory()` ⭐ Recommended |
| **B** | Implement directory support only in kernel syscall layer |
| **C** | Flatten everything - no real directories, just path prefixes |

**Recommendation:** A - add helpers to CpioArchive, reusable and testable in `los_utils`.

---

### Q6: Syscall Number Assignment

Should we use syscall number 14 (next sequential) or align with Linux?

| Option | Description |
|--------|-------------|
| **A** | Use NR 14 (next sequential in our table) ⭐ Recommended |
| **B** | Use NR 61 (Linux `getdents64` number) |
| **C** | Use custom high number to avoid future conflicts |

**Recommendation:** A - our syscall numbers are already custom, consistency is better than partial alignment.

---

### Q7: DirEntry Path vs Name

Should `DirEntry` store full path or just filename?

| Option | Description |
|--------|-------------|
| **A** | Filename only (what kernel returns) ⭐ Recommended |
| **B** | Full path (convenience, matches std::fs) |
| **C** | Both (flexible) |

**Recommendation:** A for MVP - avoids allocation and path construction. Can add `path()` method later.

---

## Summary Table

| ID | Question | Recommended | Your Decision |
|----|----------|-------------|---------------|
| Q1 | Buffer size | C (4096) | ✅ C |
| Q2 | Include . and .. | B (filter) | ✅ B |
| Q3 | Non-dir error | A (NotADirectory) | ✅ A |
| Q4 | Empty dir | B (Ok + None) | ✅ B |
| Q5 | Initramfs impl | A (CpioArchive) | ✅ A |
| Q6 | Syscall number | A (NR 14) | ✅ A |
| Q7 | Entry name/path | A (name only) | ✅ A |

**Status:** ✅ All decisions made (2026-01-06) - Proceeding to implementation (TEAM_176)

---

## To Proceed

Please review the questions above and provide your decisions. You can:
1. Accept recommendations: "Accept all recommendations"
2. Override specific questions: "Q2: A, Q7: B"
3. Ask for clarification on any question

Once decisions are made, TEAM_175 will update phase-2.md and proceed to implementation planning.
