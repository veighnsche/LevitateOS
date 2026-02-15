# Technical Debt

This file tracks deliberate shortcuts and transitional shims we want to remove.

## `output/` Symlink Shim (Per-Distro)

**Current state**

- Canonical build outputs live under `.artifacts/out/<DistroDir>/...`.
- For compatibility, each distro directory also has:
  - `<DistroDir>/output -> ../.artifacts/out/<DistroDir>`

This keeps older scripts/paths working while we centralize artifacts.

**Why this is debt**

- Hidden indirection is confusing (`output/` looks like a real directory).
- Some code/recipes still hardcode `output/...` instead of using the centralized
  output helper (`distro_builder::artifact_store::central_output_dir_for_distro`).
- It makes “standalone submodule” vs “superrepo” behavior harder to reason about.

**Plan to remove**

1. Replace `output/...` assumptions in Rust code with
   `central_output_dir_for_distro()` (SSOT path computation).
2. Replace `output/...` assumptions in `.rhai` recipes:
   - Either compute the centralized output path directly, or
   - Pass it into recipes explicitly (preferred if/when recipe supports it).
3. Update docs/scripts that reference `<DistroDir>/output/...`.
4. Delete the symlinks once the repo no longer relies on them.

