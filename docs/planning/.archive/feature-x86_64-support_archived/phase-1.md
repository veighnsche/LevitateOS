# Phase 1: Discovery — x86_64 Support

## Feature Summary
LevitateOS currently only runs on AArch64 (ARM64). This feature implements x86_64 architecture support, enabling the kernel to boot and run on modern Intel/AMD processors (testing on QEMU). This provides a second testbed for the kernel's architecture-independent design.

## Success Criteria
- [ ] Kernel boots on QEMU x86_64.
- [ ] Paging is initialized (kernel mapped in higher-half).
- [ ] Interrupts (IDT) and Exceptions are handled.
- [ ] `sys_write` and `sys_exit` system calls work.
- [ ] Userspace "Hello World" runs.

## Current State Analysis
- **Architecture**: `kernel/src/arch` has an `aarch64` module and a `mod.rs` that conditionally exports it.
- **Build**: `xtask` hardcodes `qemu-system-aarch64` and related flags.
- **Userspace**: Userspace binaries are compiled for `aarch64`.

## Operations
- No functional changes to AArch64.
- New `arch::x86_64` module will be strictly additive.
- `xtask` needs to learn about `TARGET` environment variable or argument.

## Constraints
- **Multiboot2**: We will use Multiboot2 for booting (GRUB/limine/QEMU -kernel).
- **no_std**: x86_64 implementation must be `no_std`.
