# xtask

Development task runner for LevitateOS — automates building, testing, and running the kernel.

## Purpose

This crate implements the **cargo-xtask pattern** to provide project-specific commands that are more complex than what `cargo` offers natively. It handles:

1. Cross-compilation for AArch64
2. Running QEMU with proper device configuration
3. Multi-level testing (unit, behavior, regression)

## Architecture

```
xtask/src/
├── main.rs         # CLI entry point, build/run commands
└── tests/
    ├── mod.rs      # Test module exports
    ├── unit.rs     # Host-side cargo test runner
    ├── behavior.rs # Golden file boot verification
    └── regression.rs # Static analysis tests
```

## Commands

### Build

```bash
cargo xtask build all
```

Builds the kernel in release mode for `aarch64-unknown-none` target.

### Run

```bash
cargo xtask run default
```

Builds and runs in QEMU with default profile (512MB RAM, 1 core, cortex-a53).

### Run with Pixel 6 Profile

```bash
cargo xtask run pixel6
```

Builds and runs with Pixel 6 hardware approximation:
- 8GB RAM
- 8 cores
- cortex-a76 CPU
- GICv3 interrupt controller

### Test

```bash
cargo xtask test [suite]
```

| Suite | Description |
|-------|-------------|
| `all` | Run all test suites (default) |
| `unit` | Run `cargo test` on `levitate-hal` and `levitate-utils` |
| `behavior` | Boot kernel, compare output to golden file |
| `regress` | Static analysis for known bug patterns |

## Test Suites

### Unit Tests (`tests/unit.rs`)

Runs host-side Rust unit tests on crates with the `std` feature:

```bash
cargo test -p levitate-hal --features std --target x86_64-unknown-linux-gnu
cargo test -p levitate-utils --features std --target x86_64-unknown-linux-gnu
```

### Behavior Tests (`tests/behavior.rs`)

Verifies kernel boot output matches a golden reference file:

1. Builds kernel with `--features verbose` (enables boot messages)
2. Runs QEMU headless with 5-second timeout
3. Captures serial output to `tests/actual_boot.txt`
4. Compares against `tests/golden_boot.txt`

This enforces **Rule 4: Silence is Golden** — production builds are silent, but behavior tests use verbose mode.

### Regression Tests (`tests/regression.rs`)

Static analysis that catches bugs unit tests can't:

| Test | Description |
|------|-------------|
| **API Consistency** | `enable_mmu` stub signature matches real function |
| **Constant Sync** | `KERNEL_PHYS_END` matches `linker.ld` `__heap_end` |
| **Code Patterns** | `input.rs` uses `dimensions()` not hardcoded values |

## QEMU Profiles

```rust
pub enum QemuProfile {
    Default,  // 512MB, 1 core, cortex-a53, GICv2
    Pixel6,   // 8GB, 8 cores, cortex-a76, GICv3
    GicV3,    // 512MB, 1 core, cortex-a53, GICv3
}
```

Each profile configures:
- Machine type (`virt` or `virt,gic-version=3`)
- CPU model
- Memory size
- SMP topology

## QEMU Device Configuration

All profiles include:
- `virtio-gpu-device` — Framebuffer
- `virtio-keyboard-device` — Keyboard input
- `virtio-tablet-device` — Absolute pointer (mouse)
- `virtio-net-device` — Network (user mode)
- `virtio-blk-device` — Block storage (`tinyos_disk.img`)

## Building & Running

```bash
# From project root
cargo xtask build all
cargo xtask run default
cargo xtask test

# From xtask directory (automatically detects project root)
cargo run -- build
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `anyhow` | Error handling with context |
| `clap` | CLI argument parsing (derive mode) |

## Integration with CI

```bash
# Full test suite (suitable for CI)
cargo xtask test all
```

Exit codes:
- `0` — All tests passed
- Non-zero — Build or test failure

## Golden File Updates

When kernel boot behavior intentionally changes:

1. Run `cargo xtask test behavior` — it will fail with diff
2. Review the diff carefully
3. If changes are correct, update `tests/golden_boot.txt`:
   ```bash
   cp tests/actual_boot.txt tests/golden_boot.txt
   ```
4. Commit with explanation of behavior change
