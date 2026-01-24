# TEAM_104: CLAUDE.md Refactoring

**Status:** Complete
**Started:** 2026-01-24
**Completed:** 2026-01-24

## Objective

Refactor all CLAUDE.md files into a unified, non-redundant system with a code map to help agents find where to implement things.

## Problems Solved

| Problem | Solution |
|---------|----------|
| No code map | Added "Code Map: Where Things Live" to root CLAUDE.md |
| "STOP READ THEN ACT" duplicated 15x | Single copy in root, removed from all per-crate files |
| Outdated root architecture | Replaced with accurate directory tree |
| Anti-cheat in leviso/CLAUDE.md | Moved to `.teams/KNOWLEDGE_anti-cheat-testing.md` |
| Wrong path in leviso | Fixed (was `install-tests/`, now `testing/install-tests/`) |
| Inconsistent structure | All per-crate files now have standard format |

## New Structure

### Root CLAUDE.md (~150 lines)
- What is LevitateOS (brief)
- STOP. READ. THEN ACT. (single copy)
- Code Map: Where Things Live
- Global Rules (10 rules)
- Commands
- Team File Requirement

### Per-Crate CLAUDE.md (~30-50 lines each)
- What is {crate}?
- What Belongs Here
- What Does NOT Belong Here (table format)
- Commands
- Crate-specific notes

## Files Modified

- [x] Root CLAUDE.md - complete rewrite with code map
- [x] leviso/CLAUDE.md - slimmed from 183 to 59 lines
- [x] distro-spec/CLAUDE.md - added boundaries, kept architecture
- [x] tools/recipe/CLAUDE.md - standardized format
- [x] tools/recstrap/CLAUDE.md - standardized, kept error codes
- [x] tools/recfstab/CLAUDE.md - standardized format
- [x] tools/recchroot/CLAUDE.md - standardized format
- [x] testing/install-tests/CLAUDE.md - improved, added philosophy
- [x] testing/rootfs-tests/CLAUDE.md - standardized format
- [x] testing/cheat-guard/CLAUDE.md - standardized format
- [x] testing/cheat-test/CLAUDE.md - standardized format
- [x] testing/hardware-compat/CLAUDE.md - created (was missing)
- [x] llm-toolkit/CLAUDE.md - standardized format
- [x] docs/tui/CLAUDE.md - standardized format
- [x] docs/content/CLAUDE.md - standardized format
- [x] Created `.teams/KNOWLEDGE_anti-cheat-testing.md`

## Line Count Comparison

| File | Before | After | Change |
|------|--------|-------|--------|
| Root CLAUDE.md | 223 | 149 | -74 |
| leviso/CLAUDE.md | 183 | 59 | -124 |
| distro-spec/CLAUDE.md | ~60 | 81 | +21 |
| tools/recipe/CLAUDE.md | ~50 | 54 | +4 |
| tools/recstrap/CLAUDE.md | ~80 | 65 | -15 |
| All others | varied | 30-65 | standardized |

**Total reduction:** ~200 lines of duplicated content removed across all files.

## Key Improvements

1. **Every crate now has "What Does NOT Belong Here"** - directly addresses the duplicate code problem
2. **Root has authoritative code map** - agents can now find where implementations belong
3. **Single source for global rules** - no more duplication across files
4. **Anti-cheat content preserved** - moved to KNOWLEDGE file for reference without bloating CLAUDE.md

## Log

- 2026-01-24: Team file created
- 2026-01-24: Root CLAUDE.md rewritten with code map
- 2026-01-24: Created KNOWLEDGE_anti-cheat-testing.md
- 2026-01-24: Refactored all 14 per-crate CLAUDE.md files
- 2026-01-24: Complete
