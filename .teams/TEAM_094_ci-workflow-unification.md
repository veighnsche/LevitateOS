# TEAM_094: CI Workflow Unification

**Status:** Complete
**Started:** 2026-01-23
**Completed:** 2026-01-23

## Objective

Unify duplicated CI configurations across 10+ Rust submodules using GitHub Actions reusable workflows.

## Problem

- ~144 lines copy-pasted across simple crates
- ~220-290 lines for tools with custom tests
- Inconsistencies: cache keys, branch names, Cargo.lock handling, `continue-on-error`

## Solution

Created two reusable workflows in the main repo:
- `.github/workflows/rust-ci.yml` - test, clippy, fmt, msrv
- `.github/workflows/rust-release.yml` - version bump, tag, publish

Submodules now call these with ~35 lines instead of 144+.

## Files Created

| File | Lines | Purpose |
|------|-------|---------|
| `.github/workflows/rust-ci.yml` | 88 | Reusable CI: test, clippy, fmt, msrv |
| `.github/workflows/rust-release.yml` | 90 | Reusable release: bump, tag, publish |

## Files Modified

| File | Before | After | Reduction |
|------|--------|-------|-----------|
| `leviso-elf/.github/workflows/ci.yml` | 143 | 35 | 75% |
| `leviso-deps/.github/workflows/ci.yml` | 144 | 35 | 76% |
| `testing/cheat-guard/.github/workflows/ci.yml` | 144 | 35 | 76% |
| `testing/cheat-test/.github/workflows/ci.yml` | 144 | 35 | 76% |
| `tools/recstrap/.github/workflows/ci.yml` | 287 | 108 | 62% |
| `tools/recfstab/.github/workflows/ci.yml` | 238 | 43 | 82% |
| `tools/recchroot/.github/workflows/ci.yml` | 227 | 42 | 81% |
| `tools/recipe/.github/workflows/ci.yml` | 37 | 38 | +3% (added release automation) |
| `testing/install-tests/.github/workflows/ci.yml` | 37 | 13 | 65% |

## Inconsistencies Fixed

| Issue | Fix |
|-------|-----|
| Cache key variation | Standardized: `Cargo.lock` + `Cargo.toml` |
| Missing restore keys | Added to all caches |
| Branch check inconsistency | Support both `master` and `main` |
| `continue-on-error: true` | Removed from all |
| Missing MSRV checks | Added to all crates |

## Notes

- llm-toolkit is Python, not Rust - skipped
- Trusted publishing (OIDC) deferred - requires per-crate crates.io setup
- recstrap retains E2E tests as separate job (too complex to abstract)
- install-tests has no release job (internal test crate)

## Total Line Count

**Before:** ~1,401 lines across 9 submodule CI files
**After:** ~384 lines in submodule CI files + 178 lines in reusable workflows = 562 total
**Reduction:** ~60% fewer lines to maintain
