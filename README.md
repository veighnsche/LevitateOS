# LevitateOS

A Fedora-based Linux distribution with an AI-powered installer.

## Quick Start

```bash
# Build the ISO (takes ~6 min)
./build-iso.sh

# Boot it
./run-vm.sh
```

## Development

Build the installer:
```bash
cargo build --release -p levitate-installer
```

Test in VM:
```bash
./run-vm.sh
# Shared folder auto-mounts at /mnt/share
/mnt/share/target/release/levitate-installer
```

Rebuild on host, re-run in VM. No ISO rebuild needed.

## Structure

```
crates/installer/   # Rust installer (WIP)
kickstarts/         # ISO build recipes
vendor/models/      # SmolLM3-3B LLM
docs/               # Design docs
```

## License

MIT
