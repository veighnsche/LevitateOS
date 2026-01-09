# TEAM_359 — Eyra Syscalls Questions

**Plan:** `docs/planning/eyra-syscalls/`

---

## Q1: ppoll Blocking Behavior

**Question:** Should `ppoll` block waiting for events, or return immediately?

**Context:** Eyra calls ppoll during initialization. A non-blocking implementation may work, but true std I/O needs blocking.

**Options:**
- **A) Non-blocking** — Always return immediately with current fd state
- **B) Blocking with timeout** — Implement proper sleep/wake mechanism
- **C) Start non-blocking, enhance later** — Pragmatic MVP

**My Recommendation:** Option C — Get Eyra working first, then enhance if needed.

---

## Q2: tkill vs tgkill

**Question:** Should we implement `tkill` only, or also `tgkill`?

**Context:**
- `tkill(tid, sig)` — Send signal to thread (legacy)
- `tgkill(tgid, tid, sig)` — Send signal to thread in specific process (more secure)

**Options:**
- **A) tkill only** — Simpler, handles current Eyra needs
- **B) Both** — Future-proof for other libc implementations

**My Recommendation:** Option A for now, add tgkill when needed.

---

## Q3: pkey_alloc Return Value

**Question:** What should `pkey_alloc` return?

**Options:**
- **A) -ENOSYS** — Syscall not implemented
- **B) -EOPNOTSUPP** — Operation not supported (more accurate)
- **C) -EINVAL** — Invalid argument (if flags != 0)

**My Recommendation:** Option A (-ENOSYS) — Clear signal that MPK not available.

---

## Summary

| Question | Default Answer | Needs User Input? |
|----------|---------------|-------------------|
| Q1: ppoll blocking | Start non-blocking | ⚠️ Confirm OK |
| Q2: tkill vs tgkill | tkill only | No |
| Q3: pkey_alloc return | -ENOSYS | No |

---

**Please confirm Q1** before Phase 3 implementation begins.
