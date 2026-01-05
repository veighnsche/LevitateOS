# hello-userspace

A simple "Hello, World!" example for LevitateOS userspace.

## Purpose

Demonstrates the basic userspace environment in LevitateOS, including:
- **Linker Script**: Usage of `linker.ld` for task-local memory layout.
- **Syscalls**: Early hooks for basic output (mirrored via kernel console).
- **ELF Loading**: Verified by the kernel's initramfs loader.

## Usage

Built as part of the `userspace` project and included in the `initramfs`.

```bash
cargo build -p hello --target aarch64-unknown-none
```
