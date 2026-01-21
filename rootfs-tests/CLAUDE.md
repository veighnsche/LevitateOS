# CLAUDE.md - Rootfs Tests

## What is rootfs-tests?

User experience tests for LevitateOS rootfs. Uses systemd-nspawn to test the OS from a first-hand user perspective.

## Commands

```bash
# Run all tests
cargo run -- run

# Run with specific tarball
cargo run -- run --tarball ../leviso/output/levitateos-base.tar.xz

# Run against existing rootfs (faster, no extraction)
cargo run -- run --rootfs /var/lib/machines/levitate-test

# Run only specific category
cargo run -- run --category auth

# Keep rootfs after tests (for debugging)
cargo run -- run --keep

# List all tests
cargo run -- list
```

## Test Categories

| Category | Tests |
|----------|-------|
| binaries | bash, coreutils, grep, sed, tar, mount, curl, recipe... |
| auth | sudo, su, visudo, passwd/shadow, PAM |
| filesystem | FHS dirs, symlinks, /etc configs, os-release |
| systemd | systemd, systemctl, journalctl, units, getty |

## Adding Tests

1. Add test struct to appropriate file in `src/tests/`
2. Implement the `Test` trait
3. Add to the `*_tests()` function

Example:
```rust
struct MyTest;

impl Test for MyTest {
    fn name(&self) -> &str { "my-test" }
    fn category(&self) -> &str { "binaries" }

    fn run(&self, container: &Container) -> Result<TestResult> {
        Ok(run_test(self.name(), self.category(), || {
            let output = container.exec_ok("my-command --version")?;
            Ok(output)
        }))
    }
}
```

## Architecture

```
rootfs-tests/
├── src/
│   ├── main.rs           # CLI and test runner
│   ├── container.rs      # systemd-nspawn wrapper
│   └── tests/
│       ├── mod.rs        # Test trait and collection
│       ├── auth.rs       # sudo, su, PAM tests
│       ├── binaries.rs   # Binary existence tests
│       ├── filesystem.rs # FHS and config tests
│       └── systemd.rs    # Systemd service tests
```

## Requirements

- systemd-nspawn (usually in systemd package)
- sudo access (for container operations)
- LevitateOS tarball or extracted rootfs
