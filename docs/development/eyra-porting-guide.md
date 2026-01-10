# TEAM_394: Eyra Porting Guide

This guide documents how to port applications to run on LevitateOS using Eyra std support.

## Overview

Eyra provides Rust's standard library (`std`) via direct Linux syscalls, without relying on libc. This allows Rust applications to run on LevitateOS with minimal modification.

## Prerequisites

Before porting an application, ensure the kernel supports required syscalls. Common requirements:

| Feature | Required Syscalls |
|---------|-------------------|
| Basic I/O | read, write, close, openat |
| Threading | clone, futex, set_tid_address |
| Async (tokio) | epoll_create1, epoll_ctl, epoll_wait, eventfd2 |
| Job Control | setpgid, getpgid, setsid, tcsetpgrp, tcgetpgrp |
| Time | clock_gettime, nanosleep |
| Memory | mmap, munmap, mprotect, madvise |

## Creating an Eyra Package

### 1. Directory Structure

```
crates/userspace/eyra/
└── your-app/
    ├── Cargo.toml
    └── src/
        └── main.rs
```

### 2. Cargo.toml Template

```toml
[package]
name = "your-app"
version = "0.1.0"
edition = "2021"

[dependencies]
# REQUIRED: Eyra replaces std with pure-Rust syscall implementation
eyra = { version = "0.22", features = ["experimental-relocate"] }

# Your app's dependencies here
```

### 3. main.rs Template

```rust
// REQUIRED: Import eyra for std replacement
extern crate eyra;

fn main() {
    println!("Hello from LevitateOS!");
}
```

### 4. Add to Workspace

Edit `crates/userspace/eyra/Cargo.toml`:

```toml
[workspace]
members = [
    "your-app",  # Add your package
    # ... other members
]
```

## Building

**IMPORTANT:** Always specify the target explicitly:

```bash
cd crates/userspace/eyra
cargo build -p your-app --release --target x86_64-unknown-linux-gnu
```

The workspace `.cargo/config.toml` automatically adds:
- `-nostartfiles` (Eyra provides its own `_start`)
- `+crt-static` (static linking)
- `relocation-model=pic` (position-independent code)

## Verifying the Binary

```bash
file target/x86_64-unknown-linux-gnu/release/your-app
```

Expected output:
```
ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), static-pie linked, stripped
```

Key indicators:
- **static-pie linked**: No dynamic linker needed
- **stripped**: Symbols removed for smaller size

## Common Issues

### 1. Wrong Target Architecture

**Symptom:** `can't find crate for 'core'` with `aarch64-unknown-none`

**Fix:** Always specify `--target x86_64-unknown-linux-gnu`

### 2. Duplicate `_start` Symbol

**Symptom:** Linker error about multiple definitions of `_start`

**Fix:** Ensure `.cargo/config.toml` includes `-nostartfiles`

### 3. Missing Syscalls

**Symptom:** Runtime error or `ENOSYS` (-38) return value

**Fix:** Implement the missing syscall in the kernel:
1. Add syscall number to `arch/x86_64/mod.rs` and `arch/aarch64/mod.rs`
2. Implement handler in `syscall/` module
3. Wire up dispatch in `syscall/mod.rs`

### 4. Async Runtime Fails

**Symptom:** tokio panics or hangs

**Fix:** Ensure epoll syscalls are implemented:
- `epoll_create1` (x86_64: 291)
- `epoll_ctl` (x86_64: 233)
- `epoll_wait` (x86_64: 232)
- `eventfd2` (x86_64: 290)

## Testing in QEMU

1. Copy binary to initramfs:
   ```bash
   cp target/x86_64-unknown-linux-gnu/release/your-app /path/to/initramfs/bin/
   ```

2. Rebuild initramfs:
   ```bash
   cargo xtask build initramfs
   ```

3. Run in QEMU:
   ```bash
   cargo xtask run --arch x86_64
   ```

4. Execute in shell:
   ```
   /bin/your-app
   ```

## Porting Tips

### Disable Optional Features

Start with minimal features to reduce syscall requirements:

```toml
[dependencies]
some-crate = { version = "1.0", default-features = false, features = ["minimal"] }
```

### Use Single-Threaded Runtime

For async apps, start with single-threaded:

```toml
tokio = { version = "1.48", default-features = false, features = ["rt", "sync"] }
# NOT: features = ["rt-multi-thread"]
```

### Create Fallback Code

For features that may not work initially:

```rust
fn main() {
    match full_featured_main() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Full mode failed: {}, using fallback", e);
            simple_fallback();
        }
    }
}
```

## Reference: Brush Shell Port

The brush shell was ported as an example. Key decisions:

1. **Minimal features:** `default-features = false, features = ["minimal"]`
2. **Simple REPL fallback:** Until tokio works fully, provides basic shell functionality
3. **External command support:** Uses `std::process::Command` which requires `clone`/`execve`

See `crates/userspace/eyra/brush/` for the complete implementation.
