# CLAUDE.md - Rootfs Tests

## ⛔ STOP. READ. THEN ACT.

Every time you think you know where something goes - **stop. Read first.**

Every time you think something is worthless and should be deleted - **stop. Read it first.**

Every time you're about to write code - **stop. Read what already exists first.**

The five minutes you spend reading will save hours of cleanup.

---

## What is rootfs-tests?

User experience tests for LevitateOS rootfs. Uses systemd-nspawn to test the OS as a **daily driver competing with Arch Linux**.

Each test answers: "Can a user do X with this OS?" - file operations, text processing, user administration, package management, networking. If Arch users expect it, we test for it.

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
