// TEAM_310: Deep integration of linux-raw-sys definitions
// TEAM_210: Error constants

/// Common errno values used by LevitateOS syscalls.
///
/// All syscalls return negative errno values on error (e.g., -ENOENT = -2).
/// This module re-exports all Linux errno constants from linux-raw-sys.
///
/// ## Most Commonly Used Error Codes
///
/// | Constant | Value | Meaning |
/// |----------|-------|---------|
/// | `EPERM` | 1 | Operation not permitted |
/// | `ENOENT` | 2 | No such file or directory |
/// | `ESRCH` | 3 | No such process |
/// | `EINTR` | 4 | Interrupted system call |
/// | `EIO` | 5 | I/O error |
/// | `ENXIO` | 6 | No such device or address |
/// | `E2BIG` | 7 | Argument list too long |
/// | `ENOEXEC` | 8 | Exec format error |
/// | `EBADF` | 9 | Bad file descriptor |
/// | `ECHILD` | 10 | No child processes |
/// | `EAGAIN` | 11 | Try again (resource temporarily unavailable) |
/// | `ENOMEM` | 12 | Out of memory |
/// | `EACCES` | 13 | Permission denied |
/// | `EFAULT` | 14 | Bad address (invalid pointer) |
/// | `ENOTBLK` | 15 | Block device required |
/// | `EBUSY` | 16 | Device or resource busy |
/// | `EEXIST` | 17 | File exists |
/// | `EXDEV` | 18 | Cross-device link |
/// | `ENODEV` | 19 | No such device |
/// | `ENOTDIR` | 20 | Not a directory |
/// | `EISDIR` | 21 | Is a directory |
/// | `EINVAL` | 22 | Invalid argument |
/// | `ENFILE` | 23 | File table overflow |
/// | `EMFILE` | 24 | Too many open files |
/// | `ENOTTY` | 25 | Not a terminal |
/// | `ETXTBSY` | 26 | Text file busy |
/// | `EFBIG` | 27 | File too large |
/// | `ENOSPC` | 28 | No space left on device |
/// | `ESPIPE` | 29 | Illegal seek |
/// | `EROFS` | 30 | Read-only file system |
/// | `EMLINK` | 31 | Too many links |
/// | `EPIPE` | 32 | Broken pipe |
/// | `EDOM` | 33 | Math argument out of domain |
/// | `ERANGE` | 34 | Math result not representable |
/// | `ENAMETOOLONG` | 36 | File name too long |
/// | `ENOSYS` | 38 | Function not implemented |
/// | `ENOTEMPTY` | 39 | Directory not empty |
/// | `ELOOP` | 40 | Too many symbolic links encountered |
///
/// ## Usage Example
///
/// ```rust,no_run
/// use libsyscall::{open, O_RDONLY, errno::ENOENT};
///
/// let fd = open("/nonexistent", O_RDONLY);
/// if fd == -(ENOENT as isize) {
///     // File not found
/// }
/// ```
pub use linux_raw_sys::errno::*;
