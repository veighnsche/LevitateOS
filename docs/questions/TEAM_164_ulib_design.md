# TEAM_164: ulib Design Questions

**Feature:** Phase 10 - Userspace Standard Library (`ulib`)  
**Created:** 2026-01-06  
**Status:** ANSWERED (TEAM_165 per kernel-development.md guidelines)

---

## Summary

These questions must be answered before Phase 3 (Implementation) can begin.

---

## Q1: Heap Initial Size and Growth

**Context:** The allocator needs to know how much heap to start with and how to grow.

**Options:**
- **A) Start with 0, grow by 4KB on first allocation** *(Recommended)*
  - Pros: Minimal memory waste, lazy allocation
  - Cons: First allocation incurs syscall

- **B) Start with 64KB, grow by 64KB**
  - Pros: Good for apps that allocate a lot upfront
  - Cons: Wastes memory for small apps

- **C) Start with 4KB, double on exhaustion**
  - Pros: Balanced, exponential growth reduces syscalls
  - Cons: More complex growth logic

**Your choice:** **A**

**Rationale (TEAM_165):** Per **Rule 20 (Simplicity)** - "Implementation simplicity is the highest priority." Lazy allocation (start with 0) is the simplest approach. Also aligns with **Rule 16 (Energy Awareness)** - don't allocate resources until needed.

---

## Q2: File Descriptor Allocation Strategy

**Context:** When `open()` is called, how do we pick the fd number?

**Options:**
- **A) Always use lowest available** *(Recommended, POSIX behavior)*
  - Pros: POSIX compliant, expected by shell scripts
  - Cons: Requires scanning table

- **B) Incrementing counter**
  - Pros: Simple, O(1)
  - Cons: fd numbers grow unbounded

- **C) Random**
  - Pros: Security benefit
  - Cons: Harder to debug, non-standard

**Your choice:** **A**

**Rationale (TEAM_165):** Per **Rule 18 (Least Surprise)** - "Always do the least surprising thing." POSIX-compliant lowest-available is what developers expect. Shell scripts and standard tools rely on this behavior.

---

## Q3: What Happens When Heap Exhausted?

**Context:** If `sbrk` cannot grow the heap (e.g., hit limit), what should happen?

**Options:**
- **A) Return null pointer (let allocator handle OOM)** *(Recommended)*
  - Pros: Standard behavior, allocator can panic or handle
  - Cons: Silent failure if not handled

- **B) Panic the process**
  - Pros: Immediate visibility
  - Cons: No recovery possible

- **C) Block until memory available**
  - Pros: Allows waiting for other processes to exit
  - Cons: Complex, can cause deadlocks

**Your choice:** **A**

**Rationale (TEAM_165):** Per **Rule 14 (Fail Fast)** - "When you must fail, fail noisily and as soon as possible." Returning null allows the allocator to decide how to fail (panic or handle gracefully). Per **Rule 6 (Robust Error Handling)** - use `Result<T, E>` pattern; the allocator wraps this.

---

## Q4: Read-Only Initramfs vs Writable Files

**Context:** When opening a file from initramfs, can it be written to?

**Options:**
- **A) Read-only only** *(Recommended)*
  - Pros: Matches initramfs semantics, simple
  - Cons: Can't modify files

- **B) Copy-on-write into heap**
  - Pros: Allows modification
  - Cons: Complex, memory usage

- **C) Defer until real filesystem**
  - Pros: Cleaner design
  - Cons: No write support for a while

**Your choice:** **A**

**Rationale (TEAM_165):** Per **Rule 20 (Simplicity)** - simplest implementation. Initramfs IS read-only by definition. Per **Rule 11 (Separation)** - kernel provides mechanism (read from archive), policy (writable files) deferred to real filesystem.

---

## Q5: Argument Passing Mechanism

**Context:** How should `argc`, `argv`, `envp` be passed to `_start`?

**Options:**
- **A) Stack-based (Linux ABI compatible)** *(Recommended)*
  - Pros: Standard, enables future libc compat
  - Cons: More complex ELF loader

- **B) Special syscall `getargs()`**
  - Pros: Simple userspace startup
  - Cons: Non-standard

- **C) Memory region at fixed address**
  - Pros: Simple to implement
  - Cons: Wastes address space, fragile

**Your choice:** **A**

**Rationale (TEAM_165):** Per **Rule 18 (Least Surprise)** and **Rule 2 (Type-Driven Composition)** - use standard interfaces that are consumable by other subsystems. Stack-based argc/argv is the universal convention. Enables future libc compatibility.

---

## Q6: Sleep Implementation

**Context:** How should `nanosleep` / `sleep()` work?

**Options:**
- **A) Busy loop with yield** (MVP fallback)
  - Pros: Works now, no scheduler changes
  - Cons: CPU waste

- **B) Timer-based wakeup** *(Recommended for proper impl)*
  - Pros: Efficient, proper OS behavior
  - Cons: Requires scheduler timer queue

- **C) Hybrid (yield with timeout check)**
  - Pros: Better than pure busy-wait
  - Cons: Still polls

**Your choice:** **B**

**Rationale (TEAM_165):** Per **Rule 16 (Energy Awareness)** - "Race to Sleep: execute tasks efficiently to return to low-power states." Busy-wait wastes CPU cycles. Timer-based wakeup is the correct kernel behavior. Per **Rule 9 (Non-blocking Design)** - avoid blocking execution flow with busy loops.

---

## Q7: Error Code Compatibility

**Context:** Should syscall error codes match Linux errno values?

**Options:**
- **A) Use Linux errno values** *(Recommended)*
  - Pros: Easier future compatibility, matches spec
  - Cons: Requires remapping existing codes

- **B) Continue custom error codes**
  - Pros: No migration needed
  - Cons: Diverges from standards

- **C) Map at library boundary**
  - Pros: Kernel stays simple, library handles compat
  - Cons: Two error code systems

**Your choice:** **A**

**Rationale (TEAM_165):** Per **Rule 18 (Least Surprise)** - standard Linux errno values are what developers expect. Per **Rule 3 (Expressive Interfaces)** - use well-known constants for type-safe error handling. Enables future libc/std compatibility.

---

## How to Answer

Please edit this file directly and fill in "Your choice:" with A, B, or C for each question. Add any notes or concerns below each question if needed.

Once all questions are answered, a team can proceed to Phase 3 implementation.
