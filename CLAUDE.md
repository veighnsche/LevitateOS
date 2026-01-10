# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

### Building

```bash
# Build everything (kernel + userspace + initramfs + Eyra coreutils)
cargo xtask build all

# Build specific components
cargo xtask build kernel      # Kernel only
cargo xtask build userspace   # Userspace + initramfs
cargo xtask build initramfs   # Create initramfs only
cargo xtask build iso         # Bootable Limine ISO (x86_64)

# Build Eyra utilities (uutils coreutils with Eyra - provides std support)
cargo xtask build eyra --arch x86_64
cargo xtask build eyra --arch aarch64
```

### Running

```bash
# Basic run (default: x86_64 with GUI)
cargo xtask run

# Run with debugging
cargo xtask run --gdb          # Start GDB server on port 1234
cargo xtask run --gdb --wait   # Wait for GDB connection before starting

# Display modes
cargo xtask run --term         # Terminal mode (no GUI)
cargo xtask run --vnc          # VNC display for browser verification
cargo xtask run --headless     # No display

# Architecture selection
cargo xtask --arch aarch64 run
cargo xtask --arch x86_64 run  # Default

# Profiles (aarch64 only)
cargo xtask run --profile pixel6  # 8GB RAM, 8 cores, cortex-a76, GICv3

# Special modes
cargo xtask run --iso          # Force ISO boot
cargo xtask run --test         # Run internal OS tests
cargo xtask run --verify-gpu   # Verify GPU display via VNC
```

### Testing

```bash
# Run all tests
cargo xtask test

# Run specific test suites
cargo xtask test unit          # Host-side unit tests
cargo xtask test behavior      # Boot output comparison against golden reference
cargo xtask test regress       # Static analysis and regression checks

# Integration tests
cargo xtask test debug         # Debug tools integration tests
cargo xtask test serial        # Serial input tests
cargo xtask test keyboard      # Keyboard input tests
cargo xtask test shutdown      # Shutdown tests
cargo xtask test screenshot    # Alpine Linux screenshot tests
cargo xtask test levitate      # LevitateOS display tests

# Update golden files when behavior intentionally changes
cargo xtask test behavior --update
cargo xtask test debug --update

# Configure golden file ratings (gold = strict, silver = auto-update)
# Edit xtask.toml to set files as "silver" during active development
# See docs/development/silver-golden-files.md for details
```

### VM Interaction

```bash
# Persistent VM session
cargo xtask vm start           # Start VM session
cargo xtask vm stop            # Stop VM session
cargo xtask vm send "ls"       # Send keystrokes to running VM
cargo xtask vm exec "ls"       # Execute command in fresh VM (slower)
cargo xtask vm screenshot      # Take screenshot of running VM
cargo xtask vm regs            # Dump CPU registers
cargo xtask vm mem 0x1000      # Dump memory at address
```

### Disk Management

```bash
cargo xtask disk create        # Create disk image if missing
cargo xtask disk install       # Install userspace binaries to disk
cargo xtask disk status        # Show disk image status
```

### Utilities

```bash
cargo xtask check              # Run preflight checks
cargo xtask clean              # Clean artifacts and QEMU locks
cargo xtask kill               # Kill running QEMU instances
```

## Architecture Overview

LevitateOS is a dual-architecture (AArch64 and x86_64) operating system kernel written in Rust. It uses a modular workspace structure with clear separation between kernel, HAL, and utilities.

### Workspace Structure

```
crates/
├── kernel/              # Main kernel binary (levitate-kernel)
├── hal/                 # Hardware Abstraction Layer (los_hal)
│   └── src/
│       ├── gic.rs       # GICv2/GICv3 auto-detection
│       ├── mmu.rs       # Page tables & translation
│       ├── console.rs   # Early console
│       ├── timer.rs     # Timer abstraction
│       └── ...
├── utils/               # Core utilities (los_utils)
│   └── src/
│       ├── spinlock.rs  # Spinlock
│       ├── ringbuffer.rs # Ring buffer
│       └── cpio.rs      # CPIO parser
├── term/                # ANSI terminal emulator (los_term)
├── gpu/                 # VirtIO GPU library (los_gpu)
├── pci/                 # PCI bus support (los_pci)
├── virtio-transport/    # VirtIO transport layer
├── drivers/             # Device drivers
│   ├── virtio-blk/
│   ├── virtio-gpu/
│   ├── virtio-input/
│   ├── virtio-net/
│   ├── nvme/
│   ├── xhci/
│   └── simple-gpu/
├── error/               # Error handling (los_error)
├── traits/              # Device traits
│   ├── input-device/
│   ├── network-device/
│   └── storage-device/
└── userspace/           # Userspace programs
    └── eyra/            # Eyra-based utilities (provides std support)
        └── coreutils/   # Git submodule: uutils coreutils

xtask/                   # Development task runner
docs/                    # Documentation
tests/                   # Golden files for behavior tests
```

### Multi-Architecture Support

LevitateOS supports both AArch64 and x86_64 through a layered abstraction:

1. **Kernel Architecture Layer** (`kernel/src/arch/`): Each architecture implements:
   - `SyscallFrame`: Register state during syscalls
   - `SyscallNumber`: Platform-specific syscall mapping
   - `Stat` / `Timespec`: Platform-specific metadata layouts
   - `cpu::wait_for_interrupt()`: Idle loop

2. **HAL Traits** (`los_hal/src/traits.rs`):
   - `InterruptController`: Generic interface for GIC (ARM) or APIC (x86)
   - `MmuInterface`: Generic interface for page tables

3. **Userspace ABI** (`userspace/libsyscall/src/arch/`):
   - AArch64: `svc #0` with x8-x0 registers
   - x86_64: `syscall` with rax, rdi, rsi registers

### Boot Sequence

1. **Assembly Entry** (`_start`): Disable interrupts, enable FP/SIMD, zero BSS, setup early page tables, enable MMU
2. **Heap Init**: Initialize `linked_list_allocator` from linker script bounds
3. **MMU**: Re-initialize with higher-half mappings (2MB blocks)
4. **Drivers**: Exception vectors → UART → GIC (auto-detect v2/v3) → Timer
5. **Memory**: Buddy allocator from memory map
6. **VirtIO**: Scan bus for GPU, Input, Block, Network devices
7. **Filesystem**: Mount filesystems, parse initramfs
8. **Main Loop**: Interactive shell with task scheduling

### Key Architectural Patterns

**Higher-Half Kernel**: Kernel runs in virtual address space `0xFFFF_8000_0000_0000`

**Linux ABI Compatibility**: LevitateOS implements the Linux AArch64 and x86_64 syscall ABIs to support unmodified Rust `std` binaries. Critical details:
- `Stat` struct must be exactly 128 bytes on AArch64
- Auxiliary vector (auxv) required on stack for `std::rt` initialization
- `TPIDR_EL0` (AArch64) / `FS` register (x86_64) must be context-switched for TLS
- `writev()`/`readv()` required for Rust `println!`
- Errno values must match Linux (e.g., `ENOSYS = 38`)

**VFS Layer**: Linux-inspired Virtual File System with:
- Superblock → Inode → Dentry → File hierarchy
- Mount support for multiple filesystems
- tmpfs, FAT32, ext4 (read-only), CPIO initramfs support

**Memory Management**:
- Buddy allocator for physical pages
- Slab allocator for kernel objects
- VMM (Virtual Memory Area) management inspired by Redox `rmm`
- Support for `mmap`, `brk`, demand paging

### Error Handling

All kernel errors use typed enums with numeric codes defined via `define_kernel_error!` macro:

```rust
define_kernel_error! {
    /// My subsystem errors (0x10xx)
    pub enum MyError(0x10) {
        /// Description
        SomethingWrong = 0x01 => "Message",
        /// Nested error
        Other(InnerError) = 0x02 => "Nested error",
    }
}
```

Error code format: `0xSSCC` where `SS` = subsystem, `CC` = code within subsystem.

## Development Guidelines

### Core Rules (from `.agent/rules/kernel-development.md`)

**Rule 4: Silence is Golden**
- Production builds produce NO output on success
- Errors must be loud and immediate
- Use `--features verbose` for behavior testing
- Default log level: `warn` (production), `trace` (verbose builds)

**Rule 5: Memory Safety**
- Minimize `unsafe` blocks
- Every `unsafe` requires `// SAFETY:` comment explaining soundness
- Wrap unsafe in safe abstractions using Newtype pattern
- Use RAII for all resource management

**Rule 6: Robust Error Handling**
- All fallible operations return `Result<T, E>`
- Use `Option<T>` for potentially missing values
- NO `unwrap()`, `expect()`, or `panic!` (enforced by clippy)
- Use `?` operator for error propagation

**Rule 14: Fail Loud, Fail Fast**
- When you must fail, fail noisily and immediately
- Return specific `Err` variants
- Do not attempt partial recovery if internal state is corrupted
- Use `debug_assert!` for internal invariants

**Rule 20: Simplicity > Perfection**
- Implementation simplicity is the highest priority
- Favor clear Rust code over complex perfect solutions
- If handling a rare edge case doubles complexity, return an `Err`

### Testing Philosophy (from `.agent/rules/behavior-testing.md`)

**All Behaviors Must Be Documented**
- Maintain `docs/testing/behavior-inventory.md` with all testable behaviors
- Each behavior gets a unique ID (e.g., `[MMU1]`, `[R2]`)
- IDs must appear in source code, test code, AND inventory

**Traceability Example**:
```rust
/// [R2] Push adds element, [R4] returns false when full
pub fn push(&mut self, byte: u8) -> bool { /* ... */ }

#[test]
/// Tests: [R1] new empty, [R2] push, [R3] FIFO, [R4] full
fn test_ring_buffer_fifo() { /* ... */ }
```

**Critical Rule: ALL TESTS MUST PASS**
- NEVER dismiss a failing test as "pre-existing" without investigation
- Test failures require root cause analysis:
  - Did YOUR changes cause it? → Fix your code
  - Is the change intentional? → Update golden files with `--update`
  - Is the test buggy? → Fix the test
  - Truly unrelated? → PROVE IT with git blame
- The cost of dismissal is high: you will be asked to redo the work

**Test Levels**:
- **Unit Tests**: Host-side tests with `--features std` (mocks for hardware)
- **Behavior Tests**: Boot kernel with `--features verbose`, compare to golden log
- **Regression Tests**: Static analysis for API consistency, constant sync, code patterns

**Golden File Updates**:
When kernel behavior intentionally changes:
1. Run test suite (it will fail with diff)
2. Review diff carefully
3. If correct, update: `cargo xtask test behavior --update`
4. Commit with explanation

### Code Organization

**Module Boundaries**:
- Kernel logic: `crates/kernel/src/`
- Hardware abstraction: `crates/hal/src/`
- Shared utilities: `crates/utils/src/`
- Device drivers: `crates/drivers/*/`

**Feature Flags** (kernel):
- `verbose`: Enable boot messages for testing
- `diskless`: Skip initrd requirement
- `multitask-demo`: Enable demo tasks
- `verbose-syscalls`: Enable syscall logging

**Crate Naming**: All library crates use `los_` prefix (LevitateOS).

### Important Patterns

**Architecture-Specific Code**:
```rust
#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64::...;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86_64::...;
```

**Verbose Logging**:
```rust
#[cfg(feature = "verbose")]
log::debug!("Verbose message");
```

**Safe Hardware Abstraction**:
```rust
// SAFETY: Writing to MMIO address 0x... is safe because the
// device specification guarantees this register is write-only
// and has no side effects on other system state.
unsafe { ptr::write_volatile(addr, value) }
```

**Error Definition**:
See `docs/planning/error-macro/phase-1.md` for subsystem allocation.

## Critical Knowledge Areas

### Linux ABI Compatibility

When modifying syscalls or userspace interaction, consult:
- `docs/specs/LINUX_ABI_GUIDE.md` - Critical gotchas and patterns
- `docs/specs/userspace-abi.md` - Definitive ABI specification

Key considerations:
- Structure padding and alignment (especially `Stat` at 128 bytes)
- Auxiliary vector on process stack (`AT_PAGESZ`, `AT_RANDOM`, `AT_PHDR`, etc.)
- TLS register context switching (`TPIDR_EL0` on AArch64, `FS` on x86_64)
- Errno value alignment with Linux (use constants from `kernel/src/syscall/mod.rs::errno`)
- Vectored I/O for `println!` (`writev`/`readv`)

### Eyra Integration

**Status**: Phase 4 complete - Eyra utilities integrated with `--with-eyra` flag
- Located in `crates/userspace/eyra/coreutils` (git submodule)
- Provides `std` support via Eyra origin runtime
- Build with: `cargo xtask build eyra --arch <arch>`
- Individual utilities replaced by multi-call binary from uutils-coreutils

### Memory Layout

| Region | Physical Address | Virtual Address | Notes |
|--------|------------------|-----------------|-------|
| Device MMIO | `0x0000_0000..0x4000_0000` | Identity mapped | VirtIO, UART, GIC |
| Kernel Start | `0x4008_0000` (AArch64) | `0xFFFF_8000_4008_0000` | Higher-half |
| Kernel Heap | After kernel | Higher-half mapped | Buddy allocator |

### QEMU Profiles

| Profile | RAM | Cores | CPU | GIC/APIC |
|---------|-----|-------|-----|----------|
| Default (aarch64) | 512MB | 1 | cortex-a53 | GICv2 |
| Pixel 6 (aarch64) | 8GB | 8 | cortex-a76 | GICv3 |
| Default (x86_64) | 32GB | 2+ | i3 | APIC |

## Team Logs

Implementation details and decisions are tracked in `.teams/` with TEAM_XXX identifiers. When making significant changes, reference the relevant TEAM number in comments and commit messages.

## References

- **Roadmap**: `docs/ROADMAP.md` - Development phases
- **Architecture**: `docs/ARCHITECTURE.md` - Design principles and workspace structure
- **Behavior Inventory**: `docs/testing/behavior-inventory.md` - All testable behaviors
- **QEMU Profiles**: `docs/QEMU_PROFILES.md` - Hardware emulation configurations
- **Contributing**: `CONTRIBUTING.md` and `CODE_OF_CONDUCT.md`
- **Security**: `SECURITY.md` - Security vulnerability reporting
