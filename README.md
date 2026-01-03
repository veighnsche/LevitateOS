# LevitateOS

A Rust-based Operating System targeting AArch64 (ARMv8).

## 📚 Documentation
- [Roadmap](docs/ROADMAP.md): Detailed development phases (Drivers, MMU, Userspace).
- [Architecture](docs/ARCHITECTURE.md): Workspace structure (`levitate-kernel`, `levitate-hal`) and design principles.

## 🚀 Build and Run

LevitateOS uses a Cargo Workspace.

### Prerequisites
- Rust Nightly
- `cargo-binutils` (`cargo install cargo-binutils`)
- `rust-src` (`rustup component add rust-src`)
- `qemu-system-aarch64`

### Running
To build and boot in QEMU:
```bash
./run.sh
```

## Structure
- `kernel/`: Core OS logic.
- `levitate-hal/`: Hardware Abstraction Layer.
- `levitate-utils/`: Shared primitives.
