# TEAM_398: Review - General Purpose OS Feature Plan

**Date**: 2026-01-10
**Status**: Review Complete
**Plan Reviewed**: `docs/planning/general-purpose-os/FEATURE_PLAN.md` (TEAM_397)

---

## Summary

This review evaluates the General Purpose Unix-Compatible OS feature plan against kernel development rules, architectural alignment, and claim verification. The plan has a strong vision but contains **CRITICAL issues** that must be addressed before implementation.

---

## 1. Questions Audit

### Open Questions Not Addressed

| Question | Impact |
|----------|--------|
| Can c-gull actually be built as libc.so? | **BLOCKER** - Phase B depends on this |
| Which dynamic linker to use (custom vs. porting musl's)? | Phase C scoping |
| How will /proc be implemented? VFS integration? | Phase D architecture |
| What is the kernel-side CoW page fault handler design? | Phase A fork implementation |

### Recommendation
Add "Investigation Tasks" before each phase to answer fundamental feasibility questions.

---

## 2. Scope Check

### Overengineering Concerns

| Issue | Location | Severity |
|-------|----------|----------|
| 6 phases is excessive for initial plan | Overall structure | Medium |
| Milestone 4 (Package Management) premature | Line 316-321 | Low |
| Networking (Phase F) included but marked "optional" | Lines 261-275 | Low |

### Undersimplification Concerns

| Issue | Location | Severity |
|-------|----------|----------|
| **No tests/validation phase** | Missing | **HIGH** |
| **No cleanup/regression phase** | Missing | **HIGH** |
| Vague UoWs: "Implement ELF interpreter" | Line 188 | Medium |
| Missing: syscall strace/debugging tools | Not mentioned | Medium |
| Missing: ABI compatibility test suite | Not mentioned | **HIGH** |

### Recommended Changes

1. **Add Phase 0: Validation Infrastructure**
   - [ ] Create syscall compatibility test suite
   - [ ] Add strace-like debugging for syscalls
   - [ ] Define "done" criteria per syscall

2. **Split Phase A into A1 (fork/exec) and A2 (everything else)**
   - fork/exec is high-complexity, deserves focused attention

3. **Remove or defer Milestone 4 (Package Management)**
   - Not needed for "general purpose" status
   - Violates Rule 20 (Simplicity > Perfection)

---

## 3. Architecture Alignment

### Module Boundary Violations

| Issue | Affected Modules |
|-------|-----------------|
| Plan assumes monolithic libc.so approach | Conflicts with existing Eyra integration |
| Dynamic linker placement unclear | New crate? Kernel module? Userspace? |

### Existing Patterns Not Considered

1. **Eyra Integration (TEAM_349-395)**
   - The codebase already uses Eyra for std support
   - Plan doesn't explain transition from Eyra to c-gull/libc.so
   - Risk: Orphaned Eyra code and conflicting approaches

2. **VFS Layer**
   - Plan mentions /proc filesystem but doesn't reference existing VFS
   - Should leverage `crates/kernel/src/fs/` patterns

### Recommendations

1. Add explicit section: "Relationship to Eyra" explaining:
   - Eyra is a stopgap for TODAY
   - c-gull/libc.so is the target for GENERAL PURPOSE
   - Transition plan from one to the other

2. Reference existing VFS traits in Phase D filesystem tasks

---

## 4. Rules Compliance

| Rule | Status | Notes |
|------|--------|-------|
| **Rule 0: No Shortcuts** | ⚠️ WARNING | c-gull libc.so claim not verified |
| **Rule 4: Tests/Baselines** | ❌ FAIL | No test phase, no golden files mentioned |
| **Rule 5: No Compatibility Hacks** | ✅ PASS | Plan aims for proper Linux ABI |
| **Rule 6: Cleanup Phase** | ❌ FAIL | No cleanup phase exists |
| **Rule 7: Well-Scoped Structure** | ⚠️ WARNING | 6 phases may be too many |
| **Rule 10: Handoff Checklist** | ❌ FAIL | Team assignments vague, no checklist |
| **Rule 14: Fail Loud** | N/A | Implementation concern |
| **Rule 20: Simplicity** | ⚠️ WARNING | Package management is scope creep |

### Required Fixes

1. **Add Test Phase** after Phase A with:
   - Unit tests for each syscall
   - ABI compatibility tests against Linux reference
   - Behavior tests for fork/exec lifecycle

2. **Add Cleanup Phase** at end:
   - Remove deprecated code paths
   - Consolidate duplicate implementations
   - Update documentation

3. **Create Handoff Checklist** for each phase:
   - [ ] All tests pass
   - [ ] Documentation updated
   - [ ] No regressions
   - [ ] TEAM file created

---

## 5. Claim Verification

### CRITICAL: c-gull as libc.so.6

**Claim** (Line 128-144): "c-gull can be built as libc.so.6"

**Verification Result**: ❌ **FALSE**

Research findings:
- c-gull Cargo.toml does NOT configure `crate-type = ["cdylib"]`
- Mustang (parent project) explicitly states: "Dynamic linking isn't implemented yet"
- Eyra deliberately avoids dynamic linking
- No existing example of c-gull as shared library exists

**Impact**: **Phase B (Critical Milestone) is not feasible as written.**

### Alternative Approaches

| Option | Effort | Compatibility |
|--------|--------|---------------|
| A. Contribute cdylib support to c-gull | High | Best long-term |
| B. Port musl-libc instead | Medium | Proven approach |
| C. Static-only binaries (defer dynamic) | Low | Limited compatibility |
| D. Binary patching at load time | High | Fragile |

**Recommendation**: Update plan to:
1. Investigate c-gull cdylib feasibility (new task)
2. Have musl-libc as fallback option
3. Prioritize static binary support as immediate goal

### Other Claims

| Claim | Status | Evidence |
|-------|--------|----------|
| fork/vfork 🟡 Partial | ✅ VERIFIED | `sys_clone` exists but no fork semantics |
| epoll 🟡 Partial | ✅ VERIFIED | `sys_epoll_*` in `epoll.rs` (TEAM_394) |
| poll/select 🟡 Partial | ✅ VERIFIED | `sys_ppoll` in `sync.rs` (TEAM_360) |
| fcntl 🟡 Partial | ✅ VERIFIED | Stub implementation in `fd.rs` (TEAM_394) |
| chmod/chown 🔴 Missing | ✅ VERIFIED | No implementation found |
| /proc 🔴 Missing | ✅ VERIFIED | No procfs in codebase |

---

## 6. Refinements Applied

### Priority: Critical (Blocks Work)

1. **REWRITE Phase B** to address libc.so feasibility:
   ```
   Phase B: libc Implementation
   - Task B.0: Investigate c-gull cdylib feasibility (1-2 days)
     - Can c-gull be compiled with crate-type = ["cdylib"]?
     - What symbols are missing for full libc ABI?
     - Decision point: c-gull vs musl-libc
   - Task B.1: Build libc.a (static) first (proven approach)
   - Task B.2: Build libc.so.6 (if B.0 succeeds) OR port musl
   ```

2. **ADD Milestone 0.5: Static Binary Compatibility Testing**
   - Before claiming static binaries work, test with:
     - musl-compiled hello world
     - Simple Rust binary (no Eyra)
     - uutils single-binary

### Priority: Important (Quality)

3. **ADD Test Infrastructure** to Phase A:
   ```
   - syscall_compat_tests/ directory with Linux reference tests
   - Each syscall gets a test that runs on both Linux and LevitateOS
   - CI integration for regression detection
   ```

4. **CLARIFY Eyra Transition**:
   ```
   Add to Phase B:
   "Relationship to Eyra: Eyra currently provides std support by requiring
   app modification. The libc.so approach eliminates this requirement.
   Once libc.so works, Eyra-modified apps continue working (they just
   don't need modification anymore)."
   ```

5. **ADD Handoff Checklist Template**:
   ```markdown
   ## Phase X Completion Checklist
   - [ ] All tasks marked complete
   - [ ] Tests added and passing
   - [ ] No regressions in existing tests
   - [ ] Documentation updated
   - [ ] TEAM_XXX file created with implementation details
   ```

### Priority: Minor (Polish)

6. **Remove Milestone 4** (Package Management) - out of scope for v1

7. **Consolidate Team Assignments**:
   - Current: "TEAM_400-420" is too vague
   - Better: Assign specific TEAM numbers when phases start

---

## Exit Criteria Checklist

- [x] Questions reflected in plan (noted as gaps)
- [x] Not over/under-engineered (identified issues)
- [x] Architecture-aligned (noted conflicts)
- [x] Rules-compliant (noted failures)
- [x] Claims verified (**CRITICAL: c-gull claim FALSE**)
- [x] Team file has review summary (this document)

---

## Verdict

**Plan Status**: ⚠️ **REQUIRES REVISION**

The General Purpose OS vision is sound and the mission is clear. However, the plan cannot proceed as-is due to:

1. **BLOCKER**: Phase B's core assumption (c-gull as libc.so.6) is not currently feasible
2. **Missing**: Test infrastructure and validation phases
3. **Missing**: Cleanup and handoff checklists

### Recommended Next Steps

1. **Investigate** c-gull cdylib support (create TEAM_399)
2. **Revise** Phase B based on investigation results
3. **Add** testing/validation phases
4. **Re-review** revised plan

---

## References

- Plan: `docs/planning/general-purpose-os/FEATURE_PLAN.md`
- c-ward: https://github.com/sunfishcode/c-ward
- Mustang (documents dynamic linking limitation): https://github.com/sunfishcode/mustang
- Existing syscalls: `crates/kernel/src/syscall/`
