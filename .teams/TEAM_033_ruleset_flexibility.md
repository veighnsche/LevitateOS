# TEAM_033: Ruleset Flexibility Refactor

## Objective
Codify the Rust kernel development rules into the build system using workspace-level lints and compiler flags to ensure high-fidelity enforcement of Safety, Robustness, and idiomatic Rust practices.

## Status
- [x] Claimed TEAM_033
- [x] Read and generalized rulesets
- [x] Researched and integrated 2025 modern best practices
- [x] Re-integrated Unix Philosophy (McIlroy's Laws)
- [x] **Codified Rules**:
    - Implemented `[workspace.lints]` in root `Cargo.toml`.
    - Enforced Rule 5 (Safety): `unsafe_code = "deny"`, `missing_safety_doc = "deny"`.
    - Enforced Rule 6 (Robustness): `unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"`.
    - Enforced Rule 13 (Representation): `match_same_arms = "deny"`.
    - Enforced Rule 21 (Economy): `pedantic` lints enabled across workspace.
    - Configured all crates (`kernel`, `levitate-hal`, `levitate-utils`, `xtask`) to inherit workspace lints.
- [x] Final review and handoff

## Changes
- **Updated root `Cargo.toml`**: Added comprehensive lint suite under `[workspace.lints]`.
- **Updated crate `Cargo.toml` files**: Added `lints.workspace = true` to all members.
- **Updated `kernel-development.md`**: Added a new section "VI. Enforcement" documenting the automated checks.
