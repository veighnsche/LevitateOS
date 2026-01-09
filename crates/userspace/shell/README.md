# levitate-shell

The primary interactive shell for LevitateOS userspace.

## Purpose

`lsh` (Levitate Shell) provides a command-line interface for interacting with the kernel and filesystems.

## Features

- **Built-in Commands**: `ls`, `cat`, `echo`, `help`.
- **ANSI Support**: Integrated with `levitate-terminal` for consistent text rendering.
- **Process Spawning**: Capable of launching other ELF binaries from the initramfs.

## Architecture

```
userspace/shell/src/
├── main.rs         # Shell entry point and main loop
└── commands/       # Implementation of shell commands
```

## Usage

Built and packaged into the boot-time initramfs.

```bash
# Build
cargo build -p shell --target aarch64-unknown-none
```
