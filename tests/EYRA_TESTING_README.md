# Eyra Userspace Integration - Testing Documentation

This document describes the comprehensive test suite for the Eyra userspace integration, designed to freeze and verify the current implementation state.

## Overview

The Eyra integration test suite covers 51 distinct behaviors across 4 test categories:
- **36 Eyra integration behaviors** (EY1-EY36)
- **15 LibSyscall behaviors** (LS1-LS15)

All tests follow the behavior-driven testing methodology documented in `.agent/rules/behavior-testing.md`.

## Test Categories

### 1. Unit Tests
**Location**: `crates/userspace/eyra/libsyscall/tests/integration_tests.rs`  
**Run with**: `cargo test --test integration_tests --features std`

Tests the libsyscall library in isolation with std support:
- Syscall number constants (LS13-LS15)
- Error handling conventions (LS10-LS12)
- Architecture-specific syscall conventions (LS5, LS8)
- No-std compatibility (LS22)

**Traceability**: Each test includes `[LSxx]` markers linking behavior to implementation.

### 2. Regression Tests
**Location**: `tests/eyra_regression_tests.rs`  
**Run with**: `cargo test --test eyra_regression_tests`

Verifies cross-file consistency and configuration:
- Workspace cargo configuration (EY6-EY10)
- Dependency specifications (EY19-EY25)
- Documentation completeness (EY26-EY30)
- Build system setup (nostartfiles, sysroot)

**Purpose**: Catch configuration drift and ensure setup is reproducible.

### 3. Integration Tests
**Location**: `tests/eyra_integration_test.rs`  
**Run with**: `cargo test --test eyra_integration_test -- --ignored`

Tests the complete build pipeline:
- Sysroot configuration (EY5)
- Stub creation (libgcc_eh, getauxval) (EY11-EY13)
- Binary format verification (EY2-EY4, EY14-EY18)
- Full build from scratch (EY1)
- x86_64 expected failure (EY34)

**Note**: These tests are marked `#[ignore]` because they're expensive. Run manually when needed.

### 4. Behavior Tests
**Location**: `tests/eyra_behavior_test.sh`  
**Run with**: `./tests/eyra_behavior_test.sh`

Tests LevitateOS integration:
- Initramfs inclusion (EY31-EY33)
- Binary spawning on LevitateOS (EY35)
- Known crash behavior documentation (EY36)

**Note**: Requires QEMU and a built LevitateOS kernel.

## Running All Tests

### Quick Run (Regression + Unit)
```bash
./tests/run_eyra_tests.sh
```

This runs all fast tests and skips expensive integration/behavior tests if tools are missing.

### Complete Run (All Tests)
```bash
# Ensure prerequisites
sudo dnf install binutils aarch64-linux-gnu-gcc qemu-system-aarch64

# Build the test binary first
cd crates/userspace/eyra
cargo build --release --target aarch64-unknown-linux-gnu -p libsyscall-tests

# Run complete suite
cd ../..
STRICT_MODE=1 ./tests/run_eyra_tests.sh
```

With `STRICT_MODE=1`, the test runner stops on first failure.

### Run Individual Test Categories
```bash
# Unit tests only
cargo test --test integration_tests --manifest-path crates/userspace/eyra/libsyscall/Cargo.toml --features std

# Regression tests only
cargo test --test eyra_regression_tests

# Integration tests (expensive)
cargo test --test eyra_integration_test -- --ignored

# Behavior tests (requires QEMU)
./tests/eyra_behavior_test.sh
```

## Test Traceability

Every test includes behavior IDs that trace back to:
1. **Behavior Inventory**: `crates/userspace/eyra/BEHAVIOR_INVENTORY.md`
2. **Source Code**: Comments in implementation files
3. **Test Code**: Test function doc comments and assertions

Example:
```rust
/// Tests: [EY6] workspace config contains -nostartfiles
#[test]
fn test_nostartfiles_in_workspace_config() {
    // [EY6] Verify -nostartfiles is in workspace .cargo/config.toml
    let content = fs::read_to_string("crates/userspace/eyra/.cargo/config.toml")?;
    assert!(content.contains("-nostartfiles"));
}
```

Grep for `[EY6]` to find:
- Where the behavior is documented
- Where it's implemented  
- Where it's tested

## Behavior Coverage

### Build System (EY1-EY5)
- ✅ Cross-compilation for aarch64
- ✅ Static linking verification
- ✅ Binary size constraints
- ✅ Sysroot configuration

### Linker Configuration (EY6-EY10)
- ✅ Workspace-level -nostartfiles
- ✅ No duplication in build.rs
- ✅ Static PIE flags
- ✅ Sysroot path correctness

### Build Artifacts (EY11-EY13)
- ✅ libgcc_eh.a stub creation
- ✅ getauxval stub linking
- ✅ Clean build without errors

### Binary Properties (EY14-EY18)
- ✅ Entry point in valid range
- ✅ LOAD segments at expected addresses
- ✅ Correct segment permissions
- ✅ No dynamic interpreter

### Dependencies (EY19-EY22)
- ✅ Correct dependency versions
- ✅ Optional eyra dependency
- ✅ Feature flag configuration
- ✅ No-std compatibility

### Workspace (EY23-EY25)
- ✅ Workspace membership
- ✅ Resolver version

### Documentation (EY26-EY30)
- ✅ NOSTARTFILES_README.md
- ✅ X86_64_STATUS.md
- ✅ TEAM log files

### Integration (EY31-EY36)
- ✅ Initramfs inclusion
- ✅ Executable permissions
- ✅ Binary count verification
- ⚠️ Spawns successfully (PID assigned)
- ⚠️ Crashes at 0x0 (documented kernel bug)

### LibSyscall Core (LS1-LS15)
- ✅ Syscall wrappers
- ✅ Architecture abstraction
- ✅ Error handling
- ✅ Const safety

## Known Limitations

### EY35/EY36: Execution Crash
The libsyscall-tests binary successfully spawns on LevitateOS but crashes at address 0x0 due to a kernel bug in `enter_user_mode`. This is **documented** and **expected** until the kernel fix is implemented.

**Status**: ⚠️ Known issue, not a test failure  
**Documentation**: `.teams/TEAM_382_libsyscall_eyra_integration_test.md`

### EY34: x86_64 Not Supported
The x86_64 build intentionally fails because LevitateOS is aarch64-only and x86_64 Eyra integration is not required.

**Status**: ✅ Expected failure, documented in `X86_64_STATUS.md`

## Test Maintenance

### When Tests Fail

Follow the guidelines in `.agent/rules/behavior-testing.md`:

1. **STOP** - Do not dismiss failures
2. **INVESTIGATE** - Read the test, understand what it checks
3. **DETERMINE ROOT CAUSE**:
   - Your changes broke it? → Fix your code
   - Intentional change? → Update the test/baseline
   - Test is buggy? → Fix the test
   - Truly unrelated? → PROVE IT with git blame

4. **NEVER CONTINUE WITH FAILING TESTS**

### Adding New Behaviors

1. Add to `BEHAVIOR_INVENTORY.md` with unique ID
2. Add `[ID]` comment to source code
3. Write test with `[ID]` in doc comment
4. Run tests to verify traceability

### Updating Golden Files

If behavior intentionally changes:
```bash
# For behavior tests
./tests/eyra_behavior_test.sh  # Review differences
# If correct, update BEHAVIOR_INVENTORY.md with new expected values
```

## CI/CD Integration

To integrate into continuous integration:

```bash
#!/bin/bash
# .github/workflows/eyra-tests.yml equivalent

set -e

# Install dependencies
sudo dnf install -y binutils aarch64-linux-gnu-gcc sysroot-aarch64-fc43-glibc

# Run non-QEMU tests (fast)
cargo test --test integration_tests --features std
cargo test --test eyra_regression_tests

# Optional: Run expensive tests on nightly builds
cargo test --test eyra_integration_test -- --ignored
```

## References

- **Behavior Testing Rules**: `.agent/rules/behavior-testing.md`
- **Behavior Inventory**: `crates/userspace/eyra/BEHAVIOR_INVENTORY.md`
- **Main Inventory**: `docs/testing/behavior-inventory.md` (Group 19)
- **Integration Results**: `.teams/TEAM_382_libsyscall_eyra_integration_test.md`
- **Cross-compilation**: `.teams/TEAM_380_setup_aarch64_cross_compilation.md`
- **Nostartfiles**: `.teams/TEAM_381_centralize_nostartfiles_config.md`

## Success Criteria

A complete test run is successful when:
- ✅ All unit tests pass
- ✅ All regression tests pass
- ✅ All integration tests pass (if tools available)
- ✅ Behavior tests show expected results (spawn + documented crash)
- ✅ No unexpected test skips

The Eyra integration is considered **frozen** when all these tests pass consistently.
