# Core Utilities Specifications

This directory contains specifications for the essential file management and text tools implemented in LevitateOS, adhering to the **POSIX.1-2017** standard with Linux ABI compatibility.

## Specification Index

| Utility | Description | Options | POSIX Reference |
|---------|-------------|---------|-----------------|
| [cat](cat.md) | Concatenate and print files | `-u` | [POSIX cat](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cat.html) |
| [ls](ls.md) | List directory contents | `-a`, `-l`, `-h`, `-F`, `-R`, `-1` | [POSIX ls](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/ls.html) |
| [cp](cp.md) | Copy files and directories | `-f`, `-i`, `-p`, `-R` | [POSIX cp](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cp.html) |
| [mv](mv.md) | Move/rename files | `-f`, `-i` | [POSIX mv](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/mv.html) |
| [rm](rm.md) | Remove files and directories | `-f`, `-i`, `-r`, `-d` | [POSIX rm](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/rm.html) |
| [mkdir](mkdir.md) | Create directories | `-p`, `-m`, `-v` | [POSIX mkdir](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/mkdir.html) |
| [rmdir](rmdir.md) | Remove empty directories | `-p`, `-v` | [POSIX rmdir](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/rmdir.html) |
| [touch](touch.md) | Update timestamps/create files | `-a`, `-c`, `-m`, `-r`, `-t` | [POSIX touch](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/touch.html) |
| [pwd](pwd.md) | Print working directory | `-L`, `-P` | [POSIX pwd](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/pwd.html) |
| [ln](ln.md) | Create hard/symbolic links | `-s`, `-f` | [POSIX ln](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/ln.html) |

## Design Philosophy

These utilities are designed to be:

1. **POSIX-Compliant**: Following the IEEE Std 1003.1-2017 specification where applicable.
2. **Linux ABI Compatible**: Using standard Linux syscall numbers and data structures.
3. **Familiar Interface**: Providing the same command-line experience as GNU levbox.

## Reference Resources

| Resource | Description |
|----------|-------------|
| [POSIX.1-2017 Utilities](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/contents.html) | Official POSIX utility specifications |
| [Linux man-pages](https://man7.org/linux/man-pages/) | Linux documentation project |
| [GNU Coreutils Manual](https://www.gnu.org/software/levbox/manual/) | GNU implementation documentation |

## Syscall Dependencies

Each specification documents the required syscalls. Common dependencies include:

| Syscall | Used By |
|---------|---------|
| `open`, `openat` | cat, cp, touch, ls |
| `read`, `write` | cat, cp |
| `close` | All |
| `stat`, `lstat`, `fstat` | ls, cp, rm, mv |
| `getdents64` | ls, cp -R, rm -r |
| `mkdir`, `mkdirat` | mkdir, cp -R |
| `rmdir`, `unlinkat` | rmdir, rm |
| `rename`, `renameat` | mv |
| `link`, `linkat` | ln |
| `symlink`, `symlinkat` | ln -s |
| `getcwd` | pwd |
| `utimensat` | touch |

## Exit Code Conventions

All utilities follow standard exit code conventions:

| Exit Code | Meaning |
|-----------|---------|
| `0` | Success |
| `1` | Minor errors (e.g., file not found) |
| `2` | Serious errors (e.g., command-line usage error) |

## Implementation Status

| Utility | Status | Notes |
|---------|--------|-------|
| cat | 🔴 Planned | |
| ls | 🔴 Planned | |
| cp | 🔴 Planned | |
| mv | 🔴 Planned | |
| rm | 🔴 Planned | |
| mkdir | 🔴 Planned | |
| rmdir | 🔴 Planned | |
| touch | 🔴 Planned | |
| pwd | 🔴 Planned | |
| ln | 🔴 Planned | |

Legend: 🟢 Complete | 🟡 In Progress | 🔴 Planned
