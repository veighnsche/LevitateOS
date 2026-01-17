# TEAM_017: Void Linux musl Research Documentation

## Status: COMPLETE

## Goal
Document how Void Linux implements musl support as a reference for LevitateOS.

## Tasks
- [x] Create team file
- [x] Create documentation file with research findings

## Output
- `docs/void-linux-musl-reference.md` - Full research documentation

## Key Findings
1. Void uses runit (not systemd) specifically because systemd doesn't work with musl
2. **Void uses GNU coreutils on musl** - patched, not replaced with busybox/uutils
3. Void maintains supplementary libraries: musl-fts, musl-obstack, musl-legacy-compat
4. Aggressive patching strategy with upstream contributions
5. Clear documentation of what doesn't work (NVIDIA, proprietary, V8, multilib)

## Important Context
**Current LevitateOS:** glibc + systemd + GNU coreutils
**Future LevitateOS:** musl + runit + GNU coreutils (patched for musl)
