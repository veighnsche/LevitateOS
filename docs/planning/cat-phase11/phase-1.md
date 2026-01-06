# Phase 1: Discovery — `cat` Utility

**TEAM_182** | 2026-01-06

## Feature Summary

The `cat` utility concatenates and prints files to standard output. This is the first Phase 11 coreutil and serves as a validation of the syscall layer for file operations.

### Problem Statement

Users need a command-line tool to view file contents and concatenate multiple files together. This is fundamental to any UNIX-like system.

### Who Benefits

- Users exploring the filesystem
- Shell scripting (pipelines)
- Developers debugging application behavior

## Success Criteria

1. `cat file.txt` prints file contents to stdout
2. `cat file1.txt file2.txt` concatenates and prints both files
3. `cat` (no args) reads from stdin until EOF
4. `cat -` reads from stdin at that point in the argument sequence
5. `cat -u` works (unbuffered mode — no visible difference with current implementation)
6. Exit code 0 on success, >0 on error
7. Errors print diagnostic to stderr and continue with next file

## Current State Analysis

### How Does the System Work Today?

Currently, only the shell (`lsh`) has basic `echo` and file operations are limited to kernel-level operations (ELF loading, initramfs access).

No user-facing file viewing utility exists.

### Existing Workarounds

None — users cannot view file contents interactively.

## Codebase Reconnaissance

### Code Areas Likely Touched

| Path | Change Type |
|------|-------------|
| `userspace/levbox/` | **NEW** — Create new crate |
| `userspace/Cargo.toml` | **MODIFY** — Add levbox to workspace |
| `xtask/src/image.rs` | **MODIFY** — Include levbox in initramfs |

### Public APIs Involved

- `ulib::fs::File` — Open and read files
- `ulib::io::Read` trait — Read bytes
- `libsyscall::read/write` — Raw I/O for stdin/stdout
- `libsyscall::openat/close` — File operations

### Tests or Golden Snapshots

- Behavior test will verify stdout output
- No existing golden files impacted

### Non-obvious Constraints

1. **stdin detection**: fd 0 is stdin, must handle `-` operand
2. **Binary safety**: Must handle NUL bytes in files
3. **Error handling**: Must continue after errors on individual files
4. **No std**: Must use `no_std` environment with ulib/libsyscall

## Constraints

### Performance
- Use 4096-byte read buffer (per spec)

### Compatibility
- POSIX-compliant behavior
- Only `-u` option required (unbuffered, but effectively the default anyway)

### Environment
- `no_std` — Cannot use standard library
- initramfs — Files come from FAT32 image on virtio-blk

## Questions

No blocking questions identified — all syscalls exist and spec is clear.
