# Team 034 - Fix Missing Core Crate

## Task context
The project is failing to compile because the `aarch64-unknown-none` target is missing the `core` crate. This typically happens when the target is not installed via `rustup`.

## Status
- [x] Investigate environment: Found that global `build-std` and `target` in `.cargo/config.toml` were causing conflicts between host tools (`xtask`) and the kernel.
- [x] Apply Workspace Harmonization: Separated host and target build logic.
- [x] Refine IDE diagnostics: Used `rust-analyzer.check.extraArgs` with `["--exclude", "xtask"]` to prevent host-only dependencies (`anstyle`) from being checked against the bare-metal target.
- [x] Answered user questions: Explained the trade-offs of switching targets and the risks of silencing errors.
- [x] Audit Lint Configuration: Refined lints and verified that `missing_safety_doc = "deny"` is correctly enforcing project standards.
