## ✨ Features
- **Higher-Half Kernel**: Runs in the top-half of the 48-bit virtual address space (`0xFFFF800000000000`).
- **Standard AArch64 MMU**: Uses TTBR1 for the kernel and TTBR0 for identity/userspace.
- **Micro-kernel Ready**: Modular workspace design with a clean HAL.

## 📚 Documentation
- [Roadmap](docs/ROADMAP.md): Detailed development phases (Drivers, MMU, Userspace).
- [Architecture](docs/ARCHITECTURE.md): Workspace structure and design principles.
- [Higher-Half Planning](docs/planning/higher-half-kernel/): Design notes for the MMU transition.

## 🚀 Build and Run

LevitateOS uses a Cargo Workspace.

### Prerequisites
- Rust Nightly
- `cargo-binutils` (`cargo install cargo-binutils`)
- `rust-src` (`rustup component add rust-src`)
- `qemu-system-aarch64`
- `aarch64-linux-gnu` toolchain (for `objcopy` and `ld` if using raw assembly)

### Running
To build and boot in QEMU:
```bash
cargo xtask run
```

### Testing
```bash
cargo xtask test           # Run all tests (behavior + regression)
cargo xtask test behavior  # Run behavior test only (golden log comparison)
cargo xtask test regress   # Run regression tests only
```

### Other Commands
```bash
cargo xtask build          # Build kernel only
cargo xtask --help         # Show all commands
```

## Structure
- `kernel/`: Core OS logic and Higher-Half boot.
- `levitate-hal/`: Hardware Abstraction Layer (MMU, GIC, UART).
- `levitate-utils/`: Shared primitives (Locking).
