# Phase 2: Design - CI/CD Pipeline

## Proposed Solution
A GitHub Actions workflow `.github/workflows/release.yml` that:
1. Runs on `push` to `main`, `pull_request`, and `workflow_dispatch`.
2. Has two main jobs: `build-x86_64` and `build-aarch64`.
3. A `release` job that depends on both build jobs (only runs on tags).

## Workflow Steps (High Level)
1. **Setup:**
   - Checkout code.
   - Install Rust nightly (as per `rust-toolchain.toml`).
   - Install system dependencies (`mtools`, `xorriso`, `binutils-aarch64-linux-gnu`).
2. **Build:**
   - `cargo xtask build all --arch <arch>`
   - `cargo xtask build iso --arch x86_64` (x86_64 only).
3. **Artifact Upload:**
   - Upload ISO, kernels, and disk images as workflow artifacts.
4. **Release (Optional):**
   - On tag push, create a GitHub Release and upload all artifacts.

## Behavioral Decisions & Questions
- **Q288.1:** Should we use a specific version of `xorriso` or `mtools`?
  - *Recommendation:* Use the latest available in Ubuntu `latest` runner.
- **Q288.2:** Do we want to run tests in the CI as well?
  - *Recommendation:* Yes, `cargo xtask test unit` should run. Functional tests (QEMU) are harder in standard runners without KVM but can be attempted headless.
- **Q288.3:** For AArch64, do we need a bootable image format other than the raw binary?
  - *Current state:* `kernel64_rust.bin` is used. We should probably provide this and the `initramfs.cpio`.

- **Q288.4: Should we include `tinyos_disk.img` in the release?**
  - *Decision:* Yes, it contains userspace apps and is required for a complete experience on some platforms (e.g. AArch64).
- **Q288.5: Do we need to sign artifacts?**
  - *Decision:* No, SHA256 checksums are sufficient for now.
