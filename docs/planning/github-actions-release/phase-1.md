# Phase 1: Discovery - CI/CD Pipeline

## Feature Summary
Implement a GitHub Actions workflow to automate building LevitateOS for x86_64 and AArch64. The pipeline will generate and release ISOs (for x86_64) and other artifacts (kernels, initramfs, disk images) for both architectures.

## Success Criteria
- [ ] GitHub Actions workflow triggers on push to `main` and manual trigger.
- [ ] x86_64 build produces `levitate.iso` and `levitate-kernel`.
- [ ] AArch64 build produces `kernel64_rust.bin` and `initramfs.cpio`.
- [ ] Artifacts are uploaded to the action run.
- [ ] Releases are automatically created on tag with attached artifacts.

## Current State Analysis
- **Build System:** Uses `cargo xtask` (Rust).
- **Architectures:** Supports `x86_64` and `aarch64`.
- **Dependencies:** 
    - `cargo` (nightly with `build-std`).
    - `mtools` (`mformat`, `mcopy`) for disk image creation.
    - `xorriso` for ISO creation.
    - `cpio` and `find` for initramfs.
    - `aarch64-linux-gnu-objcopy` for AArch64 kernel conversion.
    - `curl` to fetch Limine binaries.
- **Artifacts:**
    - x86_64: `levitate.iso`, `target/x86_64-unknown-none/release/levitate-kernel`.
    - AArch64: `kernel64_rust.bin`, `initramfs.cpio`, `tinyos_disk.img`.

## Codebase Reconnaissance
- `xtask/src/build.rs`: Contains the logic for building kernels, userspace, initramfs, and ISOs.
- `xtask/src/image.rs`: Handles disk image creation.
- `rust-toolchain.toml`: Defines the required nightly toolchain and components.

## Constraints
- GitHub Runners (Ubuntu) need specific packages installed (`mtools`, `xorriso`, `gcc-aarch64-linux-gnu`).
- ISO build is currently only supported for x86_64.
