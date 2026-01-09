# levitate-kernel

The core operating system kernel for LevitateOS.

## Overview

LevitateOS is an AArch64 and x86_64 higher-half kernel written in Rust. It targets the QEMU `virt` machine, Pixel 6 hardware, and Intel NUC (x86_64).

## Architecture

The kernel is organized into several key subsystems:

- **Boot & Assembly** (`src/arch/`): Early boot code, exception vectors, and MMU initialization.
- **Memory Management** (`src/mm/`): Buddy and Slab allocator integration, heap management, and page table control.
- **Drivers** (`src/drivers/`): High-level driver logic (UART, GIC, Timer, VirtIO via PCI).
- **Process & Scheduling** (`src/task/`, `src/syscall/`): Task management, context switching, and Linux-compatible syscall handling.
- **Filesystem** (`src/fs/`): VFS layer, mount management, tmpfs, initramfs (CPIO), and FAT32/ext4 support.

## Feature Flags

- `verbose`: Enables granular boot logging for diagnostic purposes and automated behavior testing.
- `diskless`: Skip requirements for an external block device during boot.
- `multitask-demo`: Enable pre-defined tasks to demonstrate preemptive multitasking.

## Boot Sequence

1. `_start` (ASM) -> Early MMU & Stack setup.
2. `kernel_main` (Rust) -> Subsystem initialization:
   - Exception handlers
   - Heap allocator
   - Console & Logging
   - Interrupt Controller (GIC)
   - Physical Memory (DTB)
   - VirtIO Bus Scan
   - FS Mount & Initramfs
3. PID 1 (`init`) spawn.

## Development

Build the kernel using the root `xtask` runner:

```bash
cargo xtask build kernel
```

Run in QEMU:

```bash
cargo xtask run
```
