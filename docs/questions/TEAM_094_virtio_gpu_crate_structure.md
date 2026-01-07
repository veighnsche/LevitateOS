# Questions: VirtIO GPU Crate Structure

**Team:** TEAM_094  
**Date:** 2026-01-05  
**Related Plan:** `docs/planning/virtio-gpu-scanout/`

---

## Q1: Complete Replacement vs. Wrapper?

Should we completely replace `virtio-drivers` with a custom implementation, or create a wrapper that adds debugging on top?

**Options:**
- **A) Complete replacement** — Build protocol structs and driver from scratch (like Tock)
- **B) Wrapper with hooks** — Keep virtio-drivers, add logging wrapper around it

**Recommendation:** Option A — virtio-drivers uses blocking calls and hides too much state

**User Answer:** A

---

## Q2: Crate Naming

What should the new crates be named?

**Options:**
- **A) `levitate-virtio` + `levitate-virtio-gpu`** — Clear separation, general VirtIO base
- **B) Extend `levitate-gpu`** — Keep existing crate, add protocol module inside
- **C) `levitate-virtio-protocol` + `levitate-gpu`** — Protocol separate from driver

**Recommendation:** Option A — matches Tock's structure, allows future VirtIO device drivers

**User Answer:** A, ALright but remember that the END GOAL is running this on the Pixel 6 oriole, so we have several GPU drivers to implement

---

## Q3: HAL Trait Compatibility

Should the new driver implement `virtio-drivers`'s HAL traits, or define new ones?

**Options:**
- **A) Define new traits** — Clean break, full control
- **B) Implement existing HAL traits** — Easier migration, but couples to virtio-drivers patterns

**Recommendation:** Option A — we already have `levitate-hal::VirtioHal`, just expand it

**User Answer:** A

---

## Q4: Async vs. Blocking

Should the new driver support async operations?

**Options:**
- **A) Blocking only** — Simpler, matches current behavior
- **B) Async-first** — Better for future, but more complex
- **C) Dual API** — Both blocking and async wrappers

**Recommendation:** Option A for now — focus on debugging first, async later

**User Answer:** B, DO IT RIGHT FROM THE START!!! NO MORE SIMPLER IMPLEMTATIONS THAT INTRODUCES OTHER BUGS!

---

## Blocking Status

These questions do NOT block Phase 1 (protocol struct definition). They affect Phase 3 (migration) and Phase 5 (cleanup).

---

## TEAM_095 Review Notes

### Tock Reference Verification Issue

TEAM_094 cited Tock's VirtIO GPU implementation as a reference. However, the directory `.external-kernels/tock/chips/virtio/src/devices/virtio_gpu/` is **empty** (0 items).

**Options:**
1. Tock submodule may be incomplete — run `git submodule update --init --recursive`
2. Tock VirtIO GPU may be in a different location
3. Use alternative reference (e.g., VirtIO spec directly, QEMU source)

**Recommendation:** Proceed with VirtIO 1.1 spec as primary reference. Tock patterns are still valid conceptually even if code is unavailable.

