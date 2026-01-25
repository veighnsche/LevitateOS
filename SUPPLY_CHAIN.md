# Supply Chain

## Package Sources

LevitateOS extracts packages from Rocky Linux 10 official repositories. Rocky Linux is a community-driven enterprise Linux distribution, binary-compatible with Red Hat Enterprise Linux.

## Verification

Every RPM is GPG-signed by Rocky Linux. Verify signatures with:

```bash
rpm -K /path/to/package.rpm
```

Rocky's public keys are available at https://rockylinux.org/keys/

## What We Do

1. Download RPMs from official Rocky Linux mirrors
2. Verify GPG signatures match Rocky's public keys
3. Extract package contents using `rpm --root --nodeps`
4. Assemble extracted files into squashfs image

## What We Don't Do

- We don't recompile packages
- We don't inject code
- We don't modify binaries
- We don't strip signatures
- We don't add patches

## What You Can Verify

1. **Compare squashfs contents against Rocky repos**: Extract our squashfs and compare file checksums against packages from Rocky mirrors
2. **Check RPM signatures**: Run `rpm -K` on any package in our cache
3. **Audit the extraction code**: See `leviso/src/component/packages.rs`
4. **Verify the build process**: Run `cargo run -- build` and inspect outputs

## Build Reproducibility

Given the same:
- Rocky Linux mirror state
- LevitateOS commit
- Build environment

The resulting ISO should contain identical package contents. The squashfs and ISO may differ in timestamps, but extracted files will match.

## Upstream Trust

LevitateOS inherits trust from Rocky Linux, which inherits from RHEL. The chain:

```
Red Hat (builds RHEL)
    ↓
Rocky Linux (rebuilds, signs with Rocky keys)
    ↓
LevitateOS (extracts, no modification)
```

If you trust Rocky Linux, you can trust LevitateOS packages. If you don't trust Rocky, you shouldn't use LevitateOS.

## Reporting Issues

If you find a discrepancy between Rocky packages and what LevitateOS ships, open an issue with:
1. Package name and version
2. Expected checksum (from Rocky)
3. Actual checksum (from LevitateOS)
4. Steps to reproduce
