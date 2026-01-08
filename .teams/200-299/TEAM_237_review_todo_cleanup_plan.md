# TEAM_237: Review TODO Cleanup & Crate Audit Plan

**Created:** 2026-01-07  
**Task:** Review plan at `docs/planning/todo-cleanup-crate-audit/` for completeness

---

## 1. Summary

Reviewing the TODO cleanup and crate audit plan to ensure it covers all TODOs in the source code and doesn't skip anything.

---

## 2. TODO Audit Results

### 2.1 Source Code TODOs Found

**kernel/src/syscall/mm.rs:**
| Line | TODO | In Plan? | Phase 3 Coverage |
|------|------|----------|------------------|
| 128 | Unmap previously allocated pages on failure | ✅ | Step 2 (MmapGuard) |
| 141 | Free physical pages and unmap on failure | ✅ | Step 2 (MmapGuard) |
| 177 | Implement proper VMA tracking and page unmapping | ✅ | Step 3 (VMA + munmap) |
| 206 | Implement proper page table protection modification (mprotect) | ✅ Documented | Deferred (MEDIUM) |

**kernel/src/memory/user.rs:**
| Line | TODO | In Plan? | Phase 3 Coverage |
|------|------|----------|------------------|
| 273 | Use actual entropy | ✅ Documented | Deferred (MEDIUM) |
| 308 | Pass actual HWCAP | ✅ Documented | Deferred (LOW) |
| 390 | Implement full page table teardown | ✅ | Step 1 (destroy_user_page_table) |

**kernel/src/fs/vfs/dispatch.rs:**
| Line | TODO | In Plan? |
|------|------|----------|
| 284 | Implement proper permission checking | ✅ Documented | Deferred (MEDIUM) |

**kernel/src/fs/vfs/inode.rs:**
| Line | TODO | In Plan? |
|------|------|----------|
| 159-166 | Get actual time from clock | ✅ Documented | Deferred (LOW) |

**kernel/src/task/thread.rs:**
| Line | TODO | In Plan? |
|------|------|----------|
| 127 | Share fd_table when CLONE_FILES is set | ✅ Documented | Deferred (MEDIUM) |

**kernel/src/task/mod.rs:**
| Line | TODO | In Plan? |
|------|------|----------|
| 291 | Inherit CWD from parent | ✅ Documented | Deferred (LOW) |

**userspace/ TODOs:**
| File | Line | TODO | In Plan? |
|------|------|------|----------|
| ulib/src/entry.rs | 90 | Call _init if defined | ✅ Documented (LOW) |
| levbox/src/bin/mkdir.rs | 42 | Support -p (parents) | ✅ Documented (LOW) |
| levbox/src/bin/rm.rs | 91 | Handle -r recursive | ✅ Documented (LOW) |
| levbox/src/bin/cp.rs | 92 | Write to destination | ✅ Documented (MEDIUM) |

### 2.2 Verdict: All TODOs Covered ✅

All TODOs in the source code are documented in Phase 1 and appropriately prioritized.

---

## 3. UoW File Coverage Analysis

### 3.1 Discrepancy Found

**Phase-3.md overview describes:**
- Step 1: 5 UoWs (1.1-1.5)
- Step 2: 3 UoWs (2.1-2.3)
- Step 3: 6 UoWs (3.1-3.6)
- **Total: 14 UoWs**

**Actual UoW files:**
- Step 1: 3 files (uow-1, uow-2, uow-3)
- Step 2: 2 files (uow-1, uow-2)
- Step 3: 5 files (uow-1 through uow-5)
- **Total: 10 UoW files**

### 3.2 Missing UoW Files

| Described in phase-3.md | Actual File Exists? |
|-------------------------|---------------------|
| UoW 1.1: Walker helper | ✅ phase-3-step-1-uow-1.md |
| UoW 1.2: Leaf page freeing | ❌ Merged into uow-2 |
| UoW 1.3: Table freeing (bottom-up) | ❌ Merged into uow-2 |
| UoW 1.4: Wire up destroy_user_page_table | ✅ As phase-3-step-1-uow-2.md |
| UoW 1.5: Unit test | ✅ As phase-3-step-1-uow-3.md |
| UoW 2.1: MmapGuard RAII | ✅ phase-3-step-2-uow-1.md |
| UoW 2.2: Integrate into sys_mmap | ✅ phase-3-step-2-uow-2.md |
| UoW 2.3: Test for mmap failure | ❌ Missing |
| UoW 3.1-3.5 | ✅ All present |
| UoW 3.6: VMA unit tests | ❌ Inline in uow-2 tests section |

### 3.3 Impact Assessment

The consolidation is reasonable - tests are included inline where appropriate. However:

1. **Numbering mismatch** - UoW file names don't match the overview numbering
2. **Missing test UoW** - UoW 2.3 (mmap failure test) has no dedicated file
3. **UoW 3.6** - VMA tests are inline in 3.2, not separate

---

## 4. Architecture Alignment Check

### 4.1 Verified Correct
- VMA types use `bitflags` (consistent with codebase)
- VmaList uses `Vec` (per Q1 decision - simplicity)
- Error types use `define_kernel_error!` macro
- RAII pattern (MmapGuard) follows Rust idioms

### 4.2 Potential Issues

**Issue 1: `kernel/src/memory/vma.rs` doesn't exist yet**
- UoW 3.1 creates it - correct
- Module export in `memory/mod.rs` needs to be added - documented in UoW 3.1

**Issue 2: TaskControlBlock modification**
- Plan mentions adding `vmas: IrqSafeLock<VmaList>` field
- UoW 3.3 correctly notes to check all constructors
- **Verified:** 3 construction sites exist:
  1. `kernel/src/task/mod.rs:236` - `new_bootstrap()`
  2. `kernel/src/task/mod.rs:278` - `From<UserTask>`
  3. `kernel/src/task/thread.rs:111` - direct struct construction in `clone_thread()`
- Plan should explicitly list all 3 sites to ensure none are missed

---

## 5. Findings Summary

### 5.1 GOOD ✅
- All source code TODOs are documented
- Prioritization is reasonable (HIGH = memory safety)
- Design decisions are justified per kernel-development.md rules
- Phase 3 focuses on correct scope (HIGH priority only)
- Crate audit decisions are sensible (keep custom where appropriate)

### 5.2 ISSUES ⚠️

| Issue | Severity | Recommendation |
|-------|----------|----------------|
| UoW numbering mismatch | Minor | Renumber files OR update phase-3.md |
| Missing UoW 2.3 file | Minor | Tests can remain inline |
| VMA module not in memory/mod.rs | Covered | UoW 3.1 documents this |

### 5.3 MISSING CONTENT

**Nothing critical missing.** The plan covers:
- All HIGH priority TODOs (memory safety)
- Defers MEDIUM/LOW appropriately
- Provides clear implementation guidance

---

## 6. Recommendations

1. **DONE:** Updated UoW 3.3 with verified TCB constructor locations
   - Was: vague "search for constructors"
   - Now: explicit 3 locations with line numbers

2. **Minor:** Update phase-3.md UoW counts to match actual files (14 → 10)
   OR create the missing UoW files (2.3, 3.6)

3. **Consider:** Add Phase 4/5 stubs for integration testing and documentation
   (mentioned in phase-2.md but no files exist)

4. **Verify:** Check that all MEDIUM priority items are tracked as issues elsewhere

---

## 7. Review Verdict

**PLAN IS COMPLETE AND READY FOR IMPLEMENTATION**

The plan correctly identifies and prioritizes all TODOs in the source code. The HIGH priority items (memory safety) have detailed implementation guidance. The minor UoW numbering discrepancy does not affect implementation.

---

## 8. Handoff Checklist

- [x] All source code TODOs audited
- [x] Plan coverage verified
- [x] Architecture alignment checked
- [x] No missing critical content
- [x] UoW 3.3 improved with explicit TCB constructor locations
- [ ] Phase-3.md UoW count correction (optional)

---

## 9. Changes Made

1. **Updated `phase-3-step-3-uow-3.md`:**
   - Replaced vague "search for constructors" guidance
   - Added explicit 3 TCB construction sites with line numbers
   - Ensures implementers won't miss any sites
