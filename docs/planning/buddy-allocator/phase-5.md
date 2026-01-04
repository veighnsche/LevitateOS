# Phase 5: Polish and Cleanup

**Status**: [ ] Pending | [x] In Review | [ ] Approved
**Owner**: TEAM_041
**Reviewed By**: TEAM_042 (2026-01-04)
**Target Hardware**: Pixel 6 (8GB RAM)

## 1. Cleanup
- Remove `static mut PT_POOL` from `mmu.rs` (if fully replaced — see Q3).
- Remove `__heap_start`/`__heap_end` references from `linker.ld` (if no longer needed).
- Update `kernel/src/main.rs` heap initialization (lines 261-272).
- Document new memory map.

## 2. Documentation
- Update `ARCHITECTURE.md` with memory management details.
- Update `ROADMAP.md` (Check off Buddy Allocator).

## 3. Handoff Checklist (TEAM_042 Addition)

Before marking Phase 5 complete:
- [ ] Project builds cleanly (`cargo build --release`)
- [ ] All existing tests pass (`cargo test`)
- [ ] New buddy allocator tests pass
- [ ] Boot test with `-m 1G` succeeds
- [ ] Boot test with `-m 2G` succeeds
- [ ] Regression baselines match expected output
- [ ] No `TODO(TEAM_XXX)` comments left untracked
- [ ] Team file updated with completion notes
- [ ] ROADMAP.md updated
