# TEAM_294: Audit Runner Cleanup

## Goal
Verify if the GitHub Actions workflow and `xtask` are performing proper cleanup in the CI runners to avoid artifact leakage or disk space issues.

## Context
The user asked if they are doing proper cleanup in the runners. Standard GitHub-hosted runners are ephemeral, but long-running builds or self-hosted runners can suffer from disk bloat.

## Progress Tracking
- [x] Team Registration
- [x] Audit `release.yml` for Cleanup
- [x] Audit `xtask/src/clean.rs`
- [x] Identify Leaks/Improvements
- [x] Implement Fixes

## Final Implementation Details
1. **`xtask clean` Enhancement:** Updated `@/home/vince/Projects/LevitateOS/xtask/src/clean.rs` to remove all generated binary artifacts (`.cpio`, `.img`, `.iso`, `.bin`) and staging directories (`initrd_root`, `iso_root`, etc.).
2. **Workflow Cleanup Step:** Added a "Cleanup workspace" step to `@/home/vince/Projects/LevitateOS/.github/workflows/release.yml` that runs `cargo xtask clean` at the end of both `build-x86_64` and `build-aarch64` jobs.
3. **`if: always()` Guard:** The cleanup step uses `if: always()` to ensure the runner is cleaned even if the build or test steps fail.
4. **Ordering:** The cleanup step is placed **after** the "Upload Artifacts" step to ensure artifacts are preserved in GitHub's storage before local deletion.
