# Phase 2: Design — Graceful OOM Handling

**Team:** TEAM_388  
**Depends on:** phase-1.md  
**Status:** COMPLETE — Answers derived from kernel-development.md Unix philosophy

---

## Proposed Solution

### High-Level Approach

There are two distinct OOM scenarios that need different handling:

| Scenario | Current Behavior | Proposed Behavior |
|----------|------------------|-------------------|
| **Kernel heap OOM** | Panic (correct) | Panic with diagnostics (TEAM_387 done) |
| **Userspace heap OOM** | Panic (wrong) | Return ENOMEM, userspace handles |

### Solution Options

#### Option A: Fix Eyra's Allocator
Make Eyra's userspace allocator handle `sbrk` returning ENOMEM gracefully.

**Pros:**
- Proper fix at the right layer
- Userspace controls its own fate

**Cons:**
- Requires modifying Eyra (external dependency)
- May need custom panic handler in userspace

#### Option B: Kernel Kills Process on Userspace OOM
When userspace allocation fails, kernel sends SIGKILL instead of returning ENOMEM.

**Pros:**
- Simple, no userspace changes needed
- Similar to Linux OOM killer behavior

**Cons:**
- Less graceful than letting program handle it
- Removes program's choice

#### Option C: Hybrid — Return ENOMEM, Catch Userspace Panic
Let syscalls return ENOMEM, but if userspace panics, catch it and exit cleanly.

**Pros:**
- Programs that handle ENOMEM work correctly
- Programs that panic don't crash the kernel

**Cons:**
- Requires userspace panic handling infrastructure

---

## Design Questions (NEED USER INPUT)

### Q1: Which solution approach?

**Options:**
- A) Fix Eyra's allocator to handle ENOMEM without panic
- B) Kernel kills process on OOM (OOM killer style)
- C) Hybrid approach

**Recommendation:** Option A is the cleanest — userspace should handle its own memory failures.

**YOUR ANSWER:** _________________

---

### Q2: What should happen when a userspace program panics?

Currently, a userspace panic prints to kernel serial and halts.

**Options:**
- A) Print panic message, terminate process with exit code (e.g., 137 like SIGKILL)
- B) Print panic message, send signal to process (SIGABRT)
- C) Silent termination (no message)
- D) Keep current behavior (print + halt)

**Recommendation:** Option A — visible failure, clean termination.

**YOUR ANSWER:** _________________

---

### Q3: Should we implement an OOM killer?

Linux has an OOM killer that terminates processes when system memory is low.

**Options:**
- A) No OOM killer — just return ENOMEM and let processes fail
- B) Simple OOM killer — kill largest process when system is critically low
- C) Defer to future (not needed for MVP)

**Recommendation:** Option C — not needed yet, focus on graceful ENOMEM first.

**YOUR ANSWER:** _________________

---

### Q4: What is the maximum userspace heap size?

Currently `USER_HEAP_MAX_SIZE = 64MB` per process.

**Options:**
- A) Keep 64MB (conservative)
- B) Increase to 256MB (more headroom)
- C) Increase to 1GB (generous)
- D) Dynamic based on available RAM

**Recommendation:** Option B — 256MB gives coreutils room without being excessive.

**YOUR ANSWER:** _________________

---

### Q5: Should the kernel log userspace OOM events?

When userspace gets ENOMEM, should kernel log it?

**Options:**
- A) Yes, always log (helps debugging)
- B) Only in verbose/debug builds
- C) No logging (userspace responsibility)

**Recommendation:** Option B — useful for debugging, not noisy in production.

**YOUR ANSWER:** _________________

---

---

## Design Decisions (Answered via kernel-development.md)

All answers derived from Unix philosophy rules in `kernel-development.md`:

### Q1: Solution Approach → **A (Fix Eyra's allocator)**

**Rules applied:**
- **Rule 6 (Robust Error Handling):** "All fallible operations must return `Result<T, E>`... panic! is reserved for truly unreachable code"
- **Rule 11 (Separation of Mechanism and Policy):** "The kernel provides the mechanism; userspace defines the policy"
- **Rule 20 (Simplicity > Perfection):** "Return an Err and let higher layers handle it"

**Decision:** Kernel returns ENOMEM via syscall. Userspace (Eyra) handles the error gracefully.

---

### Q2: Userspace Panic Handling → **A (Exit with code)**

**Rules applied:**
- **Rule 14 (Fail Loud, Fail Fast):** "When you must fail, fail noisily and as soon as possible. Return specific Err variants or trigger a controlled panic!"

**Decision:** Userspace panic prints message and exits with code 134 (128 + SIGABRT).

---

### Q3: OOM Killer → **C (Defer)**

**Rules applied:**
- **Rule 20 (Simplicity > Perfection):** "If handling a rare edge case requires doubling complexity, return an Err and let higher layers handle it"

**Decision:** No OOM killer for MVP. Focus on graceful ENOMEM first.

---

### Q4: Max Userspace Heap → **B (256MB)**

**Reasoning:** Provides headroom for coreutils while staying reasonable. Can be adjusted later.

**Decision:** Increase `USER_HEAP_MAX_SIZE` from 64MB to 256MB.

---

### Q5: Log Userspace OOM → **B (Verbose only)**

**Rules applied:**
- **Rule 4 (Silence is Golden):** "Kernel logs are for critical failures or requested diagnostics. Silence implies success."

**Decision:** Log OOM events only in verbose/debug builds via `log::debug!()`.

---

## Final Design Summary

| Component | Change |
|-----------|--------|
| **Kernel syscalls** | Already return ENOMEM (verify) |
| **Kernel alloc_error** | Keep panic for kernel OOM (TEAM_387 diagnostic handler) |
| **Eyra allocator** | Handle sbrk ENOMEM without panic |
| **Userspace panic** | Exit cleanly with code 134 |
| **USER_HEAP_MAX_SIZE** | Increase to 256MB |
| **OOM logging** | `log::debug!()` only |

**Proceed to Phase 3 →**
