# Questions and Answers Audit: GitHub Actions Release

## Answered Questions
- **Q288.1: Versioning of dependencies**
    - **User Answer:** Use latest in `ubuntu-latest`.
    - **Plan Reflection:** `release.yml` uses `ubuntu-latest` and `apt-get install` without version pins. Correct.
- **Q288.2: Running tests in CI**
    - **User Answer:** Yes, run `cargo xtask test unit`. Headless QEMU is hard.
    - **Plan Reflection:** `release.yml` includes `cargo xtask test unit` for both architectures. Correct.
- **Q288.3: AArch64 bootable image format**
    - **User Answer:** Provide `kernel64_rust.bin` and `initramfs.cpio`.
    - **Plan Reflection:** `release.yml` uploads both as artifacts. Correct.

## Open Questions from Plan
- **Should we include `tinyos_disk.img` in the release?**
    - **Status:** The current implementation in `release.yml` **does** include it in artifacts and the final release.
    - **Observation:** This seems to have been decided by implementation rather than explicitly answered in the docs.
- **Do we need to sign artifacts?**
    - **Status:** Plan says "Likely no for now". Current implementation does not sign.

## New Discrepancies/Observations
- **Missing `libgcc-s1`?** For AArch64, `gcc-aarch64-linux-gnu` is installed, but sometimes `libgcc-s1:aarch64` or similar is needed if the build isn't fully static. However, `xtask` uses `aarch64-linux-gnu-objcopy` which is included in the compiler package.
- **Limine Binaries:** `xtask` downloads Limine binaries via `curl` if missing. `release.yml` installs `curl`. This aligns.
- **ISO for AArch64:** Phase 5 mentions this as a remaining item.
