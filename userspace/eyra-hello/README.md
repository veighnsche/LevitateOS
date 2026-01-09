# Eyra Hello - Test Binary for LevitateOS std Support

**TEAM_351** | Created: 2026-01-09

## Purpose

This crate tests Rust `std` support on LevitateOS using [Eyra](https://github.com/sunfishcode/eyra), 
a pure-Rust implementation of `std` that makes Linux syscalls directly.

## Quick Start

```bash
# Build for x86_64 (uses rust-toolchain.toml automatically)
cargo build --release --target x86_64-unknown-linux-gnu -Zbuild-std=std,panic_abort

# Build for aarch64
cargo build --release --target aarch64-unknown-linux-gnu -Zbuild-std=std,panic_abort

# Test on Linux
./target/x86_64-unknown-linux-gnu/release/eyra-hello
```

## Toolchain

This crate pins to **`nightly-2025-04-28`** via `rust-toolchain.toml`.
This version is tested and known to work with Eyra 0.22.

## Build Output

```
=== Eyra Test on LevitateOS ===

[OK] println! works
[OK] argc = 1
     argv[0] = './eyra-hello'
[OK] Instant::now() works
[OK] elapsed = 2.197µs
[OK] HashMap works (getrandom ok), value = 42

=== Eyra Test Complete ===
```

## Alternative: Use ulib

If Eyra build issues persist, test syscalls using LevitateOS's native `ulib`:

```rust
// userspace/ulib-test/src/main.rs
#![no_std]
#![no_main]

extern crate ulib;

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    ulib::println!("Testing new syscalls...");
    
    // Test gettid
    let tid = unsafe { ulib::syscall::gettid() };
    ulib::println!("gettid() = {}", tid);
    
    // Test getuid
    let uid = unsafe { ulib::syscall::getuid() };
    ulib::println!("getuid() = {}", uid);
    
    0
}
```

## Syscalls Required by Eyra

The following syscalls have been implemented for Eyra support (TEAM_350):

| Syscall | aarch64 | x86_64 | Status |
|---------|---------|--------|--------|
| gettid | 178 | 186 | ✅ |
| exit_group | 94 | 231 | ✅ |
| getuid/geteuid | 174-175 | 102/107 | ✅ |
| getgid/getegid | 176-177 | 104/108 | ✅ |
| clock_getres | 114 | 229 | ✅ |
| madvise | 233 | 28 | ✅ (stub) |
| getrandom | 278 | 318 | ✅ |
| arch_prctl | N/A | 158 | ✅ |
| faccessat | 48 | 269 | ✅ |

## References

- [Eyra GitHub](https://github.com/sunfishcode/eyra)
- [Origin (startup code)](https://github.com/sunfishcode/origin)
- [rustix (syscalls)](https://github.com/bytecodealliance/rustix)
