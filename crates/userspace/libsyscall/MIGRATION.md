# libsyscall Migration to Eyra

TEAM_380: Documentation for the libsyscall migration to the eyra directory for std support.

## Overview

The `libsyscall` crate has been migrated from `crates/userspace/libsyscall` to `crates/userspace/eyra/libsyscall` to enable integration tests with std support via the Eyra runtime.

## What Changed

### Directory Structure

**Before**:
```
crates/userspace/libsyscall/
├── Cargo.toml          (no_std only)
├── src/                (syscall wrappers)
└── tests/              (Cargo test format - no std)
```

**After**:
```
crates/userspace/eyra/
├── libsyscall/         (migrated crate)
│   ├── Cargo.toml      (with optional std via Eyra)
│   ├── src/            (unchanged syscall wrappers)
│   └── README.md
└── libsyscall-tests/   (new test binary)
    ├── Cargo.toml
    ├── src/
    │   ├── main.rs
    │   ├── errno_tests.rs
    │   ├── integration_tests.rs
    │   ├── memory_tests.rs
    │   ├── path_validation.rs
    │   └── time_tests.rs
    └── README.md
```

### Cargo.toml Changes

The new `libsyscall/Cargo.toml` includes:

```toml
[dependencies.eyra]
version = "0.22"
features = ["experimental-relocate"]
optional = true

[features]
default = []
std = ["eyra"]
```

This allows libsyscall to be used both:
- As a no_std library (default) for bare-metal userspace programs
- With std support (via `features = ["std"]`) for integration tests

### Test Migration

The original `tests/*.rs` files under libsyscall have been converted from Cargo test format to a standalone test runner binary:

- Each test module exports a `run_tests() -> (usize, usize)` function
- Main binary orchestrates running all tests and reporting results
- Tests use `std::panic::catch_unwind` for isolation
- Colored output with ✓/✗ markers for each test

### Workspace Integration

The eyra workspace (`crates/userspace/eyra/Cargo.toml`) now includes:

```toml
[workspace]
members = [
    "eyra-hello",
    "eyra-test-runner",
    "libsyscall",           # New
    "libsyscall-tests",     # New
]
```

## Why This Migration?

### Problem

The original libsyscall tests couldn't use std features like:
- `String` and `Vec` for dynamic test data
- `std::panic::catch_unwind` for test isolation
- Rich formatting and colored output
- Integration with test harnesses

### Solution

By moving libsyscall into the eyra workspace:
1. Eyra provides std-compatible APIs through pure Rust Linux syscalls
2. Tests can use full std while still targeting LevitateOS (no libc dependency)
3. Static-PIE binaries work on LevitateOS (no dynamic linker needed)
4. Tests can be run as regular binaries on the target system

## Building

### Library Only

```bash
cd crates/userspace/eyra
cargo build -p libsyscall --target aarch64-unknown-linux-gnu
```

### With Tests

```bash
cd crates/userspace/eyra
cargo build -p libsyscall-tests --target aarch64-unknown-linux-gnu --release
```

**Note**: Currently blocked on cross-compilation sysroot setup. The code compiles but linking requires aarch64 system libraries.

## Usage

### In no_std Programs (Original Use Case)

```rust
// No changes needed - libsyscall still defaults to no_std
use libsyscall::{read, write, exit};

fn main() {
    let buf = [0u8; 64];
    let n = read(0, &buf);
    write(1, &buf[..n as usize]);
    exit(0);
}
```

### In Eyra Programs (With std Support)

```toml
[dependencies]
libsyscall = { path = "../libsyscall", features = ["std"] }
eyra = { version = "0.22", features = ["experimental-relocate"] }
```

```rust
use libsyscall::{read, write};
use std::string::String;  // Now available!

fn main() {
    let mut data = String::new();
    // ... use std features ...
}
```

## Testing

Run tests on LevitateOS:

```bash
# Build test binary
cargo xtask build eyra --arch aarch64

# Run in VM
cargo xtask vm exec "/bin/libsyscall-tests"
```

## Next Steps

1. **Resolve Cross-Compilation**: Install aarch64 sysroot or use build-std
2. **Integrate with CI**: Add libsyscall-tests to automated test suite
3. **Behavior Tests**: Add golden file tests for syscall outputs
4. **Old Location**: Decide whether to keep or remove `crates/userspace/libsyscall`

## Benefits

✅ Full std support for integration tests
✅ Better test isolation with panic catching
✅ Rich test output with colored results
✅ No changes to core libsyscall implementation
✅ Backward compatible (no_std by default)
✅ Static-PIE binaries (no dynamic linker dependency)

## Tradeoffs

⚠️ Slightly more complex build setup (Eyra toolchain)
⚠️ Cross-compilation requires additional tooling
⚠️ Two copies of libsyscall (temporary during migration)
