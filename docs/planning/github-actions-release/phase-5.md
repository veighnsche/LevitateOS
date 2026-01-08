# Phase 5: Polish, Docs, and Cleanup - CI/CD Pipeline

## Implementation Summary
- Created `.github/workflows/release.yml` with support for x86_64 and AArch64.
- Automated unit testing for both architectures.
- Artifact generation for:
    - **x86_64:** `levitate.iso`, `levitate-kernel`, `initramfs.cpio`, `tinyos_disk.img`.
    - **AArch64:** `kernel64_rust.bin`, `initramfs.cpio`, `tinyos_disk.img`.
- Automated release creation on tag (e.g., `v0.1.0`) with checksums.

## How to use
1. **Pushes to `main`** or **Pull Requests** will trigger the build and test jobs.
2. **Tags** starting with `v` (e.g., `v1.0.0`) will trigger the release job, which collects artifacts from both builds and creates a GitHub Release.

## Maintenance
- The workflow uses `ubuntu-latest` and installs dependencies like `mtools`, `xorriso`, and `gcc-aarch64-linux-gnu` via `apt`.
- Rust toolchain is managed by `dtolnay/rust-toolchain` and follows the `rust-toolchain.toml` in the project root.

## Remaining Items
- [x] Create `.github/workflows/release.yml` with support for x86_64 and AArch64.
- [x] Automated unit testing for both architectures.
- [x] Artifact generation for x86_64 and AArch64.
- [x] Automated release creation on tag with checksums.
