# TEAM_190: Rename coreutils to levbox

**Date**: 2026-01-06
**Status**: ✅ Complete

## Task

Rename "coreutils" to "levbox" throughout the repo to avoid confusion with uutils-coreutils and align with busybox-style naming.

## Rationale

- Future integration with uutils-coreutils would cause naming conflicts
- "levbox" follows busybox naming convention (LevitateOS + box)
- Clearer distinction between external coreutils and LevitateOS built-ins

## Changes Made

### Directories Renamed
- `userspace/coreutils/` → `userspace/levbox/`
- `docs/specs/coreutils/` → `docs/specs/levbox/`

### Files Updated

| File | Change |
|------|--------|
| `userspace/levbox/Cargo.toml` | Package name: coreutils → levbox |
| `userspace/Cargo.toml` | Workspace member: coreutils → levbox |
| `userspace/levbox/build.rs` | Linker path: coreutils/link.ld → levbox/link.ld |
| `userspace/levbox/src/bin/cat.rs` | Doc reference updated |
| `docs/specs/levbox/*.md` | All coreutils references → levbox |
| `docs/planning/*.md` | References updated |
| `docs/ROADMAP.md` | References updated |
| `.teams/TEAM_*.md` | References updated |

## Verification

- `cargo xtask build all` ✅
- `cargo xtask test behavior` ✅

## Handoff

- [x] Directories renamed
- [x] Cargo.toml files updated
- [x] All documentation references updated
- [x] Build passes
- [x] Behavior tests pass
- [x] Team file created
