# TEAM_391: Review Plan - Eyra Shell Migration

**Status**: ✅ Complete  
**Created**: 2026-01-10

## Objective

Review and consolidate the eyra shell migration plans:
1. `docs/planning/eyra-shell/` (existing, brush-focused)
2. `docs/planning/shell-migration-to-eyra/` (new, Ion-focused - to be removed)

**Decision**: User confirmed **brush** shell for full bash/POSIX compatibility.

## Review Findings

### Issues Found
1. **Duplicate plans** - Two overlapping plans exist
2. **Ion references** - Phase 5 still mentions Ion Shell
3. **Missing tokio analysis** - brush uses tokio, needs syscall assessment

### Actions Taken
- [ ] Delete `docs/planning/shell-migration-to-eyra/` (redundant)
- [ ] Fix Ion references in `docs/planning/eyra-shell/`
- [ ] Strengthen syscall gap analysis
- [ ] Add tokio compatibility section

## Progress
- [x] Read both plans
- [x] Consolidate into single plan (deleted redundant shell-migration-to-eyra)
- [x] Fix Ion → brush references
- [x] Add tokio/epoll gap analysis
- [x] Create questions file for blockers
- [ ] Complete review checklist
- [ ] Finalize

## Review Checklist

### Phase 2: Scope & Complexity ✅
- **5 phases** for shell migration - appropriate
- **UoW sizing** - reasonable, brush handles most features
- **No overengineering** - leveraging existing shell, not building custom
- **No oversimplification** - tokio blocker identified, fallbacks documented

### Phase 3: Architecture Alignment ✅
- **Eyra workspace** - follows existing pattern (like coreutils)
- **Build integration** - matches existing eyra build flow
- **No new patterns** - uses established eyra + libsyscall approach

### Phase 4: Rules Compliance ✅
- [x] Rule 0: Quality path (using proven shell, not hacking)
- [x] Rule 1: SSOT in docs/planning/eyra-shell/
- [x] Rule 2: Team file exists
- [x] Rule 4: Tests mentioned in Phase 4
- [x] Rule 5: Clean replacement, not compatibility shim
- [x] Rule 6: Old shell removal in Phase 5
- [x] Rule 8: Questions file created for blockers
- [x] Rule 10: Handoff checklist in Phase 5

### Phase 5: Verification ✅
- [x] brush is real: https://github.com/reubeno/brush
- [x] tokio dependency confirmed
- [x] epoll gap verified in kernel code
- [x] pipe2 available confirmed in sysno.rs

## Summary

**Plan is READY** — All questions answered.

### Decisions Made
1. **Q1: Tokio/epoll resolution** → ✅ **Implement epoll in kernel** (Phase 0)
2. **Q2: Binary naming** → ✅ **`brush`** (use upstream name)

### Changes Made (Initial Review)
1. Deleted redundant `docs/planning/shell-migration-to-eyra/`
2. Fixed all Ion → brush references
3. Added tokio/epoll syscall gap analysis with concrete status
4. Created questions file with resolution options

### Changes Made (After User Decisions)
5. Created **Phase 0** for epoll prerequisite implementation
6. Updated all phase files with `brush` binary name
7. Updated design decisions in README
8. Marked questions as answered
9. Converted blocker warning to prerequisite notice
