# TEAM_177: File Read Implementation Questions

**Feature:** File Read (`File::read()`) for ulib Phase 10  
**Planning Doc:** `docs/planning/file-read/phase-2.md`  
**Status:** Awaiting user decisions

---

## Questions Requiring Decisions

### Q1: EOF Behavior

What should happen when reading at or past end of file?

| Option | Description |
|--------|-------------|
| **A** | Return 0 (standard POSIX behavior) ⭐ Recommended |
| **B** | Return error (EINVAL or custom) |

**Recommendation:** A - Standard POSIX. Return 0 signals EOF.

---

### Q2: Partial Read Behavior

If user requests more bytes than available (e.g., 1000 bytes from 100-byte file)?

| Option | Description |
|--------|-------------|
| **A** | Read available bytes, return actual count ⭐ Recommended |
| **B** | Return error if can't satisfy full request |

**Recommendation:** A - Standard behavior. Return what's available.

---

### Q3: Read on Stdout/Stderr

What if user tries to `read()` from fd 1 (stdout) or fd 2 (stderr)?

| Option | Description |
|--------|-------------|
| **A** | Return EBADF (bad file descriptor) ⭐ Recommended |
| **B** | Return 0 (EOF) |

**Recommendation:** A - stdout/stderr are write-only.

---

### Q4: Read on Directory Fd

What if user tries to `read()` on a directory fd?

| Option | Description |
|--------|-------------|
| **A** | Return EISDIR (new error code for "is a directory") |
| **B** | Return EBADF ⭐ Recommended |

**Recommendation:** B - Simpler, directory fds are for getdents only.

---

### Q5: Maximum Single Read Size

Should there be a max bytes per `read()` call?

| Option | Description |
|--------|-------------|
| **A** | No limit (read entire file if requested) ⭐ Recommended |
| **B** | Limit to 4KB |

**Recommendation:** A - Files are bounded by file size, no need for artificial limit.

---

### Q6: Concurrent Reads

Two processes open same file and read concurrently. Any issues?

| Option | Description |
|--------|-------------|
| **A** | Works fine - each fd has independent offset ⭐ Recommended |
| **B** | Needs additional locking |

**Recommendation:** A - Each fd has its own offset. INITRAMFS is read-only.

---

## Summary Table

| ID | Question | Recommended | Your Decision |
|----|----------|-------------|---------------|
| Q1 | EOF behavior | A (return 0) | ✅ A |
| Q2 | Partial read | A (return partial) | ✅ A |
| Q3 | Read stdout/stderr | A (EBADF) | ✅ A |
| Q4 | Read directory | B (EBADF) | ✅ B |
| Q5 | Max read size | A (no limit) | ✅ A |
| Q6 | Concurrent reads | A (works fine) | ✅ A |

**Status:** ✅ All decisions made (2026-01-06) - Proceeding to implementation (TEAM_178)

---

## Implementation Note

This is a **low-complexity feature** (~88 lines total):
- Infrastructure already exists (fd_table with offset tracking)
- Pattern established by sys_getdents
- Can be implemented in single session after questions answered

---

## To Proceed

Please review and provide decisions:
- **"Accept all recommendations"** to proceed quickly
- **"Q3: B, Q5: B"** to override specific questions
