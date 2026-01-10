# Toolchain Management and Variadic Arguments (VaList)

## Overview
LevitateOS is currently in a state of **toolchain fragmentation**. Different components require different versions of the Rust nightly compiler due to dependencies on unstable features.

## Component Toolchains
- **Kernel & xtask**: Modern nightly (`nightly-x86_64-unknown-linux-gnu`).
- **Eyra & Coreutils**: Pinned nightly (`nightly-2025-04-28`).

## Critical Gotchas

### 1. Variadic Arguments (`VaList`) Breaking Changes
The Rust variadic argument API is highly volatile. A major breaking change occurred between the pinned Eyra toolchain and modern nightlies:
- **Pinned Version (`2025-04-28`)**: `VaList<'a, 'f>` (two lifetime parameters).
- **Modern Nightly**: `VaList<'a>` (one lifetime parameter).

**Impact**: If you build Eyra utilities with the modern nightly (or if `xtask` forces its toolchain on the build), you will see `E0107` errors in crates like `printf-compat` and `c-scape`.

### 2. Environment Variable Pollution
`cargo xtask` runs using the host toolchain. When it spawns child processes (like `cargo build` for Eyra), it may pass down `RUSTUP_TOOLCHAIN` or other environment variables that override the child's `rust-toolchain.toml`.

**Pattern for Sub-builds**:
Always remove toolchain-related environment variables and explicitly specify the toolchain if necessary:
```rust
let mut cmd = Command::new("cargo");
cmd.current_dir(sub_dir)
   .arg("+nightly-2025-04-28") // Force the correct toolchain
   .env_remove("RUSTUP_TOOLCHAIN")
   .args(["build", ...]);
```

### 3. Vendoring Pitfalls
Vendoring a crate (like `printf-compat`) to "fix" toolchain-induced errors is usually a mistake. It creates a "split brain" where:
- The code is modified to match one toolchain.
- But it is actually used by another component expecting the original API.
- Re-syncing with upstream becomes impossible.

**Recommendation**: Always address the **toolchain mismatch** before attempting to patch library code.

## Verification Technique
To verify which toolchain a sub-process is actually using, add a temporary check:
```bash
cargo --version # Should match the pinned toolchain, not the xtask toolchain
```

## Pattern: Workspace Profiles
When moving sub-packages into a Cargo workspace (as done for Eyra in `TEAM_371`), move all `[profile.release]` and `[unstable]` settings to the **workspace root** `Cargo.toml`. Cargo ignores these settings in sub-packages once they are part of a workspace.

## Eyra Workspace Structure (TEAM_378)

The Eyra coreutils live in `crates/userspace/eyra/` as a Cargo workspace:

```
crates/userspace/eyra/
├── .cargo/config.toml      ← Workspace build config (static-pie flags)
├── .gitignore              ← Prevents per-utility stale artifacts
├── Cargo.toml              ← Workspace manifest
├── Cargo.lock              ← Single authoritative lock file
├── target/                 ← Single shared build directory
├── rust-toolchain.toml     ← Pinned to nightly-2025-04-28
├── eyra-test-runner/       ← Tests Eyra std library
├── eyra-hello/             ← Minimal Eyra sanity check binary
├── cat/, cp/, echo/, ...   ← Individual utilities
```

**Key points:**
- All utilities share the workspace `target/` directory
- Per-utility `target/`, `Cargo.lock`, and `.cargo/` are forbidden (see `.gitignore`)
- Each utility has a `build.rs` that adds `-nostartfiles` (Eyra provides `_start`)
- The `init` process runs coreutils tests after `eyra-test-runner` completes
