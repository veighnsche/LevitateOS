# TEAM_201: Opus Review (after Iteration 33)

**Date**: 2026-02-04
**Status**: Complete
**Type**: Review

## Scope

Reviewed the last 3 haiku iterations (31-33) covering:
- AcornOS: UKI building, ISO_LABEL constant fix, APK database initialization
- IuppiterOS: udev I/O scheduler rule, operator user creation, iuppiter-engine fix
- distro-spec: sdparm removal from REFURBISHMENT_PACKAGES

## Bugs Found and Fixed

### 1. Duplicate console= parameter in IuppiterOS UKI cmdline

**File**: `IuppiterOS/src/artifact/uki.rs`
**Severity**: Medium (functional — duplicate kernel cmdline params)

The UKI builder added `SERIAL_CONSOLE` (`console=ttyS0,115200n8`) to the base cmdline,
but every UKI entry in `distro-spec::iuppiter::uki` already includes
`console=ttyS0,115200n8` in its `extra_cmdline`. Result: every IuppiterOS UKI
got `console=ttyS0,115200n8` twice in the kernel cmdline.

**Root cause**: IuppiterOS UKI builder was copied from AcornOS pattern but the
distro-spec entries were designed differently. AcornOS entries have empty
`extra_cmdline`, so console params must go in the base cmdline. IuppiterOS
entries include console params in `extra_cmdline`, making the base cmdline
addition redundant.

**Fix**: Removed `SERIAL_CONSOLE` from base cmdline construction. Base now only
contains `root=LABEL=IUPPITER` (live) or `root=LABEL=root rw` (installed).
Updated tests to match.

**Commit**: `fix(iuppiter): remove duplicate console= parameter from UKI cmdline`

## Code Quality Observations (No Action Needed)

- IuppiterOS operator user (`users.rs`): Clean. Group/user parsing is correct.
- IuppiterOS iuppiter-engine placeholder: PID file captures `$$` (parent shell)
  not subshell PID. Acceptable for placeholder per PRD 7.8.
- IuppiterOS udev rule: Correctly targets rotational drives.
- No `distro_spec::acorn` imports remain in IuppiterOS.

## Files Modified

- `IuppiterOS/src/artifact/uki.rs` (1 commit)

## Verification

- `cargo check --workspace`: Clean
- `cargo test -p acornos --lib`: 34 pass
- `cargo test -p iuppiteros`: 22 pass
- `cargo test -p distro-builder`: 60+ pass
- `cargo test -p distro-spec`: 73+ pass
