# TEAM_153: Feature - Error Handling Macro

**Created:** 2026-01-06  
**Task:** Design and plan `define_kernel_error!` macro for consistent error handling  
**Status:** Plan Complete - Ready for Implementation

---

## Feature Summary

Create a declarative macro that enforces consistent error handling patterns across all kernel subsystems, eliminating boilerplate and guaranteeing format consistency.

## Plan Location

`docs/planning/error-macro/`

## Related Work

- TEAM_150: Established `BlockError` pattern
- TEAM_152: Implemented error codes across 7 error types

## Key Decisions

| Question | Decision |
|----------|----------|
| Q1: Nested errors | Extend macro syntax to support `Variant(InnerType)` |
| Q2: Subsystem constant | Yes - export `SUBSYSTEM: u8` constant |
| Q3: Duplicate detection | No - rely on code review |
| Q4: Location | New `levitate-error` crate |

## Progress

- [x] Phase 1: Discovery
- [x] Phase 2: Design (with answered questions)
- [x] Phase 3: Implementation plan
- [x] Phase 4: Integration plan

## Next Steps

Ready for implementation. Execute Phase 3 UoWs:
1. Create `levitate-error` crate
2. Add workspace dependencies
3. Migrate `FdtError` (simple proof)
4. Migrate `SpawnError` (nested proof)
5. Verify migrations
