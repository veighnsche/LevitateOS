# LevitateOS Userspace

This directory contains userspace applications for LevitateOS.
It is a standalone Cargo workspace.

## Structure

- **libsyscall/**: Shared library defining the system call ABI and wrappers.
- **shell/**: Interactive shell (lsh) binary.

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
