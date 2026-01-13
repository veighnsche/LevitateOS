# Phase 3: Restructure

## Objective

Reorganize the codebase so the structure reflects what the project actually is: a Linux distribution builder, not a "task runner."

---

## The Problem

Currently:
- `xtask/` suggests this is auxiliary build tooling
- The initramfs builder (the crown jewel) is buried at `xtask/src/build/initramfs/`
- Config files are scattered (`initramfs/`, `xtask/data/`, etc.)

After:
- `src/` is the main binary
- `src/builder/` contains the core product at top level
- `config/` has all configuration in one place

---

## Tasks

### 3.1 Move xtask Source to Root

After Phase 2, xtask should contain only active code (~25 files). Move it to root:

```bash
# Move source files
mv xtask/src/* src/
mkdir -p src

# We'll handle Cargo.toml separately
```

### 3.2 Rename build/ to builder/

```bash
mv src/build src/builder
```

This is semantic: "builder" describes what it IS, not what it DOES.

### 3.3 Verify Module Structure After Move

After the move, `src/` should contain:

```
src/
├── main.rs              # Entry point, CLI
├── builder/             # Core distro builder (was build/)
│   ├── mod.rs
│   ├── linux.rs         # Linux kernel builder
│   ├── busybox.rs       # BusyBox builder
│   ├── openrc.rs        # OpenRC builder
│   ├── initramfs/       # Initramfs builder (5 files)
│   │   ├── mod.rs
│   │   ├── builder.rs
│   │   ├── cpio.rs
│   │   ├── manifest.rs
│   │   └── tui.rs
│   ├── iso.rs           # ISO creator
│   ├── orchestration.rs # Build coordination
│   └── commands.rs      # Build CLI enum
├── qemu/                # QEMU command builder (3 files)
│   ├── mod.rs
│   ├── builder.rs
│   └── profile.rs
├── run.rs               # Run commands
├── vm/                  # VM interaction (4 files)
│   ├── mod.rs
│   ├── session.rs
│   ├── exec.rs
│   └── debug.rs
├── support/             # Utilities (4 files)
│   ├── mod.rs
│   ├── preflight.rs
│   ├── clean.rs
│   └── qmp.rs
├── disk/                # Disk management (2 files)
│   ├── mod.rs
│   └── image.rs
├── tests/               # Test modules (~14 files, review needed)
│   ├── mod.rs
│   └── ...
├── config.rs            # Config loading
└── calc.rs              # Calculator utility
```

**Target: ~25 active files**

### 3.4 Consolidate Configuration

```bash
# Create config directory
mkdir -p config

# Move initramfs config
mv initramfs/initramfs.toml config/
mv initramfs/files config/
mv initramfs/scripts config/  # If useful, otherwise delete

# Remove empty initramfs directory
rmdir initramfs/
```

### 3.5 Update Cargo.toml

Replace root `Cargo.toml` with merged version:

```toml
[package]
name = "levitate"
version = "2.0.0"
edition = "2021"
description = "Rust-native Linux distribution builder"
license = "MIT"

[[bin]]
name = "levitate"
path = "src/main.rs"

[dependencies]
# Copy dependencies from xtask/Cargo.toml
anyhow = "1.0"
clap = { version = "4", features = ["derive"] }
num_cpus = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
ratatui = { version = "0.26", optional = true }
# ... etc

[features]
default = []
tui = ["ratatui"]

[lints.rust]
unsafe_op_in_unsafe_fn = "allow"

[lints.clippy]
# Keep existing clippy configuration
```

### 3.6 Update All Import Paths

Search and replace in all `.rs` files:

| Old | New |
|-----|-----|
| `crate::build::` | `crate::builder::` |
| `use build::` | `use builder::` |
| `mod build;` | `mod builder;` |

### 3.7 Update Config Path References

In `src/builder/initramfs/manifest.rs` (or wherever configs are loaded):

```rust
// Before
let manifest_path = "initramfs/initramfs.toml";

// After
let manifest_path = "config/initramfs.toml";
```

Similarly for any other config file references.

### 3.8 Remove Old xtask Directory

```bash
rm -rf xtask/
```

---

## Directory Structure After Phase 3

```
levitate/
├── src/
│   ├── main.rs
│   ├── builder/
│   │   ├── mod.rs
│   │   ├── linux.rs          # KEEP - Linux kernel builder
│   │   ├── busybox.rs        # KEEP - BusyBox builder
│   │   ├── openrc.rs         # KEEP - OpenRC builder
│   │   ├── initramfs/        # KEEP - Core product
│   │   │   ├── mod.rs
│   │   │   ├── builder.rs
│   │   │   ├── cpio.rs
│   │   │   ├── manifest.rs
│   │   │   └── tui.rs
│   │   ├── iso.rs            # KEEP - ISO creator
│   │   ├── orchestration.rs  # REWRITTEN in Phase 2
│   │   └── commands.rs       # CLI enum
│   ├── qemu/                 # KEEP
│   ├── run.rs                # KEEP
│   ├── vm/                   # KEEP
│   ├── support/              # KEEP
│   ├── disk/                 # KEEP
│   ├── tests/                # REVIEWED
│   ├── config.rs             # KEEP
│   └── calc.rs               # KEEP
├── config/
│   ├── initramfs.toml
│   ├── files/
│   │   └── etc/
│   └── scripts/
├── linux/                    # Kernel submodule
├── toolchain/                # Build outputs (gitignored)
├── docs/
├── tests/                    # Golden files
├── .teams/
├── Cargo.toml
└── README.md
```

---

## Test Module Decisions

The `tests/` module needs review. Based on analysis:

### KEEP (Generic/Useful for Linux+OpenRC)
- `common.rs` - Shared utilities
- `screenshot.rs` - Screenshot capture
- `screenshot_alpine.rs` - Alpine screenshots
- `screenshot_levitate.rs` - Levitate screenshots
- `unit.rs` - Host-side tests (probably still valid)

### LIKELY STALE (Custom Kernel Specific)
- `behavior.rs` - Tests custom kernel boot (golden files need rewrite)
- `regression.rs` - Tests kernel internals (GIC, MMU, VirtIO)
- `coreutils.rs` - Tests coreutils (removed, BusyBox now)

### NEEDS REVIEW
- `serial_input.rs` - May work with Linux
- `keyboard_input.rs` - May work with Linux
- `shutdown.rs` - May work with Linux
- `backspace.rs` - May work with Linux
- `debug_tools.rs` - Needs review

**Decision for Phase 3**: Keep all test files but mark stale ones with a comment. Full test review in Phase 4.

---

## Verification

```bash
# Build should work
cargo build

# Run should work
cargo run -- run --term

# Or with release
cargo run --release -- run --term

# Verify binary name
./target/release/levitate --help
# Should show "levitate" not "xtask"
```

---

## Checklist

- [ ] Moved `xtask/src/*` to `src/`
- [ ] Renamed `src/build/` to `src/builder/`
- [ ] Created `config/` directory
- [ ] Moved `initramfs/` contents to `config/`
- [ ] Updated root `Cargo.toml`
- [ ] Updated all import paths (`build::` → `builder::`)
- [ ] Updated config file path references
- [ ] Removed old `xtask/` directory
- [ ] `cargo build` succeeds
- [ ] `cargo run -- run --term` boots to shell
- [ ] Binary is named `levitate`
- [ ] Ready for Phase 4
