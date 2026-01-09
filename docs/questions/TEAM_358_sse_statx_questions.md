# TEAM_358 — SSE/FPU & statx Questions

**Plan:** `docs/planning/userspace-sse-statx/`

---

## Q1: XSAVE vs FXSAVE for FPU State

**Question:** Should we use XSAVE (variable-size, supports AVX/AVX-512) or FXSAVE (fixed 512 bytes, SSE only)?

**Options:**
- **A) FXSAVE only** — Simpler, fixed 512 bytes, SSE/SSE2 support
- **B) XSAVE with CPUID** — Future-proof, but requires feature detection, variable buffer size
- **C) FXSAVE now, XSAVE later** — Pragmatic path

**My Recommendation:** Option A (FXSAVE) for MVP. XSAVE adds complexity and AVX isn't required for basic Eyra/PIE support.

**USER DECISION:** ✅ FXSAVE for MVP. Full Linux compatibility is the goal.

---

## Q2: Kernel FPU Usage

**Question:** Does our kernel build emit SSE instructions? If so, we need to handle kernel→user FPU state boundaries.

**Context:** 
- x86_64-unknown-none target typically disables SSE with `-C target-feature=-sse`
- Need to verify our toolchain settings

**If kernel uses SSE:** Must save/restore FPU state on syscall entry/exit
**If kernel doesn't use SSE:** Only need context switch save/restore

**My Recommendation:** Check `.cargo/config.toml` for target features. If SSE is disabled in kernel, this is a non-issue.

---

## Q3: Syscall Number for statx

**Question:** User reported syscall 302 causes ENOSYS, but Linux x86_64 `statx` is syscall 332. Which should we implement?

**Analysis:**
- Linux syscall 302 on x86_64 is `pkey_alloc` (memory protection keys)
- Linux syscall 332 on x86_64 is `statx`
- The binary might be calling the wrong syscall, or this is a different syscall

**My Recommendation:** 
1. Implement `statx` at correct number (332 x86_64 / 291 aarch64)
2. If 302 is actually being called, that's a different syscall (`pkey_alloc`)

**User Clarification Needed:** Can you confirm the exact syscall number from strace or crash logs?

---

## Q4: Extended statx Fields

**Question:** Which optional fields in `struct statx` should we populate?

**Fields in question:**
| Field | Description | Can We Populate? |
|-------|-------------|------------------|
| `stx_btime` | Birth/creation time | ❌ VFS doesn't track |
| `stx_mnt_id` | Mount point ID | ❌ Not implemented |
| `stx_attributes` | Immutable/append-only flags | ❌ Not supported |
| `stx_dio_*` | Direct I/O alignment | ❌ No direct I/O |

**My Recommendation:** Return 0 for all extended fields. Set `stx_mask` to indicate only basic fields are valid (STATX_BASIC_STATS).

---

## Summary

| Question | Default Answer | Needs User Input? |
|----------|---------------|-------------------|
| Q1: XSAVE vs FXSAVE | FXSAVE only | ⚠️ Confirm OK |
| Q2: Kernel FPU | Check config | No |
| Q3: Syscall number | 332 (statx) | ⚠️ Clarify 302 |
| Q4: Extended fields | Return zeros | No |

---

**Q1 RESOLVED:** FXSAVE for MVP  
**Q3:** Implementing statx at correct Linux numbers (332/291). If 302 is needed, that's `pkey_alloc` — separate feature.
