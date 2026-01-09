//! TEAM_166: Userspace Standard Library (`ulib`) for LevitateOS.
//!
//! This library provides `std`-like abstractions for userspace programs:
//! - Global allocator backed by `sbrk`
//! - File I/O abstractions
//! - Environment access
//! - Time utilities
//!
//! ## Usage
//! ```rust
//! // Enable the allocator in your binary
//! extern crate ulib;
//!
//! // Now you can use Vec, Box, String, etc.
//! let mut v = Vec::new();
//! v.push(42);
//! ```

#![no_std]
#![feature(alloc_error_handler)]

// TEAM_212: Process entry and lifecycle
pub mod entry;
pub mod panic;

// TEAM_166: Module structure per Phase 10 design
pub mod alloc;
// TEAM_168: File and I/O modules
pub mod fs;
pub mod io;
// TEAM_169: Environment and argument access
pub mod env;
// TEAM_170: Time abstractions
pub mod time;

// Re-export commonly used items
pub use alloc::LosAllocator;
pub use fs::File;
// TEAM_176: Re-export directory iteration types
pub use fs::{read_dir, DirEntry, FileType, ReadDir};
pub use io::{Error, ErrorKind, Read, Result, Write};
// TEAM_180: Re-export buffered I/O types
pub use io::{BufReader, BufWriter};
// TEAM_212: Re-export process lifecycle functions
pub use entry::{abort, atexit, exit, sched_yield, sleep};
pub use entry::{kill, pause, raise, signal, Signal};

// Re-export libsyscall for convenience
pub use libsyscall;

/// TEAM_166: Prelude module for convenient imports.
pub mod prelude {
    pub use crate::alloc::LosAllocator;
    pub use libsyscall::{print, println};
}
