# TEAM_313 — Review Fork/Exec Plan

**Created**: 2026-01-08
**Purpose**: Review and refine the fork-exec implementation plan
**Status**: Complete

---

## Context

Reviewing `docs/planning/fork-exec/` which is a subplan of `docs/planning/stability-maturation/`.

The fork-exec plan addresses Phase 3 blockers in stability-maturation:
- `sys_clone` only supports threads, not fork
- `sys_exec` is a stub (returns ENOSYS)

---

## Review Findings

### Phase 1 — Questions and Answers Audit ✅

**Questions files**: None found for fork-exec plan.

**Open questions in plan**: Phase 1 lists 4 open questions (Q1-Q4) that are answered in Phase 2.
- Q1 (COW vs Eager): Answered → Eager copy
- Q2 (FD inheritance): Answered → Clone on fork, keep on exec
- Q3 (Pending signals): Answered → Cleared for child
- Q4 (vfork): Answered → Not implementing

**Finding**: All questions answered inline. No `.questions/` file needed since answers are self-contained.

### Phase 2 — Scope and Complexity Check ✅

**Structure**: 3 phases (Discovery, Design, Implementation), 5 steps

| Metric | Count | Assessment |
|--------|-------|------------|
| Phases | 3 | Appropriate |
| Steps | 5 | Right-sized |
| New functions | 4 | Minimal |
| Modified functions | 2 | Minimal |

**Overengineering signals**: NONE FOUND
- No unnecessary abstractions
- COW deferred (correct)
- vfork deferred (correct)
- Steps are SLM-sized

**Oversimplification signals**: MINOR ISSUES

1. **Missing Phase 4**: Plan mentions "Phase 4 — Integration and Testing" at end of Phase 3 but no file exists.
   - **Recommendation**: This is acceptable if Phase 3 Step 5 covers integration. Mark as non-issue.

2. **exec_args missing**: Plan mentions `exec_args()` for `spawn_args()` migration but doesn't define it.
   - Current: `spawn_args(path, argv)` → `fork() + exec_args(path, argv)`
   - Plan only defines `exec(path)` with no argv support.
   - **ISSUE**: Need `exec_args()` or `execve(path, argv, envp)` variant.

3. **Jump to userspace**: Phase 3 `sys_exec()` calls `jump_to_user_exec()` but this function doesn't exist.
   - **Recommendation**: Need to define how exec returns to new entry point.

### Phase 3 — Architecture Alignment ✅

**Verified against codebase**:

| Plan Claim | Actual | Status |
|------------|--------|--------|
| `sys_clone` returns ENOSYS for fork | Line 424-428 in `syscall/process.rs` | ✅ Verified |
| `sys_exec` is stub | Line 139-140: "exec is currently a stub" | ✅ Verified |
| `create_user_page_table()` exists | `memory/user.rs` line 32 | ✅ Verified |
| Need `copy_user_address_space()` | Not in `memory/user.rs` | ✅ Correct |
| Need `clear_user_address_space()` | Not in `memory/user.rs` | ✅ Correct |
| `spawn_from_elf()` pattern | `task/process.rs` lines 45-48 | ✅ Verified |
| `create_thread()` exists | `task/thread.rs` line 48 | ✅ Verified |

**Pattern Alignment**:
- Fork implementation mirrors `create_thread()` pattern but with copied address space.
- Exec implementation reuses existing ELF loading from `spawn_from_elf_with_args()`.
- Plan correctly identifies code reuse opportunities.

**File locations match existing structure**: ✅

### Phase 4 — Global Rules Compliance ✅

| Rule | Status | Notes |
|------|--------|-------|
| Rule 0 (Quality > Speed) | ✅ | Eager copy is simple, COW deferred |
| Rule 1 (SSOT) | ✅ | Plan in `docs/planning/fork-exec/` |
| Rule 2 (Team Registration) | ✅ | TEAM_312 registered |
| Rule 3 (Before Work) | ✅ | Discovery phase done |
| Rule 4 (Regression Protection) | ⚠️ | Mentions golden tests but no specific test plan |
| Rule 5 (Breaking Changes) | ✅ | Clean removal of Spawn/SpawnArgs |
| Rule 6 (No Dead Code) | ✅ | Step 5 removes deprecated syscalls |
| Rule 7 (Modular Refactoring) | ✅ | Functions scoped correctly |
| Rule 10 (Before Finishing) | ⚠️ | No handoff checklist in plan |
| Rule 11 (TODO Tracking) | ✅ | Step 4 has exit criteria |

### Phase 5 — Verification and References ✅

**Verified Claims**:
1. `Spawn`/`SpawnArgs` deprecated in `los_abi` — ✅ Verified (lines 57-62 in aarch64.rs)
2. 6 spawn callsites to migrate:
   - `userspace/libsyscall/src/process.rs` (2 - spawn, spawn_args)
   - `userspace/init/src/main.rs` (1)
   - `userspace/levbox/src/bin/test/suite_test_core.rs` (1)
   - `userspace/levbox/src/bin/test/test_runner.rs` (1)
   - `userspace/shell/src/main.rs` (1)
3. `libsyscall` already has `clone()` and `exec()` wrappers — ✅ Verified

**Unverified/Incorrect Claims**:
1. Plan mentions `shell/src/main.rs` uses `spawn_args()` — needs `exec_args()` for migration.
2. `jump_to_user_exec()` function not defined anywhere — needs implementation.

---

## Issues Found

### Critical Issues: 0

### Important Issues: 2

1. **exec_args() missing** — Plan should define `exec_args(path, argv)` for `spawn_args` callsites.
2. **jump_to_user_exec() undefined** — Need to specify how exec jumps to new entry point.

### Minor Issues: 2

1. **No handoff checklist** in Phase 3 (add to Step 5).
2. **Phase 1 Step 4** incomplete (marked with `- [ ]` but Phase 2 answers the questions).

---

## Recommendations

### 1. Add exec_args() to Phase 3 Step 4

The userspace API needs:
```rust
pub fn exec_args(path: &str, argv: &[&str]) -> isize
```

### 2. Clarify exec return mechanism

Phase 3 Step 3 should specify that `sys_exec()` modifies the current trap frame's PC and SP rather than calling a separate function.

### 3. Mark Phase 1 Step 4 complete

Step 4 says "Define fork() behavior precisely" — this is done in Phase 2.

---

## Overall Assessment

**Plan Quality**: GOOD

The fork-exec plan is well-structured, appropriately scoped, and correctly identifies the work needed. The implementation approach is sound and follows existing patterns.

**Ready for Implementation**: YES (with minor fixes noted above)

**Complexity Rating**: MEDIUM — Touches MMU, task management, and userspace, but follows established patterns.

---

## Handoff Checklist

- [x] All review findings documented
- [x] Plan issues identified
- [x] Recommendations provided
- [x] Team file complete
