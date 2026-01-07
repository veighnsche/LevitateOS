# TEAM_235: Feature Plan - TODO Cleanup & Crate Audit

## Status: REVIEWED & APPROVED (TEAM_236)

## Summary

Created comprehensive feature plan to:
1. Address all known TODOs in the codebase
2. Audit hand-rolled implementations for potential crate replacements

## Planning Location

`docs/planning/todo-cleanup-crate-audit/`

## Progress

- [x] Registered as TEAM_235
- [x] Discovered all TODOs in kernel, crates, and userspace
- [x] Analyzed current crate dependencies
- [x] Identified hand-rolled implementations
- [x] Phase 1 - Discovery document
- [x] Phase 2 - Design document with questions

## Discovered TODOs (11 kernel, 4 userspace)

### HIGH Priority (Memory Safety)
1. `destroy_user_page_table` - leaks pages
2. mmap failure cleanup - leaks on partial failure
3. VMA tracking for munmap - stub implementation

### MEDIUM Priority
4. fd_table sharing (CLONE_FILES)
5. Real entropy for AT_RANDOM
6. mprotect implementation
7. Permission checking in VFS

### LOW Priority
8. Real timestamps, HWCAP, CWD inheritance, etc.

## Crate Audit Results

| Component | Recommendation |
|-----------|----------------|
| ELF Parser | Keep custom (simple, focused) |
| CPIO Parser | Keep custom (adequate, few alternatives) |
| Ring Buffer | Keep custom (trivial) |
| Intrusive List | Consider migration to `intrusive-collections` |
| Buddy Allocator | Keep custom (specialized, well-tested) |

## Open Questions (6 total in phase-2.md)

User must answer Q1-Q6 before Phase 3 can begin.

## Handoff

Phase 1, 2, & 3 complete. Ready for implementation.

### Phase 3 UoW Files Created

**Step 1: Page Table Teardown (3 UoWs)**
- `phase-3-step-1-uow-1.md` - Recursive page table walker
- `phase-3-step-1-uow-2.md` - Implement destroy_user_page_table
- `phase-3-step-1-uow-3.md` - Unit test

**Step 2: mmap Failure Cleanup (2 UoWs)**
- `phase-3-step-2-uow-1.md` - MmapGuard RAII type
- `phase-3-step-2-uow-2.md` - Integrate into sys_mmap

**Step 3: VMA Tracking (5 UoWs)**
- `phase-3-step-3-uow-1.md` - VMA types (new file)
- `phase-3-step-3-uow-2.md` - VmaList container
- `phase-3-step-3-uow-3.md` - Add to TaskControlBlock
- `phase-3-step-3-uow-4.md` - Record VMAs in mmap
- `phase-3-step-3-uow-5.md` - Implement munmap
