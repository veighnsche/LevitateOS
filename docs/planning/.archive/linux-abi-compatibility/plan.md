# Linux ABI Compatibility Bugfix Plan

**TEAM_339** | Created: 2026-01-09  
**Status:** In Progress

**Last Updated:** 2026-01-09 (TEAM_344 review)

## Bug Summary

LevitateOS uses Linux syscall **numbers** but has **custom syscall signatures** that differ from Linux ABI. This makes the OS incompatible with standard Linux binaries and libc implementations.

**Severity:** High (fundamental ABI mismatch)  
**Impact:** Cannot run unmodified Linux ELF binaries

## Root Cause (from Investigation)

Deliberate design decision to use length-counted strings `(ptr, len)` instead of null-terminated strings for path arguments. This breaks Linux ABI while providing safety benefits.

## Phases

| Phase | Name | Status | Description |
|-------|------|--------|-------------|
| 1 | [Understanding and Scoping](phase-1.md) | ✅ DONE | Catalog all discrepancies |
| 2 | [Root Cause Analysis](phase-2.md) | ✅ DONE | Classify by fix complexity |
| 3 | [Fix Design and Validation](phase-3.md) | 📋 TODO | Design fix approach |
| 4 | [Implementation and Tests](phase-4.md) | 🔄 PARTIAL | Execute fixes (UoW 4.2 done by TEAM_342) |
| 5 | [Cleanup and Handoff](phase-5.md) | 📋 TODO | Final verification |

## Estimated Effort

| Phase | Est. UoWs | Complexity |
|-------|-----------|------------|
| 1 | 2-3 | Low |
| 2 | 2-3 | Low |
| 3 | 3-4 | Medium |
| 4 | 15-25 | High |
| 5 | 2-3 | Low |
| **Total** | **24-38** | |

## Key Deliverables

1. All syscall signatures match Linux ABI exactly
2. Struct layouts (Stat, Termios) match Linux exactly
3. Error codes consistent between kernel and userspace
4. Regression tests for ABI compatibility
5. Documentation of any remaining LevitateOS extensions

## Dependencies

- Phase 2 depends on Phase 1
- Phase 3 depends on Phase 2
- Phase 4 depends on Phase 3
- Phase 5 depends on Phase 4

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Break existing userspace | High | High | Run all tests after each change |
| Introduce security bugs | Medium | High | Careful null-termination handling |
| Miss edge cases | Medium | Medium | Comprehensive test coverage |
| Large scope creep | Medium | Medium | Strict phase boundaries |

## Related Files

- `.teams/TEAM_339_investigate_linux_abi_compatibility.md` - Investigation results
- `docs/questions/TEAM_339_linux_abi_compatibility_decision.md` - Decision question
