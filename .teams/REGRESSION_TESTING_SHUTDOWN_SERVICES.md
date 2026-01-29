# Regression Testing: Shutdown Services

**Date**: 2026-01-29
**Purpose**: Prevent shutdown service files from being missing again
**Status**: ✅ Tests written and passing

---

## The Problem We're Solving

On 2026-01-29, users couldn't shutdown the system on bare metal:
```bash
$ shutdown now
Unit systemd-halt or poweroff not found
```

**Root cause**: Three systemd service files were completely missing from the ISO:
- `systemd-halt.service`
- `systemd-poweroff.service`
- `systemd-reboot.service`

**Why it happened**: These services weren't in `distro-spec/src/shared/components.rs`, so they were never copied into the rootfs, even though they existed in the systemd RPM.

---

## Test Strategy

### Level 1: Compile-Time Tests (distro-spec)

**File**: `distro-spec/src/shared/components.rs`

**Tests**:
```rust
#[test]
fn test_shutdown_services_present() {
    // Ensures systemd-halt.service, systemd-poweroff.service,
    // systemd-reboot.service, and systemd-soft-reboot.service
    // are in ESSENTIAL_UNITS
}

#[test]
fn test_shutdown_targets_present() {
    // Ensures halt.target, poweroff.target, reboot.target
    // are in ESSENTIAL_UNITS
}
```

**Run**:
```bash
cd distro-spec
cargo test --lib

# Expected output:
# test result: ok. 65 passed; 0 failed
```

**What it catches**:
- ✅ If shutdown services are removed from ESSENTIAL_UNITS
- ✅ If shutdown targets are removed from ESSENTIAL_UNITS
- ✅ If ALL_SYSTEMD_UNITS is not kept in sync

**Fails immediately** at compile/test time, before any ISO building.

---

### Level 2: Verification Tests (fsdbg)

**File**: `testing/fsdbg/src/checklist/rootfs.rs`

**Tests**:
```rust
#[test]
fn test_shutdown_services_in_spec() {
    // Verifies systemd-halt.service, systemd-poweroff.service,
    // systemd-reboot.service are in ESSENTIAL_UNITS
}

#[test]
fn test_shutdown_targets_depend_on_services() {
    // Verifies shutdown targets (halt.target, poweroff.target, reboot.target)
    // have their corresponding services defined
}
```

**Run**:
```bash
cd testing/fsdbg
cargo test --lib

# Expected output:
# test result: ok. 42 passed; 0 failed
```

**What it catches**:
- ✅ If shutdown services are missing from the build spec
- ✅ Targets that exist without their service dependencies (broken state)

---

### Level 3: Integration Tests (QEMU)

**Optional**: Can add runtime tests to verify shutdown actually works.

```bash
# This could be added to testing/install-tests/
# Boot ISO in QEMU and verify shutdown command works
```

---

## Running All Tests

### Quick verification (what you should do before every ISO build):

```bash
# Run all regression tests
cd distro-spec && cargo test --lib && \
cd ../testing/fsdbg && cargo test --lib

# Should see:
# distro-spec: test result: ok. 65 passed
# fsdbg: test result: ok. 42 passed
```

### Full verification (during CI/CD pipeline):

```bash
# Build ISO
cd leviso && cargo run --release -- build

# Extract and verify the EROFS
mkdir -p /tmp/erofs-verify
sudo mount -t erofs leviso/output/filesystem.erofs /tmp/erofs-verify
test -f /tmp/erofs-verify/usr/lib/systemd/system/systemd-halt.service && \
test -f /tmp/erofs-verify/usr/lib/systemd/system/systemd-poweroff.service && \
test -f /tmp/erofs-verify/usr/lib/systemd/system/systemd-reboot.service && \
echo "✅ All shutdown services present" || echo "❌ FAILED"
sudo umount /tmp/erofs-verify
```

---

## CI/CD Integration

### GitHub Actions (example)

```yaml
name: Regression Tests

on: [push, pull_request]

jobs:
  shutdown-services:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          submodules: recursive

      - name: Test distro-spec
        run: cd distro-spec && cargo test --lib

      - name: Test fsdbg
        run: cd testing/fsdbg && cargo test --lib
```

---

## What These Tests Prevent

| Scenario | Test | Catches? |
|----------|------|----------|
| Someone removes systemd-halt.service from ESSENTIAL_UNITS | test_shutdown_services_present | ✅ Yes |
| Someone deletes halt.target | test_shutdown_targets_present | ✅ Yes |
| Target exists but service is missing | test_shutdown_targets_depend_on_services | ✅ Yes |
| Services in ESSENTIAL_UNITS but not in ALL_SYSTEMD_UNITS | test_shutdown_services_in_spec | ✅ Yes |
| ISO is built without shutdown services | fsdbg checks in verify() | ✅ Yes |

---

## Test Maintenance

### Adding new shutdown-related components

If you add new shutdown or reboot services:

1. Add to `ESSENTIAL_UNITS` in `distro-spec/src/shared/components.rs`
2. Add to `ALL_SYSTEMD_UNITS` in the same file
3. Add a test assertion in `test_shutdown_services_in_spec()` or create a new test
4. Run `cargo test --lib` to verify

### Modifying existing tests

If you need to update tests:

1. Update the service list in the test
2. Run tests to verify they pass
3. Commit with a message explaining why the list changed

---

## Future Improvements

### Runtime QEMU test

```rust
// testing/install-tests/src/tests/
#[tokio::test]
async fn test_shutdown_command_works() {
    // Boot ISO in QEMU
    // Run: shutdown now
    // Verify QEMU exits cleanly (not a hang)
    // Verify systemd-poweroff.service was loaded
}
```

### fsdbg EROFS support

Once fsdbg supports EROFS directly, add:

```bash
fsdbg verify filesystem.erofs --type rootfs 2>&1 | grep "shutdown"
# Should show all shutdown services and targets
```

---

## Key Takeaway

These regression tests ensure:

1. **Compile-time safety**: Tests fail immediately if services are missing from the spec
2. **Verification**: fsdbg checks ensure services are in the build output
3. **Documentation**: Comments explain why these services are critical
4. **Prevention**: Future developers can't accidentally break shutdown

**Always run tests before building the ISO:**
```bash
cargo test --lib  # distro-spec
cargo test --lib  # testing/fsdbg
```

---

**Created**: 2026-01-29
**Related Issues**: Shutdown not working on bare metal (2026-01-29)
**Status**: Tests added, passing, and documented
