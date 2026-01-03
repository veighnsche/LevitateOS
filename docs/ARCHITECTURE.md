# LevitateOS Architecture

**TEAM_009: Workspace Refactoring**

LevitateOS uses a modular **Cargo Workspace** structure, inspired by **Tock OS** and **Redox**. This ensures clear separation of concerns between core kernel logic, hardware abstraction, and shared utilities.

## Workspace Structure

The project root defines the workspace members in `Cargo.toml`:

```toml
[workspace]
members = [
    "kernel",
    "levitate-hal",
    "levitate-utils",
]
```

### 1. Core Kernel (`levitate-kernel`)
- **Location**: `kernel/`
- **Purpose**: High-level OS logic, task scheduling, memory management, and device coordination.
- **Dependencies**: Depends on `levitate-hal` and `levitate-utils`.
- **Note**: This is the binary crate (`main.rs`) that produces the kernel executable.

### 2. Hardware Abstraction Layer (`levitate-hal`)
- **Location**: `levitate-hal/`
- **Purpose**: Low-level drivers and hardware interfacing.
- **Components**:
  - `console`: UART (PL011) interaction.
  - `gic`: Generic Interrupt Controller management.
  - `timer`: AArch64 Generic Timer driver.
- **Design Rule**: Code here should handle `unsafe` MMIO but expose safe APIs to the kernel.

### 3. Utilities (`levitate-utils`)
- **Location**: `levitate-utils/`
- **Purpose**: Shared, hardware-agnostic primitives.
- **Components**:
  - `Spinlock`: Synchronization primitive required by both Kernel and HAL.
- **Design Rule**: Must be `#![no_std]` and mostly dependencies-free.

## Build System

- **Toolchain**: `aarch64-unknown-none`
- **Runner**: `run.sh`
  - Builds the workspace (`cargo build --release`).
  - Extracts the binary from `target/aarch64-unknown-none/release/levitate-kernel`.
  - Converts it to a raw binary (`objcopy`).
  - Launches QEMU with specific device flags (`-device virtio-gpu`, etc.).

## Gotchas & Notes

- **Strict Alignment**: AArch64 requires strict alignment. We use `strict-align` target feature (or similar) where possible, but `levitate-utils` may generate warnings about it being unstable.
- **QEMU Bus**: VirtIO devices in QEMU (legacy/MMIO) are order-sensitive or specific to the command line arguments. Check `run.sh` vs `virtio.rs` scanning logic if devices aren't found.
- **External Kernels**: Reference implementations are stored in `.external-kernels/` which is excluded from VS Code analysis to improve performance.
