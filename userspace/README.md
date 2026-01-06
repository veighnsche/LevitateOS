# LevitateOS Userspace

This directory contains userspace applications for LevitateOS.
It is a standalone Cargo workspace.

## Structure

- **[libsyscall/](libsyscall/README.md)**: Shared library defining the system call ABI and wrappers.
- **[shell/](shell/README.md)**: Interactive shell (lsh) binary.
- **[init/](init/README.md)**: PID 1 process implementation.
- **[repro_crash/](repro_crash/README.md)**: Diagnostic crash reproduction tool.
- **[ulib/](ulib/README.md)**: Userspace standard library effort.

## Building

```bash
cd userspace
cargo build --release
```

Binaries are output to `target/aarch64-unknown-none/release/`.
Note: Build relies on `build.rs` in each crate to set linker arguments.

## Adding to Initramfs

Run the helper script from the project root:

```bash
./scripts/make_initramfs.sh
```

This will copy the built binaries to `initrd_root/` and generate `initramfs.cpio`.
