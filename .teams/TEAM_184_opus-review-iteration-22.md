# TEAM_184: Opus Review after Iteration 22

**Date**: 2026-02-04
**Status**: Complete
**Type**: Review (no code changes)

## Scope

Reviewed last 3 haiku iterations (20-22) covering:

- **AcornOS** (3 commits): EROFS size reduction, sshd_config commented-line fix, profile.d test instrumentation copy
- **IuppiterOS** (3 commits): initramfs command implementation, pervasive copy-paste fix (14 files), test instrumentation with ___SHELL_READY___
- **distro-builder**: No new commits
- **distro-spec**: No new commits

## Verification

| Crate | Tests | Result |
|-------|-------|--------|
| acornos | 31 unit + 1 doc | Pass |
| iuppiteros | 18 unit + 1 doc | Pass |
| distro-builder | 25 unit + 2 doc | Pass |
| distro-spec | 73 unit + 3 doc | Pass |
| workspace check | - | Clean |

## Findings

**No bugs found.** The last 3 iterations produced clean, correct code:

1. sshd_config line-by-line fix is correct (avoids matching commented lines)
2. EROFS size reduction is well-scoped (CopyWifiFirmware only, packages still installed)
3. profile.d copy loop handles edge cases properly
4. IuppiterOS initramfs command matches AcornOS pattern
5. Copy-paste fix is comprehensive — verified no remaining `distro_spec::acorn` imports
6. Test instrumentation script is ash-compatible with proper markers

**Observation**: IuppiterOS has redundant inittab configs (definitions.rs LIVE_FINAL vs iso.rs overlay). Overlay wins at runtime. Not a bug, just redundancy.

## Files Modified

None — clean review, no fixes needed.

## Key Decisions

- No changes warranted. Previous opus review (iteration 20) fixed the critical copy-paste bugs, and haiku's subsequent work was clean.

## PRD Status

Phases 1-4 complete for both AcornOS and IuppiterOS. Phase 5 (ISO build) is next.
