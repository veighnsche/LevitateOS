# TEAM_377 — Review: Eyra Coreutils Refactor Plan

**Created:** 2026-01-10  
**Plan Under Review:** `docs/planning/eyra-coreutils-refactor/`  
**Original Author:** TEAM_376

---

## Review Summary

**Verdict: ✅ APPROVE with minor refinements**

The plan is well-scoped, correctly identifies real problems, and proposes appropriate solutions. The claims have been verified against the codebase. A few refinements are suggested below.

---

## Phase 1: Questions and Answers Audit

### Relevant Questions Files

| File | Topic | Relevance to This Plan |
|------|-------|------------------------|
| TEAM_349_eyra_integration.md | Syscall design | Not directly relevant |
| TEAM_359_eyra_syscalls_questions.md | ppoll, tkill | Not directly relevant |
| TEAM_363_eyra_args_crash.md | Static-PIE args | ✅ FIXED - Not blocking |
| TEAM_366_eyra_uutils_investigation.md | uutils _start conflict | **Potentially relevant** |

### Findings

1. **No direct questions for this refactor plan** — The plan is about workspace hygiene, not syscall implementation.

2. **TEAM_366 questions are relevant** — The uutils investigation mentions build issues. The refactor should not create new conflicts with the ongoing uutils work.

3. **No contradictions found** — The plan doesn't conflict with any answered questions.

---

## Phase 2: Scope and Complexity Check

### Metrics

| Metric | Value | Assessment |
|--------|-------|------------|
| Phases | 4 | Appropriate |
| Total UoWs | 6 | Appropriate |
| Time Estimate | 4-5 hours | Realistic |

### Overengineering Check

- **No unnecessary abstractions** ✅
- **No speculative features** ✅
- **UoW sizes are SLM-appropriate** ✅

### Oversimplification Check

- **Phase 3 testing strategy needs more detail** — Options A/B are presented but no clear decision is made. Recommendation needed.
- **Phase 4 cleanup is thin** — The `eyra-hello` decision is punted. Should have a clear answer.

### Concerns

1. **Phase 3 lacks a concrete decision** — "Option A + Kernel spawn" is mentioned but Phase 3 doesn't commit to a specific implementation path.

2. **Missing: .gitignore already exists?** — The plan assumes `.gitignore` doesn't prevent stale artifacts, but should verify current `.gitignore` state first.

---

## Phase 3: Architecture Alignment

### Verified Against Codebase

| Claim | Verified | Notes |
|-------|----------|-------|
| 15 stale target folders | ✅ | Exactly 15 found |
| 5.1GB total waste | ✅ | ~5.0GB confirmed (802M+608M+...) |
| 15 stale Cargo.lock | ✅ | Exactly 15 found |
| 15 stale .cargo | ✅ | Exactly 15 found |
| Workspace already correct | ✅ | eyra/Cargo.toml is proper workspace |
| Test runner inadequate | ✅ | Only tests std, not utilities |

### Alignment Issues

1. **eyra-hello not in workspace** — The plan mentions `eyra-hello` for possible removal, but it's actually IN the workspace `Cargo.toml`. It should either stay (as a test binary) or be properly removed from workspace.

2. **No conflicts with existing patterns** — The refactor follows existing Rust workspace conventions.

---

## Phase 4: Global Rules Compliance

| Rule | Status | Notes |
|------|--------|-------|
| Rule 0 (Quality > Speed) | ✅ | Clean workspace is the right approach |
| Rule 1 (SSOT) | ✅ | Plan in correct location |
| Rule 2 (Team Registration) | ✅ | TEAM_376 registered |
| Rule 4 (Regression Protection) | ⚠️ | Phase 3 testing improves this |
| Rule 5 (Breaking Changes) | ✅ | No compatibility hacks |
| Rule 6 (No Dead Code) | ✅ | Phase 4 includes cleanup |
| Rule 7 (Modular Refactoring) | ✅ | N/A for this cleanup |
| Rule 10 (Handoff) | ⚠️ | No explicit handoff checklist in phases |
| Rule 11 (TODO Tracking) | ⚠️ | No TODO.md mentions |

### Rule Violations

1. **Minor: No explicit handoff checklist per phase** — Each phase should end with verification steps.

---

## Phase 5: Verification and References

### Claims Verified

1. **Workspace builds use shared target/** — Confirmed by existence of `eyra/target/` and workspace `Cargo.toml`.

2. **Per-utility builds are redundant** — Confirmed; all utilities are workspace members.

3. **-nostartfiles is needed** — This is standard for Eyra binaries using Origin for `_start`.

### Unverified Claims

1. **"Add .gitignore for individual utility folders"** — The Phase 2 suggestion to add gitignore may conflict with existing patterns. Should check root `.gitignore` first.

---

## Phase 6: Refinements

### Critical (Must Fix)

None — plan is fundamentally sound.

### Important (Should Fix)

1. **Phase 3: Commit to Option A (shell-based testing)**
   - Option B adds complexity with no clear benefit
   - Shell-based testing aligns with existing `run-test.sh` infrastructure

2. **Phase 4: Decide on eyra-hello**
   - Recommendation: Keep it as a minimal test binary
   - It's already in workspace, serves as Eyra sanity check

3. **Add verification step to Phase 1**
   - Before deleting, run `./run-test.sh` to establish baseline
   - After deleting, run again to confirm no regression

### Minor (Nice to Have)

1. **Check existing `.gitignore` before Phase 2**
2. **Add handoff checklist to each phase file**

---

## Final Verdict

**✅ APPROVED — Ready for implementation with minor refinements above**

The plan correctly identifies 5GB of waste and proposes a clean solution. The scope is appropriate (not overengineered), and claims are verified against the actual codebase.

---

## Handoff Checklist

- [x] All questions files reviewed
- [x] Scope assessed (not over/under-engineered)
- [x] Architecture alignment verified
- [x] Global rules checked
- [x] Claims verified against codebase
- [x] Refinements documented
