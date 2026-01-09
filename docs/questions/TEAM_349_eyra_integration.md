# Questions: Eyra Integration Prerequisites

**TEAM_349** | 2026-01-09  
**Answered:** 2026-01-09 (based on kernel-development.md rules)

---

## Q1: Random Number Generation Strategy

**Context:** `getrandom` syscall is required for `std` (HashMap seeds, etc.)

**Options:**
- A) Hardware RNG only (`RNDR` on ARM, `RDRAND` on x86)
- B) PRNG seeded from boot time + memory layout
- C) Hardware if available, fallback to PRNG

**Answer:** **Option C** (hardware with PRNG fallback)

**Rationale (Rules Applied):**
- **Rule 6 (Robustness):** All fallible operations must be handled explicitly. Hardware RNG may not be available on all platforms, so we need a fallback.
- **Rule 17 (Resilience):** Design for transient failures — hardware RNG can fail.
- **Rule 19 (Diversity):** Support heterogeneous hardware; not all CPUs have `RNDR`/`RDRAND`.
- **Rule 20 (Simplicity):** Option C is only slightly more complex than A, but much more robust.

**Implementation:**
```rust
pub fn sys_getrandom(buf: usize, len: usize, flags: u32) -> i64 {
    // Try hardware first, fallback to PRNG
    if let Some(hw_rng) = HardwareRng::try_get() {
        hw_rng.fill(buf, len)
    } else {
        KERNEL_PRNG.fill(buf, len)
    }
}
```

---

## Q2: clone3 vs clone

**Context:** Origin (Eyra's thread startup) may use either `clone` or `clone3`

**Answer:** **Start with `clone` only**

**Rationale (Rules Applied):**
- **Rule 20 (Simplicity > Perfection):** `clone` is sufficient for thread creation. Don't implement `clone3` unless proven necessary.
- **Rule 21 (Economy):** Programmer time > machine time. Verify Origin uses `clone` before implementing `clone3`.
- **Rule 6 (Robustness):** If Origin needs `clone3`, return `ENOSYS` and let it fall back to `clone`.

**Action:** Check Origin source code. If `clone3` is required, implement it. Otherwise, defer.

---

## Q3: /proc/self/exe Support

**Context:** `std::env::current_exe()` reads `/proc/self/exe` on Linux

**Answer:** **Option B** — Return error (allowed by spec)

**Rationale (Rules Applied):**
- **Rule 20 (Simplicity):** Implementing procfs is significant work. If handling a rare edge case requires doubling complexity, return an `Err`.
- **Rule 11 (Separation):** The kernel provides mechanism, not policy. `current_exe()` is a convenience, not a necessity.
- **Rule 1 (Modularity):** procfs is a separate subsystem. Don't implement it just for one feature.

**Implementation:** `current_exe()` returns `Err(io::Error::new(ErrorKind::NotFound, "not supported"))`

**Future:** If procfs is needed for other reasons, implement `/proc/self/exe` then.

---

## Q4: x86_64 TLS via arch_prctl vs clone

**Context:** x86_64 can set TLS base via `arch_prctl` or `clone` with `CLONE_SETTLS`

**Answer:** **Implement both**

**Rationale (Rules Applied):**
- **Rule 6 (Robustness):** Main thread needs `arch_prctl`; spawned threads use `clone` + `CLONE_SETTLS`. Both are required for complete TLS support.
- **Rule 2 (Composition):** These are orthogonal mechanisms that work together. Clean interface between them.
- **Rule 20 (Simplicity):** `arch_prctl` is ~20 lines of code. Not complex.

**Implementation:**
```rust
pub fn sys_arch_prctl(code: i32, addr: usize) -> i64 {
    match code {
        ARCH_SET_FS => { /* wrmsr(IA32_FS_BASE, addr) */ 0 }
        ARCH_GET_FS => { /* read and write to addr */ 0 }
        _ => errno::EINVAL
    }
}
```

---

## Q5: Signal Delivery Scope

**Context:** `tgkill` sends signals to specific threads

**Answer:** **Immediate delivery, no per-thread queues**

**Rationale (Rules Applied):**
- **Rule 20 (Simplicity):** Per-thread signal queues add ~200 lines of complexity. Start simple.
- **Rule 14 (Fail Fast):** If a signal can't be delivered immediately, fail noisily rather than queue silently.
- **Rule 17 (Resilience):** Simple immediate delivery is easier to reason about and debug.

**Implementation:** Deliver signal to thread's pending mask immediately. If thread is blocked, wake it.

**Future:** If real-time signal queuing is needed, add it as a separate enhancement.

---

## Q6: fcntl Operations Needed

**Context:** Eyra may use `fcntl` for various operations

**Answer:** **Implement minimal set: F_GETFD, F_SETFD, F_GETFL, F_SETFL**

**Rationale (Rules Applied):**
- **Rule 20 (Simplicity):** Implement only what's needed. `F_DUPFD` can return `ENOSYS` (use `dup3` instead).
- **Rule 6 (Robustness):** Return proper errors for unsupported operations rather than silently failing.
- **Rule 3 (Expressive Interfaces):** Use enums for fcntl commands, not magic numbers.

**Implementation:**
```rust
pub fn sys_fcntl(fd: i32, cmd: i32, arg: usize) -> i64 {
    match cmd {
        F_GETFD => file.flags.cloexec as i64,
        F_SETFD => { file.flags.cloexec = arg != 0; 0 }
        F_GETFL => file.status_flags as i64,
        F_SETFL => { file.status_flags = arg as u32; 0 }
        _ => errno::EINVAL
    }
}
```

---

## Q7: mmap Fixed Address Support

**Context:** `MAP_FIXED` flag requests mapping at exact address

**Answer:** **Implement MAP_FIXED**

**Rationale (Rules Applied):**
- **Rule 6 (Robustness):** Thread stacks and TLS often require fixed addresses. Without it, threading may fail in subtle ways.
- **Rule 8 (Least Privilege):** `MAP_FIXED` is a mechanism; userspace decides when to use it.
- **Rule 20 (Simplicity):** `MAP_FIXED` is ~10 lines extra in mmap. Skip the hint search, just map at the requested address.

**Implementation:**
```rust
if flags & MAP_FIXED != 0 {
    // Map at exact address, fail if occupied
    if !is_region_free(addr, len) {
        return errno::ENOMEM;
    }
    map_at(addr, len, prot)
} else {
    // Hint-based: find free region near addr
    find_and_map(addr, len, prot)
}
```

---

## Summary of Answers

| Question | Answer | Complexity |
|----------|--------|------------|
| Q1: getrandom | Hardware + PRNG fallback | Medium |
| Q2: clone3 | Defer (clone only) | None |
| Q3: /proc/self/exe | Return error | None |
| Q4: arch_prctl | Implement (x86_64) | Small |
| Q5: Signal queues | Immediate delivery | None |
| Q6: fcntl | F_GETFD/SETFD/GETFL/SETFL | Small |
| Q7: MAP_FIXED | Implement | Small |

**Guiding Principle:** Rule 20 (Simplicity > Perfection) — Start with the minimal implementation that works, extend only when proven necessary.
