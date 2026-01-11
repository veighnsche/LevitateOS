# libsyscall

Userspace syscall library for LevitateOS.

## Overview

`libsyscall` provides the low-level interface between userspace applications and the LevitateOS kernel. It defines the system call ABI, structures shared between the kernel and userspace (e.g., `Stat`, `Dirent`), and safe Rust wrappers for the `svc` instruction.

## Features

- **Standard ABI**: Implements a modular system call interface compatible with the kernel's handler architecture.
- **Safe Wrappers**: Provides high-level functions like `read()`, `write()`, `spawn()`, and `exit()` that handle register clobbering and error conversions.
- **Typed Errors**: Integrates with the kernel's error system to provide meaningful failure information in userspace.

## Usage

Applications depend on this crate to perform any interaction with the hardware or other processes.

```rust
use libsyscall::{write, STDOUT_FILENO};

fn main() {
    let _ = write(STDOUT_FILENO, b"Hello from userspace!\n");
}
```

## Architecture

- `src/arch/`: Architecture-specific syscall invocation (e.g., `aarch64.rs`, `x86_64.rs`).
- `src/lib.rs`: The high-level API exposed to applications.
- `src/sysno.rs`: System call number definitions.
- `src/errno.rs`: Error code definitions.
