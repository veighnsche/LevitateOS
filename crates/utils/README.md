# los_utils

Core utilities library for LevitateOS — provides fundamental data structures and algorithms that are used across all kernel crates.

## Purpose

This crate contains **platform-independent, `no_std`-compatible** utilities that can be:

1. Used in the kernel without heap allocation overhead
2. Unit tested on the host with the `std` feature
3. Shared between `los_hal`, `los_term`, and the kernel

## Architecture

```
crates/utils/src/
├── lib.rs    # Spinlock, RingBuffer implementations
├── cpio.rs   # CPIO archive parser (initramfs)
└── hex.rs    # Hex formatting utilities
```

## Key Components

### Synchronization Primitives

This crate re-exports synchronization primitives from the `spin` crate:

- **Mutex**: Re-exported from `spin::Mutex`.
- **RwLock**: Alias for `spin::RwLock`.
- **Once**: Alias for `spin::Once`.
- **Lazy**: Alias for `spin::Lazy`.
- **Barrier**: Alias for `spin::Barrier`.

Reference the [spin crate documentation](https://docs.rs/spin) for detailed behaviors.

### Collections

This crate re-exports collections from the `hashbrown` crate:

- **HashMap**: High-performance hash map.
- **HashSet**: High-performance hash set.

**Note**: Usage requires `extern crate alloc` in your crate root, as these collections are heap-allocated.

### RingBuffer (`lib.rs`)

A fixed-size, const-generic ring buffer for byte streams (e.g., UART RX):

```rust
let mut rb = RingBuffer::<1024>::new();
rb.push(0x41);           // Returns true (success)
let byte = rb.pop();     // Returns Some(0x41)
```

**Behaviors (R1-R8):**
- **R1**: New buffer is empty
- **R2**: Push adds element
- **R3**: Pop removes oldest (FIFO order)
- **R4**: Push returns false when full
- **R5**: Pop returns None when empty
- **R6**: Wraps around at capacity
- **R7/R8**: `is_empty()` correctness

### CPIO Parser (`cpio.rs`)

Parses CPIO New ASCII Format archives (used for initramfs):

```rust
let archive = CpioArchive::new(initramfs_bytes);

// Iterate over files
for entry in archive.iter() {
    println!("{}: {} bytes", entry.name, entry.data.len());
}

// Get specific file
if let Some(data) = archive.get_file("init") {
    // Execute init
}
```

**Supported Formats:**
- `070701` — CPIO newc format
- `070702` — CPIO newc with CRC

**Behaviors (CP1-CP10):**
- **CP1/CP2**: Valid magic detection
- **CP3**: Invalid magic rejection
- **CP4/CP5**: Hex string parsing
- **CP6**: Ordered iteration
- **CP7**: Stops at TRAILER!!!
- **CP8/CP9**: File lookup
- **CP10**: 4-byte alignment handling

### Hex Formatting (`hex.rs`)

Pure functions for converting values to hexadecimal strings without heap allocation:

```rust
let mut buf = [0u8; 18];
let hex_str = format_hex(0xDEADBEEF, &mut buf);
// hex_str == "0x00000000deadbeef"

let c = nibble_to_hex(10);  // 'a'
```

**Behaviors (C1-C5):**
- **C1**: Zero converts correctly
- **C2**: Max u64 converts correctly
- **C3**: Mixed nibbles work
- **C4**: 0-9 → '0'-'9'
- **C5**: 10-15 → 'a'-'f'

## Features

| Feature | Description |
|---------|-------------|
| `std` | Enables `std` library for host-side unit tests |

## Building & Testing

```bash
# Build (no_std, for kernel)
cargo build -p los_utils

# Run unit tests (requires std feature)
cargo test -p los_utils --features std --target x86_64-unknown-linux-gnu
```

## Design Principles

1. **Minimal Dependencies**: Uses `spin` and `hashbrown`, relies on `core` and `alloc`.
2. **Const-Friendly**: All constructors are `const fn` for static initialization
3. **No Panics**: Functions return `Option` or `bool` instead of panicking
4. **Testable**: All logic can be tested on the host with `--features std`

## Usage in Other Crates

```toml
[dependencies]
los_utils = { path = "../utils" }
```

```rust
use los_utils::{Mutex, RingBuffer};
use los_utils::{HashMap, HashSet};
use los_utils::cpio::CpioArchive;
use los_utils::hex::{format_hex, nibble_to_hex};
```
