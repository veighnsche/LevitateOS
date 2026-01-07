# TEAM_179: Buffered I/O Design Questions

**Feature:** Buffered I/O (`BufReader`, `BufWriter`) for ulib Phase 10  
**Planning Doc:** `docs/planning/buffered-io/phase-2.md`  
**Status:** Awaiting user decisions

---

## Questions Requiring Decisions

### Q1: Default Buffer Size

What should the default buffer capacity be?

| Option | Description |
|--------|-------------|
| **A** | 1 KB (1024 bytes) - Conservative |
| **B** | 4 KB (4096 bytes) - One page |
| **C** | 8 KB (8192 bytes) - std::io default ⭐ Recommended |

**Recommendation:** C - Matches Rust std, good balance of memory vs syscall reduction.

---

### Q2: Partial Buffer Read Behavior

When BufReader's internal buffer has data but user requests more, what to do?

| Option | Description |
|--------|-------------|
| **A** | Return buffered data immediately, don't refill ⭐ Recommended |
| **B** | Refill buffer first, then return up to requested amount |

**Recommendation:** A - Standard behavior. Return what's available.

---

### Q3: BufWriter Flush Trigger

When should BufWriter automatically flush?

| Option | Description |
|--------|-------------|
| **A** | Only when buffer is completely full ⭐ Recommended |
| **B** | When buffer reaches 75% capacity |

**Recommendation:** A - Standard behavior. Flush when full or explicitly requested.

---

### Q4: Drop Error Handling

If BufWriter flush fails during Drop, what to do?

| Option | Description |
|--------|-------------|
| **A** | Silently ignore error ⭐ Recommended |
| **B** | Panic |

**Recommendation:** A - Can't propagate errors from Drop. Users should flush explicitly if they care.

---

### Q5: read_line Newline Handling

Should `read_line()` include the trailing newline?

| Option | Description |
|--------|-------------|
| **A** | Yes, include newline (matches std) ⭐ Recommended |
| **B** | No, strip newline |

**Recommendation:** A - Match std::io::BufRead behavior.

---

### Q6: Binary File read_line

What if file contains no newlines (reading binary)?

| Option | Description |
|--------|-------------|
| **A** | Read entire file (could OOM!) |
| **B** | Read up to buffer size, return available ⭐ Recommended |

**Recommendation:** B - Safe default. Users shouldn't use read_line on binary.

---

### Q7: read_line String Clearing

Should `read_line()` clear the string before appending?

| Option | Description |
|--------|-------------|
| **A** | No, append to existing (std behavior) ⭐ Recommended |
| **B** | Yes, clear first |

**Recommendation:** A - Match std. Users clear explicitly when needed.

---

### Q8: BufWriter write() Return

What should `write()` return?

| Option | Description |
|--------|-------------|
| **A** | Bytes accepted into buffer ⭐ Recommended |
| **B** | Always return full input length |

**Recommendation:** A - Standard behavior. May be less than requested if buffer fills.

---

## Summary Table

| ID | Question | Recommended | Your Decision |
|----|----------|-------------|---------------|
| Q1 | Buffer size | C (8 KB) | ✅ C |
| Q2 | Partial read | A (return available) | ✅ A |
| Q3 | Flush trigger | A (when full) | ✅ A |
| Q4 | Drop error | A (ignore) | ✅ A |
| Q5 | Include newline | A (yes) | ✅ A |
| Q6 | Binary read_line | B (up to buffer) | ✅ B |
| Q7 | Clear string | A (no, append) | ✅ A |
| Q8 | write() return | A (bytes buffered) | ✅ A |

**Status:** ✅ All decisions made (2026-01-06) - Proceeding to implementation (TEAM_180)

---

## To Proceed

Please review and provide decisions:
- **"Accept all recommendations"** to proceed quickly
- **"Q1: B, Q5: B"** to override specific questions
