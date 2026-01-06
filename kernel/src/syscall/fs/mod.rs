//! TEAM_208: Filesystem syscalls module
//!
//! Refactored from fs.rs into submodules for maintainability:
//! - `read.rs` - sys_read, stdin helpers
//! - `write.rs` - sys_write
//! - `open.rs` - sys_openat, sys_close
//! - `stat.rs` - sys_fstat
//! - `dir.rs` - sys_getdents, sys_getcwd, sys_mkdirat, sys_unlinkat, sys_renameat
//! - `link.rs` - sys_utimensat, sys_symlinkat, sys_readlinkat
//! - `mount.rs` - sys_mount, sys_umount

mod dir;
mod link;
mod mount;
mod open;
mod read;
mod stat;
mod write;

// Re-export all syscall functions
pub use dir::{sys_getcwd, sys_getdents, sys_mkdirat, sys_renameat, sys_unlinkat};
pub use link::{sys_readlinkat, sys_symlinkat, sys_utimensat};
pub use mount::{sys_mount, sys_umount};
pub use open::{sys_close, sys_openat};
pub use read::sys_read;
pub use stat::sys_fstat;
pub use write::sys_write;
