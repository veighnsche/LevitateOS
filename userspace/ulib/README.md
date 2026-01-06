# ulib

Userspace standard library for LevitateOS.

## Overview

`ulib` is an ongoing effort to provide a richer, more idiomatic environment for LevitateOS userspace applications. It aims to eventually support a subset of the Rust `std` library functionality on top of `libsyscall`.

## Goals

1. **Memory Management**: Provide a global allocator (`malloc`/`free`) for userspace.
2. **I/O Abstractions**: Buffered reading/writing for files and console.
3. **Core Types**: Support for `String`, `Vec`, and other collection types in a `no_std` environment.
4. **ANSI Support**: Integrated text formatting and terminal control.

## Current Status

- **Phase 10 (Roadmap)**: Initial implementation of basic primitives and infrastructure.
- **Relationship with libsyscall**: `ulib` sits on top of `libsyscall`, providing higher-level abstractions.

## Usage

```rust
extern crate ulib;

fn main() {
    // Future idiomatic API
    ulib::println("Hello, LevitateOS World!");
}
```
