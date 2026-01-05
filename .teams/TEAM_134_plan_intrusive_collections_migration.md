# TEAM_134 — Plan Intrusive-Collections Migration

**Started:** 2026-01-06
**Plan Location:** `docs/planning/intrusive-collections-migration/`
**Status:** Planning

## Objective

Create a detailed implementation plan for migrating the buddy allocator and slab allocator 
from manual `NonNull<T>` linked list operations to `intrusive-collections` crate.

## Background

TEAM_133 attempted this migration and discovered it's more complex than originally estimated:
- `LinkedList::new()` with `UnsafeRef` adapters has const initialization issues
- Pages live in a static memory map, not owned via Box
- Requires complete rewrite of add_to_list/remove_from_list operations

## Progress Log

### Session 1 — 2026-01-06

- [x] Phase 1: Understanding and Scoping — `phase-1.md`
- [x] Phase 2: Root Cause Analysis — `phase-2.md`
- [x] Phase 3: Fix Design and Validation Plan — `phase-3.md`
- [x] Phase 4: Implementation Steps — `phase-4.md`
- [x] Phase 5: Cleanup and Handoff — `phase-5.md`

## Key Finding

**Root Cause:** `intrusive_adapter!` with `UnsafeRef` does not generate const-compatible adapter.

**Solution:** Use `Option<LinkedList<PageAdapter>>` with lazy initialization in `init()`.

## Notes

This plan addresses Phase 4 Step 4 of the reduce-unsafe-code plan.

## Status

**PLANNING COMPLETE** — Ready for implementation by future team.
