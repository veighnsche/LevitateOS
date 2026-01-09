# TEAM_352 — ELF Loader Feature

**Created:** 2026-01-09  
**Status:** Planning

## Objective

Implement a dynamic ELF loader for LevitateOS to support dynamically linked binaries (PIE executables with relocations).

## Context

- **TEAM_351** built Eyra test binary successfully
- Binary is dynamically linked (PIE with interpreter)
- LevitateOS cannot execute it without an ELF loader
- User decided Option 3 (ELF loader) is the correct long-term solution

## Planning Documents

Location: `docs/planning/elf-loader/`

| Phase | File | Purpose |
|-------|------|---------|
| 1 | `phase-1.md` | Discovery - ELF format, current loader |
| 2 | `phase-2.md` | Design - Loader architecture |
| 3 | `phase-3.md` | Implementation |
| 4 | `phase-4.md` | Integration & Testing |
| 5 | `phase-5.md` | Polish & Documentation |

## Progress

- [x] Phase 1: Discovery
- [x] Phase 2: Design
- [x] Phase 3: Implementation (plan ready)
- [ ] Phase 4: Integration
- [ ] Phase 5: Polish

## Summary

Complete feature plan created for implementing ELF dynamic loader.

**Key Decision:** Use `goblin` crate instead of hand-rolled ELF parser (per library-audit.md)

**Estimated effort:** 4-6 hours

**Key implementation steps:**
1. Add `goblin` dependency to kernel Cargo.toml
2. Refactor elf.rs to use goblin for parsing
3. Implement load base offset (0x10000 for PIE)
4. Process R_*_RELATIVE relocations using goblin constants
5. Test with eyra-hello

## Best Practices Found

- **Theseus** uses goblin for relocation constants (`goblin::elf::reloc::*`)
- **Theseus** has arch-specific `write_relocation_arch()` functions
- **Library audit** already recommends goblin as HIGH priority replacement

See `docs/planning/elf-loader/best-practices.md` for details.
