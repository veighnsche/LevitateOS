# TEAM_340: Review Linux ABI Compatibility Plan

**Date:** 2026-01-09  
**Status:** 🔴 CRITICAL ISSUES FOUND  
**Type:** Plan Review

## Plan Location

`docs/planning/linux-abi-compatibility/`

## Review Findings

### Phase 1: Questions and Answers Audit

#### 🔴 CRITICAL: Unanswered Question Blocks Entire Plan

**Question:** `docs/questions/TEAM_339_linux_abi_compatibility_decision.md`

**Status:** UNANSWERED

The plan assumes **Option A (Full Linux Compatibility)** but the user has not confirmed this choice. The plan should not proceed until the user answers.

**Options presented:**
- A: Full Linux compatibility (~26-38 UoW)
- B: Document as LevitateOS ABI (1-2 UoW)
- C: Hybrid compatibility layer (~10-15 UoW)

**Impact:** If user chooses B or C, the entire Phase 4 implementation plan is wrong.

---

### Phase 2: Scope and Complexity Check

#### ⚠️ Potential Overengineering

1. **Phases 1-3 overlap significantly with investigation already done**
   - TEAM_339 already cataloged discrepancies in `discrepancies.md`
   - Phases 1-2 duplicate this work
   - **Recommendation:** Merge Phases 1-3 into a single "Preparation" phase

2. **UoW count seems high (17 in Phase 4 alone)**
   - Many UoWs are simple (e.g., "verify struct matches")
   - Could batch related changes more aggressively
   - **Recommendation:** Reduce to ~10-12 UoWs by combining

3. **Batch 4 "Quick Fixes" is misplaced**
   - `__NR_pause` and errno fixes are independent of path syscalls
   - Should be Batch 0.5 or parallel track
   - **Recommendation:** Move to earlier or separate track

#### ✅ Appropriate Complexity

- Batched approach is good for risk management
- Checkpoints after each batch is correct
- Phase 5 cleanup is appropriate

---

### Phase 3: Architecture Alignment

#### ✅ Correct Approach

- Using existing `crates/kernel/src/syscall/` structure
- Not creating new modules unnecessarily
- Respects existing patterns

#### ⚠️ Minor Issues

1. **Safe string helper location**
   - Plan puts `read_user_cstring()` in `syscall/mod.rs`
   - Better location: `crates/kernel/src/memory/user.rs` (where other user memory functions live)

2. **Errno consolidation**
   - Plan says "use linux_raw_sys::errno in kernel"
   - May conflict with kernel's no_std constraints
   - Need to verify linux_raw_sys works in no_std kernel

---

### Phase 4: Global Rules Compliance

| Rule | Status | Notes |
|------|--------|-------|
| Rule 0 (Quality) | ✅ | Plan chooses correctness over shortcuts |
| Rule 1 (SSOT) | ✅ | Plan in correct location |
| Rule 2 (Team) | ✅ | TEAM_339 registered |
| Rule 3 (Pre-work) | ✅ | Investigation done first |
| Rule 4 (Regression) | ⚠️ | Tests mentioned but not detailed |
| Rule 5 (Breaking) | ✅ | Clean break, no shims |
| Rule 6 (Dead Code) | ✅ | Cleanup phase exists |
| Rule 7 (Modular) | ✅ | No new modules needed |
| Rule 8 (Questions) | 🔴 | **Question unanswered** |
| Rule 9 (Context) | ✅ | Batched sensibly |
| Rule 10 (Handoff) | ✅ | Checklist in Phase 5 |
| Rule 11 (TODOs) | ⚠️ | No TODO tracking mentioned |

---

### Phase 5: Verification and References

#### Claims to Verify

| Claim | Verification | Status |
|-------|--------------|--------|
| Linux openat signature | man 2 openat | ⚠️ Need to verify exact signature |
| AT_FDCWD = -100 | Linux headers | ✅ Correct |
| aarch64 has no pause | Linux syscall table | ⚠️ Need to verify |
| Stat struct size | sizeof comparison | ⚠️ Not verified yet |

---

## Recommended Changes

### 🔴 BLOCKER: Get User Decision

Before any implementation:
1. User must answer the question in `docs/questions/TEAM_339_linux_abi_compatibility_decision.md`
2. If Option B or C chosen, scrap current plan and create new one

### ⚠️ Simplify Plan Structure

1. **Merge Phases 1-3** into single "Preparation" phase
   - Work is already done in investigation
   - No need for 3 separate phases

2. **Consolidate UoWs** in Phase 4:
   - Combine UoW 1.2/1.3/1.4 into "Verify read-only structs"
   - Combine Batch 5 into Batch 4
   - Target: 10-12 UoWs total

3. **Move Batch 4 earlier**
   - __NR_pause and errno fixes don't depend on path syscalls
   - Can be done in parallel or first

### ⚠️ Add Missing Details

1. **Test file locations** - specify where new tests go
2. **TODO tracking** - add to Phase 5
3. **Verify linux_raw_sys no_std** - check if usable in kernel

---

## Summary

| Category | Status |
|----------|--------|
| Questions Audit | 🔴 BLOCKER: Question unanswered |
| Scope/Complexity | ⚠️ Slightly overengineered |
| Architecture | ✅ Good |
| Rules Compliance | ⚠️ Minor issues |
| Verification | ⚠️ Some claims unverified |

**Verdict:** Plan is blocked until user answers the Linux compatibility question. Structure is reasonable but could be simplified.
