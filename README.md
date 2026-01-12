# LevitateOS

**An Experimental AI-Written Operating System**

> ⚠️ **This is NOT a production operating system.**
>
> LevitateOS is an experiment: *How far can I get building a general-purpose, POSIX-compatible, musl/BusyBox operating system written entirely by AI agents?*
>
> Yes, the code looks bad — the goal is making it work, not winning beauty contests.  
> No, I didn't care about security — only capability.  
> Yes, a Linux app should run on it. That's the whole point.  
> This is my own kernel, built from scratch.

LevitateOS is a Rust-based OS that aims to **run unmodified Linux binaries**. It targets POSIX compatibility via musl libc and BusyBox, providing a minimal but functional Unix environment.

| Target | Status |
|--------|--------|
| Run unmodified Linux binaries | 🎯 Goal |
| Linux syscall ABI | 🟡 In Progress |
| libc compatibility (via c-gull) | 🔲 Next Milestone |

Supports AArch64 and x86_64, targeting QEMU `virt`, Pixel 6, and Intel NUC.

## 🎯 Supported Targets

- **AArch64**: QEMU `virt`, Pixel 6
- **x86_64**: Intel NUC (7th Gen i3), QEMU `q35` (Experimental)

## ✨ Features

- **Higher-Half Kernel**: Runs in the top-half of the 48-bit virtual address space (`0xFFFF_8000_0000_0000`)
- **Standard AArch64 MMU**: Uses TTBR1 for kernel, TTBR0 for identity/userspace, with 2MB block optimization
- **GICv2/GICv3 Support**: Auto-detected via FDT for broad hardware compatibility
- **VirtIO Drivers**: GPU (framebuffer), Input (keyboard/tablet), Block (storage), Network, PCI transport
- **Filesystem Support**: VFS layer with mount support, tmpfs, FAT32, ext4 (read-only), CPIO initramfs
- **Linux ABI Compatibility**: Targeted syscall interface for Rust `std` support
- **Multitasking**: Preemptive Round-Robin scheduler with context switching
- **Buddy & Slab Allocators**: Robust physical and kernel object memory management
- **Micro-kernel Ready**: Modular workspace design with a clean HAL

## 🏗️ Architecture

```
├── crates/
│   ├── gpu/          # VirtIO GPU Library (los_gpu)
│   ├── term/         # ANSI Terminal Emulator (los_term)
│   ├── hal/          # Hardware Abstraction Layer (los_hal)
│   │   └── src/
│   │       ├── gic.rs        # GicV2/GicV3 auto-detection
│   │       ├── mmu.rs        # Page tables & translation
│   │       └── ...           # Console, Timer, FDT, VirtIO
│   ├── utils/        # Core utilities (los_utils)
│   │   └── src/
│   │       ├── lib.rs        # Spinlock, RingBuffer
│   │       ├── cpio.rs       # CPIO archive parser
│   │       └── hex.rs        # Hex formatting
│   ├── pci/          # PCI bus support (los_pci)
│   ├── virtio/       # VirtIO transport (los_virtio)
│   └── error/        # Error handling (los_error)
│
├── xtask/            # Development task runner
│   └── src/
│       ├── main.rs       # CLI (build, run, test)
│       └── tests/        # Unit, behavior, regression tests
│
├── docs/             # Documentation
│   ├── ROADMAP.md        # Development phases
│   ├── ARCHITECTURE.md   # Design principles
│   └── planning/         # Feature planning docs
│
└── tests/            # Golden files for behavior tests
```

## 🚀 Quick Start

### Prerequisites

- **Rust Nightly** with `rust-src` component
- **QEMU** (`qemu-system-aarch64`)

```bash
rustup default nightly
rustup component add rust-src
```

### Build & Run

```bash
cargo xtask run                # Build and boot in QEMU (512MB, 1 core)
cargo xtask run-pixel6         # Boot with Pixel 6 profile (8GB, 8 cores, GICv3)
```

### Testing

```bash
cargo xtask test               # Run all tests
cargo xtask test unit          # Host-side unit tests (los_hal, los_utils)
cargo xtask test behavior      # Golden log comparison (kernel boot output)
cargo xtask test regress       # Static analysis (API consistency, constant sync)
```

## 📦 Crate Overview

| Crate | Purpose |
|-------|---------|
| **[kernel](kernel/README.md)** | Main kernel binary — boot sequence, device drivers, main loop |
| **[los_gpu](crates/gpu/README.md)** | VirtIO GPU driver and graphics abstraction |
| **[los_term](crates/term/README.md)** | Platform-agnostic ANSI terminal emulator |
| **[los_hal](crates/hal/README.md)** | Hardware abstraction — GIC, MMU, Timer, UART, VirtIO HAL |
| **[los_utils](crates/utils/README.md)** | Core utilities — Spinlock, RingBuffer, CPIO parser, hex formatting |
| **[los_pci](crates/pci/README.md)** | PCI bus enumeration and configuration |
| **[los_virtio](crates/virtio/README.md)** | VirtIO transport layer |
| **[los_error](crates/error/README.md)** | Error handling infrastructure |
| **[xtask](xtask/README.md)** | Development task runner — build, run, test commands |

## 🔧 Boot Sequence

1. **Assembly Entry** (`_start`): Disable interrupts, enable FP/SIMD, zero BSS, save DTB address, setup early page tables, enable MMU
2. **Heap Init**: Initialize `linked_list_allocator` with bounds from linker script
3. **MMU**: Re-initialize with higher-half mappings and 2MB block optimization
4. **Drivers**: Exception vectors → UART console → GIC (auto-detect v2/v3) → Timer
5. **Memory**: Buddy allocator from DTB memory map
6. **VirtIO**: Scan MMIO bus for GPU, Input, Block devices
7. **Filesystem**: Mount FAT32 boot partition, parse initramfs
8. **Main Loop**: Poll input, echo UART, draw cursor

## 🖥️ QEMU Profiles

| Profile | RAM | Cores | CPU | GIC |
|---------|-----|-------|-----|-----|
| Default | 512MB | 1 | cortex-a53 | v2 |
| Pixel 6 | 8GB | 8 | cortex-a76 | v3 |
| Intel NUC | 32GB | 2+ | x86_64 i3 | APIC |

## 📍 Memory Layout

| Region | Physical Address | Virtual Address |
|--------|------------------|-----------------|
| Device MMIO | `0x0000_0000..0x4000_0000` | Identity mapped |
| Kernel Start | `0x4008_0000` | `0xFFFF_8000_4008_0000` |
| Kernel Heap | After kernel | Higher-half mapped |

## 📚 Documentation

- **[Roadmap](docs/ROADMAP.md)**: Development phases (Drivers → MMU → Userspace)
- **[Architecture](docs/ARCHITECTURE.md)**: Workspace structure and design principles
- **[Behavior Inventory](docs/testing/behavior-inventory.md)**: Testable behaviors with IDs
- **[QEMU Profiles](docs/QEMU_PROFILES.md)**: Hardware emulation configurations

## 🧪 Testing Philosophy

LevitateOS follows **Rule 4: Silence is Golden** — production builds are silent, errors are loud.

- **Unit Tests**: Host-side tests with `--features std` (mocks for hardware ops)
- **Behavior Tests**: Boot kernel with `--features verbose`, compare to golden log
- **Regression Tests**: Static analysis for API consistency and constant synchronization

## 🤝 Contributing

Contributions are welcome! Please read our **[Contributing Guide](CONTRIBUTING.md)** and **[Code of Conduct](CODE_OF_CONDUCT.md)** before submitting pull requests.

For security vulnerabilities, please refer to our **[Security Policy](SECURITY.md)**.

More technical guidelines can be found in `.agent/rules/`:
- `kernel-development.md`: Rust kernel development SOP
- `behavior-testing.md`: Testing and traceability SOP

Team logs are tracked in `.teams/` directories.

## 📄 License

LevitateOS is licensed under the **[MIT License](LICENSE)**.
```
