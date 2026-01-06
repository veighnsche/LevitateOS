# TEAM_131 — Reduce Unsafe Code via Safe Abstractions

## Objective

Create a bugfix plan to drastically reduce unsafe code across the kernel by:
1. Identifying common unsafe patterns
2. Designing safe abstraction libraries
3. Replacing raw unsafe with safe wrappers

## Investigation Log

### Phase 1 — Auditing Unsafe Patterns (COMPLETE)

**Baseline:** 148 unsafe blocks across kernel crates

**Pattern Categories Identified:**
1. Volatile I/O (12 occurrences) — MMIO/DMA access
2. Inline Assembly (30 occurrences) — System registers, barriers
3. Raw Slice Creation (7 occurrences) — from_raw_parts
4. Intrusive Lists (8 occurrences) — NonNull manipulation

## Planning Documents Created

| Phase | File | Status |
|-------|------|--------|
| Phase 1 | `docs/planning/reduce-unsafe-code/phase-1.md` | ✅ Complete |
| Phase 2 | `docs/planning/reduce-unsafe-code/phase-2.md` | ✅ Complete |
| Phase 3 | `docs/planning/reduce-unsafe-code/phase-3.md` | ✅ Complete |
| Phase 4 | `docs/planning/reduce-unsafe-code/phase-4.md` | ✅ Complete |
| Phase 5 | `docs/planning/reduce-unsafe-code/phase-5.md` | ✅ Complete |

## Proposed Abstractions

| Library | Reduces | Complexity |
|---------|---------|------------|
| `barrier` module | 8 unsafe → 1 | Low |
| `volatile` wrapper | 12 unsafe → 2 | Low |
| `sysreg` macros | 30 unsafe → 2 | Medium |
| `intrusive_list` | 8 unsafe → 2 | High |

**Estimated total reduction:** 148 → <60 unsafe blocks (**~60% reduction**)

## Status

**PLAN COMPLETE** — Ready for implementation.

Next team should start with Phase 4, Step 1 (barrier module).
