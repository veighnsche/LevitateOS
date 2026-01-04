# LevitateOS

An AArch64 operating system kernel written in Rust, targeting the QEMU `virt` machine and Pixel 6 hardware.

## ✨ Features

- **Higher-Half Kernel**: Runs in the top-half of the 48-bit virtual address space (`0xFFFF_8000_0000_0000`)
- **Standard AArch64 MMU**: Uses TTBR1 for kernel, TTBR0 for identity/userspace, with 2MB block optimization
- **GICv2/GICv3 Support**: Auto-detected via FDT for broad hardware compatibility
- **VirtIO Drivers**: GPU (framebuffer), Input (keyboard/tablet), Block (storage), Network
- **Filesystem Support**: FAT32 boot partition, ext4 root (read-only), CPIO initramfs
- **Buddy Allocator**: Physical frame allocation with coalescing (4KB–8GB blocks)
- **Micro-kernel Ready**: Modular workspace design with a clean HAL

## 🏗️ Architecture

```
LevitateOS/
├── kernel/           # Main kernel binary
│   └── src/
│       ├── main.rs       # Entry point, boot sequence, kmain()
│       ├── exceptions.rs # Exception vectors, IRQ handling
│       ├── virtio.rs     # VirtIO MMIO transport
│       ├── gpu.rs        # VirtIO GPU (embedded-graphics)
│       ├── input.rs      # VirtIO input devices
│       ├── block.rs      # VirtIO block device
│       ├── fs/           # Filesystem layer (FAT32, ext4, initramfs)
│       └── memory/       # Frame allocator integration
│
├── levitate-hal/     # Hardware Abstraction Layer
│   └── src/
│       ├── gic.rs        # GICv2/GICv3 interrupt controller
│       ├── mmu.rs        # Page tables, address translation
│       ├── timer.rs      # AArch64 generic timer
│       ├── console.rs    # UART console (print!/println!)
│       ├── uart_pl011.rs # PL011 UART driver
│       ├── fdt.rs        # Device Tree parsing
│       ├── interrupts.rs # CPU interrupt control
│       └── allocator/    # Buddy allocator, Page descriptors
│
├── levitate-utils/   # Core utilities (no_std)
│   └── src/
│       ├── lib.rs        # Spinlock, RingBuffer
│       ├── cpio.rs       # CPIO archive parser
│       └── hex.rs        # Hex formatting
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
cargo xtask test unit          # Host-side unit tests (levitate-hal, levitate-utils)
cargo xtask test behavior      # Golden log comparison (kernel boot output)
cargo xtask test regress       # Static analysis (API consistency, constant sync)
```

## 📦 Crate Overview

| Crate | Purpose |
|-------|---------|
| **[kernel](kernel/README.md)** | Main kernel binary — boot sequence, device drivers, main loop |
| **[levitate-hal](levitate-hal/README.md)** | Hardware abstraction — GIC, MMU, Timer, UART, Buddy allocator |
| **[levitate-utils](levitate-utils/README.md)** | Core utilities — Spinlock, RingBuffer, CPIO parser, hex formatting |
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

See `.agent/rules/` for development guidelines:
- `kernel-development.md`: Rust kernel development SOP
- `behavior-testing.md`: Testing and traceability SOP

Team logs are tracked in `.teams/` directories.
