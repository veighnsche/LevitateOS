# LevitateOS

**An AI-native Linux distribution**

Build intelligent Linux systems with integrated LLM capabilities and type-safe Rust tooling.

---

## What is LevitateOS?

LevitateOS is a Linux distribution with built-in AI capabilities:

- **Linux kernel** (6.19-rc5)
- **FunctionGemma** (natural language to shell command translation)
- **Full Python + PyTorch** runtime
- **systemd** (init system)

Type natural language commands and LevitateOS translates them to shell commands.

---

## Quick Start

```bash
# Build everything
cargo run -- build all

# Boot in QEMU with serial console
cargo run -- run --term
```

You'll see Linux boot with systemd starting services, then get a shell prompt:

```
Linux version 6.19.0-rc5-levitate ...
systemd[1]: Started...
levitate#
```

Try natural language:
```bash
? list all files in the current directory
# Translates to: ls -la
```

---

## Commands

```bash
# Building
cargo run -- build all           # Build everything
cargo run -- build linux         # Linux kernel only

# Running
cargo run -- run --term          # Serial console
cargo run -- run                 # GUI mode
cargo run -- run --gdb           # With GDB server

# Testing
cargo run -- test                # Run all tests
cargo run -- test behavior       # Boot output test

# Utilities
cargo run -- clean               # Clean build artifacts
cargo run -- check               # Preflight checks
```

---

## Project Structure

```
levitate/
├── src/                    # Build system source (Rust)
│   ├── builder/            # Linux/systemd/Python builders
│   ├── qemu/               # QEMU runner
│   └── tests/              # Test modules
├── linux/                  # Kernel submodule
├── tools/                  # LLM runner and utilities
├── toolchain/              # Build outputs (gitignored)
├── tests/                  # Golden files
└── docs/                   # Documentation
```

---

## Requirements

- **Rust** (stable)
- **QEMU** (`qemu-system-x86_64`)

```bash
# Fedora
sudo dnf install qemu

# Ubuntu/Debian
sudo apt install qemu-system-x86
```

---

## Architecture Support

| Arch | Status |
|------|--------|
| x86_64 | Primary |
| aarch64 | Experimental |

---

## Development

This project was developed with AI assistance. Each development session is logged in `.teams/TEAM_XXX_*.md` files.

**480+ team sessions** have contributed to this codebase.

---

## License

MIT
