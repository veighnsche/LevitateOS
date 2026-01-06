# TEAM_151: Bugfix Plan - Unified Error System

**Status:** Complete (Plan Ready)  
**Created:** 2026-01-06  
**Task:** Create comprehensive bugfix plan for unified error handling with numbered errors

## Bug Summary

LevitateOS lacks unified error handling:
- Inconsistent error types across modules (`&'static str`, enums, POSIX codes)
- No error numbering system for debugging
- Lost error context during conversions
- Panics in recoverable code paths

## Prior Investigation

- **TEAM_149**: Full investigation and inventory (`.teams/TEAM_149_investigate_error_handling.md`)
- **TEAM_150**: Fixed `block.rs` panics as proof-of-concept

## Planning Location

All phase files: `docs/planning/unified-error-system/`

## Progress

- [x] Phase 1: Understanding and Scoping
- [x] Phase 2: Root Cause Analysis  
- [x] Phase 3: Fix Design and Validation Plan
- [x] Phase 4: Implementation UoWs (7 UoWs defined)
- [x] Phase 5: Cleanup and Handoff

## Implementation Ready

**7 UoWs ready for execution:**

| UoW | Task | Est. Lines | Status |
|-----|------|------------|--------|
| 1 | Add codes to ElfError | ~30 | Ready |
| 2 | Add codes to FdtError | ~15 | Ready |
| 3 | Create MmuError type | ~80 | Ready |
| 4 | Migrate user_mm.rs | ~40 | Ready (needs UoW 3) |
| 5 | Update SpawnError | ~30 | Ready (needs UoW 1, 3) |
| 6 | Create FsError type | ~50 | Ready |
| 7 | Add codes to NetError | ~20 | Ready |

## Handoff

Next team should:
1. Pick a UoW from the list (start with independent ones: 1, 2, 3, 6, 7)
2. Follow the detailed instructions in `phase-4-uow-N.md`
3. Run verification steps after each UoW
4. Mark UoW complete and continue
