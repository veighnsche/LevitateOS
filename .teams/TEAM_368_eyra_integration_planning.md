# TEAM_368 — Eyra Integration Planning

**Date:** 2026-01-10  
**Status:** IN PROGRESS

---

## Purpose

Create feature plan phases 4-5 to complete the Eyra integration work started by TEAM_367.

TEAM_367 solved the linker conflict. This team documents the remaining work:
1. Integrate Eyra binaries into initramfs
2. Test Eyra binaries on actual kernel

---

## Context Read

- [x] Read `docs/planning/eyra-uutils-linker-fix/FUTURE_TEAMS_README.md`
- [x] Read existing phase-1.md, phase-2.md, phase-3.md (investigation phases - complete)
- [x] Read TEAM_367 log (root cause found and fixed)
- [x] Verified cat builds successfully

---

## Work Done

### Created Phase Documents
- `phase-4.md` — Initramfs Integration
- `phase-5.md` — Kernel Testing & Validation

---

## Handoff Checklist

- [x] Phase 4 created
- [x] Phase 5 created
- [x] Team file completed

---

## Summary

Created feature plan phases 4-5 for Eyra integration:

### Phase 4: Initramfs Integration
- Modify `create_initramfs()` to copy all 14 Eyra utilities
- Handle binary naming conflicts (bare-metal vs Eyra)
- Questions raised: naming strategy, automatic build integration

### Phase 5: Kernel Testing & Validation  
- Test Eyra binaries on actual LevitateOS kernel
- Syscall requirements documented
- Test strategy with 5 levels (basic → complex)
- Expected failure modes and fixes documented

### Resolved Questions
1. **Binary naming:** ✅ Replace bare-metal versions entirely
2. **Build integration:** ✅ Add `--with-eyra` flag to `cargo xtask build all`

### Remaining Question
3. **Syscall coverage:** need kernel syscall audit before Phase 5

