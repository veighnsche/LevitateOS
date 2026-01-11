//! Static libc for LevitateOS
//!
//! This crate builds c-gull as a static library (libc.a) that can be used
//! to link Rust programs without depending on glibc/musl.
//!
//! The resulting library provides:
//! - All standard libc functions (from c-gull/c-scape)
//! - Program startup code (from origin)
//! - malloc/free (from rustix-dlmalloc)
//! - Thread support

#![no_std]

// Pull in c-gull which provides all libc symbols via #[no_mangle] exports
extern crate c_gull;

// Re-export everything to ensure the linker sees all symbols
pub use c_gull::*;
