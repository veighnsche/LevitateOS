# TEAM_238: Implement TODO Cleanup & Crate Audit

**Created:** 2026-01-07  
**Plan:** `docs/planning/todo-cleanup-crate-audit/`  
**Scope:** Phase 3 - HIGH priority memory TODOs

---

## 1. Implementation Order

### Step 1: Page Table Teardown
- [x] UoW 1.1: Add recursive page table walker helper
- [x] UoW 1.2: Implement destroy_user_page_table
- [x] UoW 1.3: Add unit test

### Step 2: mmap Failure Cleanup
- [x] UoW 2.1: Create MmapGuard RAII type
- [x] UoW 2.2: Integrate into sys_mmap

### Step 3: VMA Tracking
- [x] UoW 3.1: Create VMA types
- [x] UoW 3.2: Create VmaList container
- [x] UoW 3.3: Add VmaList to TaskControlBlock
- [x] UoW 3.4: Update sys_mmap to record VMAs
- [x] UoW 3.5: Implement sys_munmap using VMA

---

## 2. Progress Log

### Session Start
- Build: ✅ PASS (`cargo xtask build kernel`)
- Behavior test: ⚠️ Pre-existing golden file mismatch (not caused by this work)
- Starting with UoW 1.1

### Implementation Complete
- All 10 UoWs implemented
- Build: ✅ PASS (`cargo xtask build all`)
- 12 warnings (all pre-existing/expected dead code)

---

## 3. Files Modified

| File | Changes |
|------|--------|
| `kernel/src/memory/user.rs` | Added `collect_page_table_entries`, implemented `destroy_user_page_table`, added test |
| `kernel/src/memory/vma.rs` | **NEW** - VMA types, VmaList, VmaError, unit tests |
| `kernel/src/memory/mod.rs` | Added `pub mod vma` |
| `kernel/src/syscall/mm.rs` | Added MmapGuard, integrated into sys_mmap, implemented sys_munmap |
| `kernel/src/task/mod.rs` | Added `vmas` field to TCB, updated 2 constructors |
| `kernel/src/task/thread.rs` | Updated clone_thread() constructor |

---

## 4. Handoff Checklist

- [x] Project builds cleanly
- [x] All UoWs implemented per plan
- [x] Team file updated
- [x] Code comments include TEAM_238 tags
- [ ] Behavioral regression tests - pre-existing failure (golden file mismatch)

---

## 5. Notes

