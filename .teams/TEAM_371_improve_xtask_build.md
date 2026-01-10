# TEAM_371: Improve xtask Build Output and Efficiency

## Objective
- Ensure `uutils` are not rebuilt unnecessarily.
- Expose full Rust build output in `xtask`.

## Status
- [x] Investigate `xtask` build logic for `uutils`.
- [x] Identify output suppression.
- [x] Identify redundant rebuild causes.
- [x] Implement fixes (Consolidated workspace build + Skip logic + Status output).
- [x] Create Eyra workspace for efficient dependency management.
- [x] Update initramfs logic for new target directory structure.
- [!] Struggling with toolchain fragmentation and `VaList` lifetime changes.

## Struggles and Findings
1.  **Toolchain Fragmentation**: The kernel and `xtask` use a modern nightly, while `eyra` and its coreutils require a specific pinned toolchain (`nightly-2025-04-28`).
2.  **`VaList` Breaking Changes**: Rust changed `VaList` from two lifetimes to one in recent nightlies. This causes E0107 errors in `printf-compat` and `c-scape` when built with the wrong toolchain.
3.  **Environment Pollution**: `xtask` was inheriting `RUSTUP_TOOLCHAIN`, forcing its modern toolchain on the `eyra` workspace, which triggered these breaking changes.
4.  **Vendoring Pitfalls**: I attempted to vendor `printf-compat` and `c-scape` to "fix" them, but the root cause was the toolchain mismatch. Vendoring created a "split brain" where code meant for one version was being checked by another.
5.  **API Volatility**: Methods like `as_va_list()` and the `Clone` implementation for `VaList` vary significantly between nightly releases.

## Current State
- `xtask` is now efficient (skips builds, shows output).
- Eyra workspace is established.
- Build is currently broken due to the `VaList` lifetime issues in the vendored crates.
- **Recommendation**: Roll back vendoring and strictly enforce toolchain separation in `xtask`.
