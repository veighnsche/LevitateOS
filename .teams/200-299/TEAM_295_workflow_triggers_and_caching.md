# TEAM_295: Workflow Trigger and Caching Optimization

## Goal
Fix the GitHub Actions trigger logic and implement efficient Rust caching.

## Context
The user noted that the workflow is not automatically running on every push and requested caching to speed up builds.

## Progress Tracking
- [x] Team Registration
- [x] Fix Trigger Logic
- [x] Implement `rust-cache`
- [x] Verify Configuration

## Final Improvements
1. **Trigger Fix:** Changed `branches: [ main ]` to `branches: [ "**" ]` for both `push` and `pull_request` events. This ensures CI runs on every branch, not just `main`.
2. **Rust Caching:** Integrated `swatinem/rust-cache@v2` for both x86_64 and AArch64 jobs.
3. **Arch-Specific Cache Keys:** Used `key: x86_64` and `key: aarch64` to prevent cache collisions between different architecture build artifacts.
4. **Build-Std Compatibility:** Confirmed `rust-cache` works correctly with `cargo build -Z build-std`.
