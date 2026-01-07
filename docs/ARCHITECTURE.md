# LevitateOS Architecture

> ⚠️ **CURRENT STATE (2026-01-05):** System boots to an interactive shell.

**TEAM_009: Workspace Refactoring**

LevitateOS uses a modular **Cargo Workspace** structure, inspired by **Tock OS** and **Redox**. This ensures clear separation of concerns between core kernel logic, hardware abstraction, and shared utilities.

## Workspace Structure

The project root defines the workspace members in `Cargo.toml`:

```toml
[workspace]
members = [
    "kernel",
    "crates/hal",
    "crates/utils",
    "crates/term",
    "crates/virtio",
    "crates/pci",
    "crates/gpu",
    "crates/error",
    "xtask",
]
```

### 1. Core Kernel (`levitate-kernel`)
- **Location**: `kernel/`
- **Purpose**: High-level OS logic, task scheduling, memory management, and device coordination.
- **Dependencies**: Depends on `los_hal`, `los_utils`, and other `los_*` crates.
- **Note**: This is the binary crate (`main.rs`) that produces the kernel executable.

### 2. Library Crates (`crates/`)

All library crates use the `los_` prefix:

| Crate | Location | Purpose |
|-------|----------|----------|
| `los_hal` | `crates/hal/` | Hardware abstraction (GIC, MMU, Timer, UART, VirtIO HAL) |
| `los_utils` | `crates/utils/` | Shared primitives (Spinlock, RingBuffer, CPIO) |
| `los_term` | `crates/term/` | ANSI terminal emulator |
| `los_virtio` | `crates/virtio/` | VirtIO transport layer |
| `los_pci` | `crates/pci/` | PCI bus enumeration |
| `los_gpu` | `crates/gpu/` | VirtIO GPU driver |
| `los_error` | `crates/error/` | Error handling macros |

## Multi-Architecture Support

LevitateOS supports multiple hardware architectures using a layered abstraction approach.

### 1. Architecture Abstraction Layer (`kernel/src/arch/`)
Generic kernel logic interacts with hardware through the `crate::arch` module. Each supported architecture (e.g., `aarch64`, `x86_64`) must implement a standard set of types and functions:
- **`SyscallFrame`**: Register state saved during a syscall.
- **`SyscallNumber`**: Platform-specific syscall mapping.
- **`Stat` / `Timespec`**: Platform-specific metadata layouts.
- **`Termios`**: Terminal configuration layout.
- **`cpu::wait_for_interrupt()`**: Idle loop implementation.

### 2. Hardware Abstraction Layer (`los_hal`)
The HAL defines traits in `crates/hal/src/traits.rs` that decouple the kernel from specific interrupt controllers and MMUs:
- **`InterruptController`**: Generic interface for GIC (ARM) or APIC (x86).
- **`MmuInterface`**: Generic interface for page table management.

## Build System

- **Toolchain**: Supports `aarch64-unknown-none` and `x86_64-unknown-none`.
- **xtask**: The primary development tool.
  - Use `--arch <arch>` to specify the target (default: `aarch64`).
  - `cargo xtask build --arch x86_64`
  - `cargo xtask run --arch aarch64`

## Gotchas & Notes

- **Strict Alignment**: AArch64 requires strict alignment. We use `strict-align` target feature (or similar) where possible, but `levitate-utils` may generate warnings about it being unstable.
- **QEMU Bus**: VirtIO devices in QEMU (legacy/MMIO) are order-sensitive or specific to the command line arguments. Check `run.sh` vs `virtio.rs` scanning logic if devices aren't found.
- **External Kernels**: Reference implementations are stored in `.external-kernels/` which is excluded from VS Code analysis to improve performance.

## Error Handling

LevitateOS uses typed error enums with numeric codes for debugging.

### Defining New Error Types

Use the `define_kernel_error!` macro for error types:

```rust
use los_error::define_kernel_error;

define_kernel_error! {
    /// My subsystem errors (0x10xx)
    pub enum MyError(0x10) {
        /// Something went wrong
        SomethingWrong = 0x01 => "Something went wrong",
        /// Nested error example
        Other(InnerError) = 0x02 => "Nested error occurred",
    }
}
```

### Error Code Format

```
0xSSCC where:
  SS = Subsystem (e.g., 0x01 for MMU, 0x03 for Spawn)
  CC = Error code within subsystem (01-FF)
```

### Subsystem Allocation

See `docs/planning/error-macro/phase-1.md` for the current subsystem list.

## Userspace & ABI

LevitateOS is transitioning from a minimal custom syscall ABI to full **Linux AArch64 ABI Compatibility** (Phase 10). This strategy enables the use of the standard Rust library (`std`) and existing UNIX toolchains. 

For implementation details and common pitfalls, see:
- [Linux ABI Compatibility Guide](file:///home/vince/Projects/LevitateOS/docs/specs/LINUX_ABI_GUIDE.md) — Critical knowledge for future teams.
- [Userspace ABI Specification](file:///home/vince/Projects/LevitateOS/docs/specs/userspace-abi.md) — Definitive target spec.
