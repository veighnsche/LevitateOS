# TEAM_095 Log: Review of virtio-gpu-scanout Plan

**Date:** 2026-01-05  
**Role:** Plan Reviewer (Second Pass)  
**Target:** `docs/planning/virtio-gpu-scanout/`

---

## Review Summary

**STATUS:** 🔍 IN PROGRESS

This is a second-pass review building on TEAM_094's initial review. Focus areas:
1. Verify user answers are reflected in plan
2. Check for async-first requirement (Q4 answer: "B, DO IT RIGHT FROM THE START!")
3. Validate scope against Pixel 6 end-goal context

---

## Phase 1 — Questions and Answers Audit

### Questions File: `.questions/TEAM_094_virtio_gpu_crate_structure.md`

| Q# | Question | User Answer | Reflected in Plan? |
|----|----------|-------------|-------------------|
| Q1 | Complete Replacement vs Wrapper? | **A** (Complete replacement) | ✅ Yes - Plan builds from scratch |
| Q2 | Crate Naming? | **A** (`levitate-virtio` + `levitate-virtio-gpu`) + context: "END GOAL is Pixel 6" | ⚠️ Partial - Crate names correct, but Pixel 6 context missing |
| Q3 | HAL Trait Compatibility? | **A** (Define new traits) | ✅ Yes - Plan mentions expanding `levitate-hal::VirtioHal` |
| Q4 | Async vs Blocking? | **B** ("DO IT RIGHT FROM THE START!!! NO MORE SIMPLER IMPLEMENTATIONS") | ❌ **NOT REFLECTED** - Plan Phase 3 says "blocking only" |

### Critical Finding: Q4 Async Requirement Not Reflected

The user explicitly requested async-first design. The current plan phases do not mention async:
- Phase 2: No mention of async command queue
- Phase 3: "Command Queue Manager" - no async specification
- Phase 4: "Health Check Loop" - could be async but not specified

**Action Required:** Update plan to specify async-first command handling.

---

## Phase 2 — Scope and Complexity Check

### Metrics

| Metric | Value | Assessment |
|--------|-------|------------|
| Phases | 5 | ✅ Appropriate for refactor scope |
| Steps per Phase | 2-3 | ✅ SLM-sized |
| New Crates | 2 | ✅ Matches user answer |
| Total UoWs | ~12 | ✅ Manageable |

### Overengineering Concerns

| Concern | Status | Notes |
|---------|--------|-------|
| Too many phases? | ✅ No | 5 phases is appropriate |
| Unnecessary abstractions? | ✅ No | Protocol structs are justified |
| Premature optimization? | ✅ No | Focus is correctness/debugging |
| Speculative features? | ⚠️ Maybe | "Scanout Health Check Loop" (Phase 4) - is this needed before basic works? |

### Oversimplification Concerns

| Concern | Status | Notes |
|---------|--------|-------|
| Missing phases? | ✅ No | Discovery → Protocol → Driver → Integration → Hardening |
| Vague UoWs? | ⚠️ Partial | Phase 2 Step 2 "Implement RAII Resource Manager" needs breakdown |
| Ignored edge cases? | ⚠️ Yes | No mention of multi-GPU (Pixel 6 has multiple) |
| Regression protection? | ✅ Yes | Phase 1 mentions golden tests |

---

## Phase 3 — Architecture Alignment

### Current vs Proposed

```
CURRENT:
kernel/src/gpu.rs         → Re-export (23 lines)
levitate-gpu/src/gpu.rs   → Wraps virtio-drivers (122 lines)
virtio-drivers            → External crate (black box)

PROPOSED:
levitate-virtio/          → General VirtIO transport
levitate-virtio-gpu/      → Protocol + Driver
levitate-gpu/             → High-level API (optional)
```

### Alignment Assessment

| Check | Status | Notes |
|-------|--------|-------|
| Follows existing patterns? | ✅ Yes | Matches `levitate-hal`, `levitate-utils` naming |
| Module boundaries clear? | ✅ Yes | Protocol/Driver/Display separation |
| File sizes reasonable? | ✅ Yes | Protocol structs in separate files |
| Pixel 6 consideration? | ⚠️ Missing | Need `levitate-gpu-mali` or similar for real HW |

### Reference: Tock

The `.external-kernels/tock/chips/virtio/src/devices/virtio_gpu/` directory is **empty**. TEAM_094's claim that "Tock has a working VirtIO GPU implementation" needs verification.

---

## Phase 4 — Global Rules Compliance

| Rule | Status | Notes |
|------|--------|-------|
| Rule 0 (Quality > Speed) | ⚠️ | Async requirement not addressed |
| Rule 1 (SSOT) | ✅ | Plan in `docs/planning/virtio-gpu-scanout/` |
| Rule 2 (Team Registration) | ✅ | TEAM_094 created questions file |
| Rule 4 (Regression Protection) | ✅ | Golden tests mentioned |
| Rule 5 (Breaking Changes) | ✅ | Clean break from virtio-drivers |
| Rule 6 (No Dead Code) | ✅ | Phase 5 mentions removing virtio-drivers |
| Rule 8 (Ask Questions) | ✅ | Questions file exists |
| Rule 10 (Before Finishing) | ⚠️ | No explicit handoff checklist in Phase 5 |
| Rule 11 (TODO Tracking) | ⚠️ | No TODO policy mentioned |

---

## Phase 5 — Verification and References

### Claim: "Tock has working VirtIO GPU implementation"

**Status:** ❌ UNVERIFIED

The directory `.external-kernels/tock/chips/virtio/src/devices/virtio_gpu/` is empty (0 items).
Either:
1. The Tock submodule is incomplete
2. Tock VirtIO GPU is in a different location
3. The claim is incorrect

**Recommendation:** Search for actual Tock GPU implementation or cite alternative reference.

### Claim: "virtio-drivers uses blocking calls"

**Status:** ✅ VERIFIED

`@/home/vince/Projects/LevitateOS/levitate-gpu/src/gpu.rs:59-61` shows:
```rust
if let Err(_e) = self.gpu.flush() {
    self.failed_flushes += 1;
}
```
The `flush()` call is synchronous/blocking.

---

## Findings Summary

### Critical (Blocks Work)

1. **Q4 Async Requirement Not in Plan** - User explicitly said "DO IT RIGHT FROM THE START!!! NO MORE SIMPLER IMPLEMENTATIONS" for async-first design. Plan must be updated.

### Important (Improves Quality)

2. **Tock Reference Unverified** - Empty directory. Need alternative reference or update claim.
3. **Pixel 6 End-Goal Context Missing** - User mentioned this is for Pixel 6, which has Mali GPU, not VirtIO. Plan should note this is for QEMU dev environment.
4. **Phase 5 Handoff Checklist Incomplete** - Doesn't match Rule 10 requirements.

### Minor (Nice-to-Have)

5. **Phase 4 "Health Check Loop"** - Consider deferring until basic scanout works.

---

## Recommended Plan Updates

### Update Phase 2: Add Async Foundation

```markdown
## Steps
1. **Step 1 – Define Protocol Constants & Structs**
2. **Step 2 – Implement RAII Resource Manager**
3. **Step 3 – Unit Test Protocol Serialization**
4. **Step 4 – Define Async Command Trait** ← NEW
   - `async fn send_command(&mut self, cmd: impl VirtIOGPUReq) -> Result<R, GpuError>`
```

### Update Phase 3: Specify Async Driver

```markdown
## Perfection Criteria
- **Async-first design** per user requirement (Q4 answer)
- Decouple the "how to talk to VirtIO" from the "what to draw"
...
```

### Update Phase 5: Complete Handoff Checklist

```markdown
## Handoff Checklist
- [ ] Project builds cleanly
- [ ] All tests pass
- [ ] `cargo xtask run` shows graphical output
- [ ] `cargo xtask gpu-dump` works
- [ ] Team file updated with remaining TODOs
- [ ] GOTCHAS.md updated
```

---

## Status

- [x] Team file created
- [x] Review phases 1-5 complete
- [x] Apply updates to plan files
- [x] Update questions file with Tock reference note

---

## Changes Applied

### `docs/planning/virtio-gpu-scanout/phase-2.md`
- Added async foundation to Target Design
- Added Pixel 6/QEMU context note
- Expanded Step 2 with ResourceId and Drop impl detail
- Added Step 4: Define Async Command Trait

### `docs/planning/virtio-gpu-scanout/phase-3.md`
- Added async-first requirement to Perfection Criteria
- Updated Step 1 to specify async command queue with Waker
- Added state machine definition to Step 2
- Enhanced Step 3 with CtrlType logging

### `docs/planning/virtio-gpu-scanout/phase-5.md`
- Added behavioral regression test to verification
- Added Cleanup section (virtio-drivers removal, dead code, TODOs)
- Added proper Handoff Checklist per Rule 10
- Added Pixel 6 context note

### `.questions/TEAM_094_virtio_gpu_crate_structure.md`
- Added TEAM_095 Review Notes section
- Documented Tock reference verification issue

---

## Handoff

**Review Complete.** The plan now reflects:
1. ✅ User's async-first requirement (Q4)
2. ✅ Proper handoff checklist (Rule 10)
3. ✅ Cleanup phase with dead code removal (Rule 6)
4. ✅ TODO tracking requirement (Rule 11)
5. ✅ Pixel 6 end-goal context noted

**Remaining Risk:** Tock reference is unverified. Recommend using VirtIO 1.1 spec directly.

---

## Spec-Driven Development Document Created

Created comprehensive VirtIO GPU specification document:
`docs/planning/virtio-gpu-scanout/VIRTIO_GPU_SPEC.md`

**Contents:**
- Complete protocol structures with Rust representations
- All command/response types (2D mode)
- Pixel formats with byte order
- Initialization sequence and state machine
- Runtime render loop pseudocode
- RAII resource pattern
- Async command trait design
- Struct sizes for buffer allocation
- QEMU-specific notes

**Source:** OASIS VirtIO 1.1 Specification, Section 5.7

This document serves as the **single source of truth** for spec-driven development.

