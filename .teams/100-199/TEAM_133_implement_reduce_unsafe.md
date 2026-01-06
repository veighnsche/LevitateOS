# TEAM_133 — Implement Reduce Unsafe Code Plan

**Started:** 2026-01-06
**Plan:** `/docs/planning/reduce-unsafe-code/`
**Status:** In Progress

## Objective

Implement the reduce-unsafe-code plan by:
1. Adding dependencies (aarch64-cpu, intrusive-collections)
2. Migrating barriers to aarch64-cpu
3. Migrating system registers to aarch64-cpu
4. Migrating intrusive lists to intrusive-collections

## Progress Log

### Session 1 — 2026-01-06

- [x] Verified test baseline (79 tests pass, 135 unsafe blocks)
- [x] Step 1: Add dependencies (intrusive-collections added to levitate-hal)
- [x] Step 2: Migrate barriers — already done by TEAM_132
- [x] Step 3: Migrate system registers in exceptions.rs:
  - ESR_EL1.get() replaces raw asm mrs
  - ELR_EL1.get() replaces raw asm mrs  
  - VBAR_EL1.set() replaces raw asm msr
- [ ] Step 4: Migrate intrusive lists — **DEFERRED** (see Blockers)

**Unsafe count:** 135 → 133 (2 reduced)

## Blockers

### Intrusive-collections Migration Complexity

The buddy allocator migration to `intrusive-collections` is more complex than estimated:

1. **Const initialization issue**: `LinkedList::new()` with `UnsafeRef` adapters isn't const-compatible in a straightforward way
2. **Static array storage**: Pages live in a static `mem_map` array, not owned via Box
3. **Complete rewrite needed**: All `add_to_list`/`remove_from_list` operations need rewriting

**Recommendation**: This needs a dedicated session with ~150 lines of changes.
Consider using `LinkedList::const_new()` or lazy initialization pattern.

## Notes

Following plan from TEAM_131/TEAM_132.

## Remaining Work

1. **Step 4**: Intrusive list migration (buddy.rs, slab/list.rs)
2. **Phase 5**: Cleanup tasks (SAFETY comments, CI check)

## Handoff Checklist

- [x] Project builds cleanly
- [x] All tests pass (79 unit tests)
- [x] Team file updated with progress
- [x] Plan documentation updated (phase-4.md)
- [x] Remaining TODOs documented

**Status:** Session complete. Step 4 deferred to future team.
