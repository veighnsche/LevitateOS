# Phase 4: Polish

## Objective

Update documentation, clean up code style, review tests, and ensure everything is consistent.

---

## Tasks

### 4.1 Update CLI Help Text

In `src/main.rs`, update the command descriptions:

```rust
#[command(name = "levitate", about = "Rust-native Linux distribution builder")]
struct Cli {
    // ...
}
```

Update any references to "xtask" in help text or comments.

### 4.2 Review and Clean Test Modules

Based on xtask analysis, the test modules fall into three categories:

#### KEEP (Generic/Useful)
| File | LOC | Action |
|------|-----|--------|
| `common.rs` | ~50 | Keep as-is |
| `screenshot.rs` | ~100 | Keep as-is |
| `screenshot_alpine.rs` | ~100 | Keep as-is |
| `screenshot_levitate.rs` | ~100 | Keep as-is |
| `unit.rs` | ~100 | Keep, may need minor updates |

#### DELETE (Custom Kernel Specific)
| File | LOC | Reason |
|------|-----|--------|
| `behavior.rs` | 385 | Tests custom kernel boot sequence, PID tracking, shell spawn |
| `regression.rs` | 1000+ | Tests GIC, MMU, GPU, VirtIO - all custom kernel internals |
| `coreutils.rs` | ~200 | Tests coreutils which was replaced by BusyBox |

**Alternative**: Instead of deleting `behavior.rs`, rewrite it for Linux+OpenRC:
- Remove references to PID tracking, shell spawn, GPU flush counts
- Create new golden file for Linux+OpenRC boot: `tests/golden_boot_linux_openrc.txt`
- Test for OpenRC service startup messages

#### REVIEW (May Still Work)
| File | LOC | Notes |
|------|-----|-------|
| `serial_input.rs` | ~100 | Test serial I/O - should work with Linux |
| `keyboard_input.rs` | ~100 | Test keyboard - should work with Linux |
| `shutdown.rs` | ~100 | Test shutdown - may need updates for OpenRC |
| `backspace.rs` | ~50 | Backspace handling - should work |
| `debug_tools.rs` | ~200 | VM debug tools - probably still valid |

### 4.3 Create New Linux+OpenRC Golden Files

Create `tests/golden_boot_linux_openrc.txt`:

```
Linux version 6.19.0-rc5-levitate (...)
...
OpenRC 0.54 is starting up Linux 6.19.0-rc5-levitate
 * Mounting /proc ... [ ok ]
 * Mounting /run ... [ ok ]
 * Mounting /sys ... [ ok ]
 * Mounting /dev ... [ ok ]
 * Setting hostname ... [ ok ]
...
levitate#
```

### 4.4 Update CLAUDE.md

Major sections to update:

**Build Commands:**
```markdown
# Before
cargo xtask build all
cargo xtask run --term

# After
cargo run -- build all
cargo run -- run --term
# Or: levitate build all
```

**Architecture Overview:**
- Remove all kernel development sections
- Update directory structure diagram
- Remove references to `crates/`

**Development Guidelines:**
- Remove kernel-specific rules
- Update testing section for new structure

### 4.5 Update README.md

Complete rewrite:

```markdown
# LevitateOS

A Rust-native Linux distribution builder. Build minimal Linux systems with
type-safe, fast tooling.

## Quick Start

```bash
# Build everything
cargo run -- build all

# Boot in QEMU
cargo run -- run --term
```

## What is LevitateOS?

LevitateOS is a build system that creates minimal Linux distributions from:
- **Linux kernel** (6.19-rc5)
- **musl libc** (static linking)
- **BusyBox** (shell + utilities)
- **OpenRC** (init system)

Think of it as a Rust alternative to Alpine Linux's shell-script toolchain.

## Commands

```bash
cargo run -- build all       # Build everything
cargo run -- build linux     # Build kernel only
cargo run -- build busybox   # Build BusyBox only
cargo run -- build openrc    # Build OpenRC only
cargo run -- build initramfs # Build initramfs only
cargo run -- run             # Boot with GUI
cargo run -- run --term      # Boot with serial console
cargo run -- run --gdb       # Boot with GDB server
cargo run -- clean           # Clean build artifacts
```

## Project Structure

```
levitate/
├── src/           # Main binary
│   ├── builder/   # Core build system
│   └── qemu/      # QEMU runner
├── config/        # Build configurations
├── linux/         # Kernel submodule
└── toolchain/     # Build outputs
```

## Requirements

- Rust toolchain (stable)
- QEMU
- musl-gcc (for static linking)
- meson + ninja (for OpenRC)

## License

MIT
```

### 4.6 Update .gitignore

```gitignore
# Build outputs
/target/
/toolchain/

# Downloaded sources (gitignored, auto-downloaded)
/toolchain/busybox/
/toolchain/openrc/
/toolchain/busybox-out/
/toolchain/openrc-out/

# Final artifacts
*.cpio
*.iso

# IDE
.idea/
.vscode/
*.swp

# OS
.DS_Store
```

### 4.7 Clean Up Dead Imports

```bash
cargo clippy -- -W unused_imports
cargo fmt
```

Fix any warnings.

### 4.8 Update Module Documentation

Each module should have a doc comment explaining what it does:

```rust
//! # Builder Module
//!
//! Core build system for LevitateOS. Creates bootable Linux distributions
//! from source components.
//!
//! ## Components
//!
//! - `linux` - Linux kernel builder
//! - `busybox` - BusyBox builder (static musl)
//! - `openrc` - OpenRC init system builder
//! - `initramfs` - CPIO archive builder with TUI
//! - `iso` - Bootable ISO creator
```

### 4.9 Remove Old TEAM Comments

Search for and evaluate TEAM comments. Remove ones that reference:
- Custom kernel development
- Removed features (Eyra, coreutils, dash)
- Obsolete decisions

Keep ones that document:
- Current architecture decisions
- Gotchas that still apply
- Historical context that's still relevant

---

## Files to Update

| File | Changes |
|------|---------|
| `src/main.rs` | CLI help text, remove xtask references |
| `CLAUDE.md` | Major rewrite for new structure |
| `README.md` | Complete rewrite |
| `.gitignore` | Update for new paths |
| `src/builder/mod.rs` | Module documentation |
| `src/tests/mod.rs` | Remove stale test modules |
| `src/tests/behavior.rs` | Rewrite for Linux+OpenRC or delete |
| `src/tests/regression.rs` | Delete (custom kernel specific) |
| `src/tests/coreutils.rs` | Delete (coreutils removed) |

---

## Verification

```bash
# No warnings
cargo clippy -- -D warnings

# Formatted
cargo fmt --check

# Docs build
cargo doc --no-deps
```

---

## Checklist

- [ ] Updated CLI help text
- [ ] Reviewed test modules:
  - [ ] Deleted or rewrote `behavior.rs`
  - [ ] Deleted `regression.rs`
  - [ ] Deleted `coreutils.rs`
  - [ ] Reviewed remaining test files
- [ ] Created Linux+OpenRC golden file
- [ ] Updated CLAUDE.md
- [ ] Updated README.md
- [ ] Updated .gitignore
- [ ] Ran `cargo clippy` - no warnings
- [ ] Ran `cargo fmt`
- [ ] Reviewed and cleaned TEAM comments
- [ ] `cargo doc` builds without errors
- [ ] Ready for Phase 5
