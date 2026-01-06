# los_error

Standard error handling infrastructure for the LevitateOS kernel and HAL crates.

## Purpose

This crate provides the `define_kernel_error!` macro, which enforces a consistent error reporting format across the entire project. This enables:

1. **Structured Error Codes**: Every error has a unique 16-bit code (`0xSSCC`) where `SS` is the subsystem and `CC` is the specific error variant.
2. **Deterministic Logging**: Errors are automatically formatted with their codes and human-readable descriptions.
3. **No-std Compatibility**: Designed specifically for kernel environments without access to the Rust standard library.

## Usage

Define a new error type for a subsystem:

```rust
use los_error::define_kernel_error;

define_kernel_error! {
    /// My Subsystem Errors (0x42)
    pub enum MyError(0x42) {
        /// Something failed
        Fail = 0x01 => "The operation failed",
        /// Resource busy
        Busy = 0x02 => "The resource is currently in use",
    }
}
```

## Error Code Structure

Numeric codes follow the pattern `0xSSCC`:
- `0xSS00`: Subsystem ID (e.g., `0x01` for MMU, `0x03` for Spawn).
- `0x00CC`: Specific error code within that subsystem.

This allows for quick identification of the source of a failure during kernel debugging.

## Verification

The crate includes unit tests verifying code generation and formatting, which can be run on the host:

```bash
cargo test -p los_error
```
