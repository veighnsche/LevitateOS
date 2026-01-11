# TEAM_440: Review Brush Requirements Plan

## Objective

Review the brush-requirements plan at `docs/planning/brush-requirements/` per the /review-a-plan workflow.

## Status: COMPLETE

## Plan Summary

The plan addresses a **crash in brush shell** caused by `rt_sigaction` syscall format mismatch.

### Phases Reviewed
- Phase 1: Understanding and Scoping ✅
- Phase 2: Root Cause Analysis ✅
- Phase 3: Fix Design and Validation Plan ✅
- Phase 4: Implementation and Tests ✅
- Phase 5: Cleanup, Regression Protection, and Handoff ✅

---

## Review Findings

### Overall Assessment: MOSTLY SOUND - 4 ISSUES TO ADDRESS

The plan correctly identifies the root cause (rt_sigaction struct pointer mismatch) and proposes an appropriate fix. However, there are gaps that should be addressed before implementation.

---

### CRITICAL ISSUES

#### 1. Missing aarch64 Sigaction Layout (Architecture Gap)

**Location:** Phase 3, Phase 4

**Problem:** The plan only defines the x86_64 sigaction struct layout. aarch64 has a **different layout** - notably, it does NOT use `sa_restorer` the same way.

**Evidence:** Linux source and LTP tests confirm x86_64 requires SA_RESTORER, while aarch64 uses a different signal return mechanism.

**Impact:** Implementation would work on x86_64 but break aarch64 (violates Phase 1 constraint: "aarch64 parity required").

**Fix Required:**
- Add aarch64 sigaction struct definition (no sa_restorer field, different offsets)
- Add `#[cfg(target_arch)]` conditional handling in Phase 4 implementation

---

### IMPORTANT ISSUES

#### 2. Open Questions Not Tracked (Rule 8 Violation)

**Location:** Phase 1, lines 69-73

**Problem:** Three open questions are listed but not recorded in `.questions/`:
1. Does aarch64 have the same sigaction format issue?
2. Are there other syscalls with struct pointer mismatches?
3. What sigaction flags does tokio require?

**Fix Required:** Create `.questions/TEAM_438_brush_sigaction.md` or answer questions in plan.

#### 3. Helper Function Duplication (Architecture Alignment)

**Location:** Phase 4, UoW 3.2

**Problem:** Proposed `read_sigaction_from_user()` duplicates existing `read_struct_from_user()` in `helpers.rs`.

**Fix Required:** Use existing `read_struct_from_user::<KernelSigaction>()` pattern instead of byte-by-byte reading.

---

### MINOR ISSUES

#### 4. Deferred 64-bit Signal Mask (TODO Missing)

**Location:** Phase 4, UoW 5.1

**Problem:** 64-bit signal mask upgrade is "lower priority" but no TODO is documented for future work. `sys_sigprocmask` also uses 32-bit masks and will have the same issue.

**Fix Required:** Add TODO entry in `TODO.md` or inline comment.

---

## Verified Claims

| Claim | Status |
|-------|--------|
| Linux rt_sigaction takes struct pointers | ✅ Correct |
| x86_64 sigaction struct is 32 bytes | ✅ Correct |
| SA_RESTORER = 0x04000000 | ✅ Correct |
| tokio requires sigaction for SIGCHLD | ✅ Correct |
| Current sys_sigaction is wrong format | ✅ Verified in code |

---

## Recommendations

1. **Add aarch64 sigaction struct** before implementing (CRITICAL)
2. **Answer or defer** the three open questions
3. **Use existing helpers** for struct I/O
4. **Add TODO** for 64-bit signal mask follow-up

---

## Handoff

The plan is architecturally sound for x86_64. With the aarch64 gap addressed, implementation can proceed. The root cause analysis is correct and verified against the codebase.

---

## Plan Strengthening Applied

All issues from review have been addressed in the plan files:

| Issue | Resolution |
|-------|------------|
| Missing aarch64 layout | Added to Phase 3 + Phase 4 with `#[cfg(target_arch)]` |
| Open questions | Answered in Phase 1 |
| Helper duplication | Updated Phase 4 UoW 3.2 to use existing helpers |
| Missing TODO | Added to Phase 4 Step 5 |

**Files Modified:**
- `docs/planning/brush-requirements/phase-1.md` - Resolved open questions
- `docs/planning/brush-requirements/phase-3.md` - Added aarch64 struct layout
- `docs/planning/brush-requirements/phase-4.md` - Arch-conditional implementation, helpers, TODO

**Plan Status:** READY FOR IMPLEMENTATION

