# libsyscall Integration Tests

TEAM_380: Integration test suite for libsyscall with std support via Eyra.

## Overview

This binary runs comprehensive integration tests for libsyscall with full std support through the Eyra runtime. The tests verify that all syscall wrappers, constants, and type definitions match the Linux ABI specification.

## Building

The tests are designed to be cross-compiled for the target architecture:

```bash
# For AArch64
cd /home/vince/Projects/LevitateOS/crates/userspace/eyra
cargo build -p libsyscall-tests --target aarch64-unknown-linux-gnu --release

# For x86_64
cargo build -p libsyscall-tests --target x86_64-unknown-linux-gnu --release
```

**Note**: Cross-compilation requires the appropriate toolchain and sysroot installed:
- AArch64: `aarch64-linux-gnu-gcc` and `gcc-aarch64-linux-gnu` package
- x86_64: Standard host toolchain

## Running

The binary can be run directly on LevitateOS or on a Linux host system:

```bash
# Copy to LevitateOS disk image
cp target/aarch64-unknown-linux-gnu/release/libsyscall-tests /path/to/disk/image

# Or run in QEMU with LevitateOS
cargo xtask vm exec "/bin/libsyscall-tests"
```

## Test Suites

1. **errno_tests**: Verifies errno constants match Linux ABI
2. **integration_tests**: Tests module exports and API surface
3. **memory_tests**: Tests mmap/munmap constants and flags
4. **path_validation**: Tests PATH_MAX and path length handling
5. **time_tests**: Tests time-related constants

## Status

✅ **COMPLETED** - Binary builds successfully for aarch64!

The cross-compilation environment has been set up with:
- Fedora aarch64 sysroot (`sysroot-aarch64-fc43-glibc`)
- Custom build script providing `libgcc_eh.a` and `getauxval()` stubs
- Proper cargo configuration with sysroot path

Built binary: `target/aarch64-unknown-linux-gnu/release/libsyscall-tests` (65KB, statically-linked)

## Prerequisites

To build, you need:
```bash
sudo dnf install -y gcc-aarch64-linux-gnu sysroot-aarch64-fc43-glibc
```

## Next Steps

1. ✅ Set up cross-compilation environment (DONE)
2. Test binary on LevitateOS kernel
3. Add to xtask build system for automated testing
4. Integrate with CI/CD pipeline
