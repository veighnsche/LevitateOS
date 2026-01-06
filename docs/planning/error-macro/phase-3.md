# Phase 3: Implementation

**Feature:** `define_kernel_error!` Macro  
**Author:** TEAM_153  
**Status:** Ready for Execution

---

## Overview

This phase creates the `levitate-error` crate with the `define_kernel_error!` macro and migrates error types.

---

## UoW Summary

| UoW | Task | Est. Lines | Dependencies |
|-----|------|------------|--------------|
| 1 | Create `levitate-error` crate with macro | ~120 | None |
| 2 | Add `levitate-error` as dependency to workspace | ~10 | UoW 1 |
| 3 | Migrate `FdtError` (simple, no nesting) | ~15 | UoW 2 |
| 4 | Migrate `SpawnError` (nested variants) | ~20 | UoW 3 |
| 5 | Verify migrations produce identical behavior | ~0 | UoW 4 |

**Total:** ~165 lines

---

## UoW 1: Create `levitate-error` Crate

### Crate Structure

```
levitate-error/
├── Cargo.toml
├── README.md
└── src/
    └── lib.rs
```

### File: `levitate-error/Cargo.toml`

```toml
[package]
name = "levitate-error"
version = "0.1.0"
edition = "2021"
description = "Error handling infrastructure for LevitateOS"

[features]
default = []

[dependencies]
# No dependencies - pure macro crate
```

### File: `levitate-error/src/lib.rs`

```rust
//! TEAM_153: Kernel error handling infrastructure.
//!
//! Provides the `define_kernel_error!` macro for consistent error type definitions.
//!
//! ## Usage
//!
//! ### Simple errors (no inner data)
//! ```ignore
//! define_kernel_error! {
//!     pub enum NetError(0x07) {
//!         NotInitialized = 0x01 => "Network device not initialized",
//!         DeviceBusy = 0x02 => "TX queue full",
//!     }
//! }
//! ```
//!
//! ### Nested errors (with inner error type)
//! ```ignore
//! define_kernel_error! {
//!     pub enum SpawnError(0x03) {
//!         Elf(ElfError) = 0x01 => "ELF loading failed",
//!         PageTable(MmuError) = 0x02 => "Page table creation failed",
//!     }
//! }
//! ```

#![no_std]

/// Macro to define a kernel error type with consistent handling.
///
/// Supports both simple variants and nested variants containing inner errors.
#[macro_export]
macro_rules! define_kernel_error {
    // Simple variants only (no inner types)
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident($subsystem:literal) {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $code:literal => $desc:literal
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        impl $name {
            /// Subsystem identifier for this error type.
            pub const SUBSYSTEM: u8 = $subsystem;

            /// Get numeric error code for debugging.
            pub const fn code(&self) -> u16 {
                match self {
                    $(Self::$variant => (($subsystem as u16) << 8) | $code,)*
                }
            }

            /// Get error name for logging.
            pub const fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant => $desc,)*
                }
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "E{:04X}: {}", self.code(), self.name())
            }
        }

        impl core::error::Error for $name {}
    };

    // Nested variants (with inner error types)
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident($subsystem:literal) {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident($inner:ty) = $code:literal => $desc:literal
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant($inner),
            )*
        }

        impl $name {
            /// Subsystem identifier for this error type.
            pub const SUBSYSTEM: u8 = $subsystem;

            /// Get numeric error code for debugging.
            pub const fn code(&self) -> u16 {
                match self {
                    $(Self::$variant(_) => (($subsystem as u16) << 8) | $code,)*
                }
            }

            /// Get error name for logging.
            pub const fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant(_) => $desc,)*
                }
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $(Self::$variant(inner) => write!(f, "E{:04X}: {} ({})", self.code(), self.name(), inner),)*
                }
            }
        }

        impl core::error::Error for $name {}
    };
}
```

---

## UoW 2: Add Dependency to Workspace

### File: `Cargo.toml` (workspace root)

Add to `[workspace]` members:
```toml
members = [
    "levitate-error",  # Add this
    # ... existing members
]
```

### File: `levitate-hal/Cargo.toml`

Add dependency:
```toml
[dependencies]
levitate-error = { path = "../levitate-error" }
```

### File: `kernel/Cargo.toml`

Add dependency:
```toml
[dependencies]
levitate-error = { path = "../levitate-error" }
```

---

## UoW 3: Migrate FdtError (Simple)

**File:** `levitate-hal/src/fdt.rs`

**Before (~35 lines):**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdtError {
    InvalidHeader,
    InitrdMissing,
}

impl FdtError {
    pub const fn code(&self) -> u16 { ... }
    pub const fn name(&self) -> &'static str { ... }
}

impl core::fmt::Display for FdtError { ... }
impl core::error::Error for FdtError {}
```

**After (~10 lines):**
```rust
use levitate_error::define_kernel_error;

define_kernel_error! {
    /// Errors that can occur during FDT parsing.
    pub enum FdtError(0x09) {
        /// [FD1] Invalid DTB header
        InvalidHeader = 0x01 => "Invalid DTB header",
        /// [FD2] Missing initrd properties
        InitrdMissing = 0x02 => "Initrd properties missing in DTB",
    }
}
```

---

## UoW 4: Migrate SpawnError (Nested)

**File:** `kernel/src/task/process.rs`

**Before (~50 lines):**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnError {
    Elf(ElfError),
    PageTable(MmuError),
    Stack(MmuError),
}

impl SpawnError {
    pub const fn code(&self) -> u16 { ... }
    pub const fn name(&self) -> &'static str { ... }
}

impl core::fmt::Display for SpawnError { ... }
impl core::error::Error for SpawnError {}
impl From<ElfError> for SpawnError { ... }
```

**After (~15 lines):**
```rust
use levitate_error::define_kernel_error;

define_kernel_error! {
    /// Process spawn errors (0x03xx)
    pub enum SpawnError(0x03) {
        /// ELF loading failed
        Elf(ElfError) = 0x01 => "ELF loading failed",
        /// Page table creation failed
        PageTable(MmuError) = 0x02 => "Page table creation failed",
        /// Stack setup failed
        Stack(MmuError) = 0x03 => "Stack setup failed",
    }
}

impl From<ElfError> for SpawnError {
    fn from(e: ElfError) -> Self {
        SpawnError::Elf(e)
    }
}
```

**Note:** `From` impls are not generated by macro - add manually as needed.

---

## UoW 5: Verify Migrations

1. `cargo build --release`
2. `cargo test -p levitate-hal -p levitate-error`
3. Verify codes:
   - `FdtError::InvalidHeader.code() == 0x0901`
   - `SpawnError::Elf(e).code() == 0x0301`
4. Verify display format:
   - `FdtError::InvalidHeader.to_string() == "E0901: Invalid DTB header"`
   - `SpawnError::Elf(e).to_string()` contains inner error

---

## Exit Criteria for Phase 3

- [ ] `levitate-error` crate created and compiles
- [ ] FdtError migrated (simple variant proof)
- [ ] SpawnError migrated (nested variant proof)
- [ ] All tests pass
- [ ] API identical to manual implementations
