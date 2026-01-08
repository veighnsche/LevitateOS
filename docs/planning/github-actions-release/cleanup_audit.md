# TEAM_294: Runner Cleanup Audit Findings

## Current State Analysis
Standard GitHub-hosted runners (`ubuntu-latest`) are ephemeral and destroyed after each job. However, the user is testing locally with `act`, which persists container state or requires manual cleanup if not configured correctly.

### `release.yml` Observations
- **No Explicit Cleanup:** There are no `post` steps to remove build artifacts (`target/`, `initramfs.cpio`, `tinyos_disk.img`).
- **Disk Usage:** Rust builds, especially with `build-std`, can consume several gigabytes in `target/`.
- **Artifacts:** Large images like `tinyos_disk.img` are created in the workspace.

### `xtask/src/clean.rs` Observations
- **Logic:** Only kills QEMU processes and removes `qmp.sock`.
- **Missing:** Does not remove `target/` directories, `.cpio` files, or `.img` files.

## Hypotheses
1. **Artifact Accumulation:** Local `act` runs will leave behind large files (`.iso`, `.img`, `.cpio`) in the project root.
2. **Target Bloat:** The `target/` directories are not cleared between local runs unless the user manually runs `cargo clean`.

## Recommendations
- **Add a Cleanup Step:** Add a final step to the workflow to remove temporary artifacts.
- **Enhance `xtask clean`:** Update `xtask` to remove generated images and archives.
- **Use `.gitignore`:** Ensure generated artifacts are ignored to prevent accidental commits (verified: they are mostly ignored, but cleanup is still good hygiene).
