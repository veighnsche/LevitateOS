# TEAM_291: Investigate AArch64 Cross-Compilation Requirements

## Goal
Investigate and verify the AArch64 cross-compilation requirements for the GitHub Actions release workflow.

## Context
The user pointed out that AArch64 builds require cross-compilation tools. I need to ensure the `.github/workflows/release.yml` and the underlying `xtask` build system correctly handle cross-compilers, linkers, and other necessary toolchain components for a non-native build environment (GitHub's `ubuntu-latest`).

## Progress Tracking
- [x] Team Registration
- [x] Understand Symptom / Requirements
- [x] Form Hypotheses
- [x] Test Hypotheses with Evidence
- [x] Narrow Down to Root Cause
- [x] Decision: Fix or Plan

## Investigation Summary
Confirmed that while the workflow installed the AArch64 cross-toolchain, the Cargo configuration lacked an explicit linker definition for the `aarch64-unknown-none` target. This would cause the build to fail on non-AArch64 hosts (like GitHub Runners) as it would attempt to use the host's native linker.

## Actions Taken
- Updated `.cargo/config.toml` to specify `aarch64-linux-gnu-gcc` as the linker for `aarch64-unknown-none`.
- This matches the `gcc-aarch64-linux-gnu` package already present in the GitHub Actions workflow.
