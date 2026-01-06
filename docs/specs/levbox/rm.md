# Utility Specification: `rm`

The `rm` utility removes directory entries (files and directories).

## Specification References

| Standard | Link |
|----------|------|
| **POSIX.1-2017** | [rm - remove directory entries](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/rm.html) |
| **Linux man-pages** | [rm(1)](https://man7.org/linux/man-pages/man1/rm.1.html) |
| **GNU Coreutils** | [rm invocation](https://www.gnu.org/software/levbox/manual/html_node/rm-invocation.html) |

## Synopsis

```bash
rm [-fiRrd] file...
```

## Options

| Option | Description |
|--------|-------------|
| `-f`, `--force` | Force. Do not prompt for confirmation. Ignore nonexistent files. |
| `-i` | Interactive. Prompt before every removal. |
| `-I` | Prompt once before removing more than three files, or when removing recursively. |
| `-R`, `-r`, `--recursive` | Recursive. Remove directories and their contents. |
| `-d`, `--dir` | Remove empty directories (equivalent to `rmdir`). |
| `-v`, `--verbose` | Explain what is being done. |
| `--help` | Display usage help and exit. |
| `--version` | Output version information and exit. |

Option precedence:
- If `-f` and `-i` are both specified, the last one wins.
- `-r` and `-R` are identical.

## Operands

- **file**: A pathname of a directory entry to be removed.

## Description

The `rm` utility removes the directory entry specified by each file operand.

### Removal Behavior

1. **Regular Files**: The file is unlinked. If link count becomes 0, storage is freed.
2. **Symbolic Links**: The link itself is removed (not the target).
3. **Directories**: Require `-r`/`-R` flag. Contents removed first (post-order traversal).
4. **Special Files**: Removed like regular files.

### Interactive Mode (`-i`)

In interactive mode, `rm` prompts:
- `rm: remove regular file 'filename'?`
- `rm: descend into directory 'dirname'?`

Response beginning with `y` or `Y` means yes.

### Force Mode (`-f`)

In force mode:
- Never prompt for confirmation.
- Do not report errors for nonexistent files.
- Override previous `-i` option.

### Recursive Removal (`-R`)

For directories:
1. Descend into directory.
2. Remove all entries recursively.
3. Remove the directory itself.

This is post-order traversal: children before parents.

## Exit Status

| Value | Condition |
|-------|-----------|
| `0` | All specified files were removed. |
| `>0` | An error occurred. |

## Errors

| Error | Condition |
|-------|-----------|
| `ENOENT` | File does not exist (error unless `-f`). |
| `EACCES` | Permission denied. |
| `EBUSY` | File is in use (e.g., mount point). |
| `EISDIR` | File is a directory and `-r` not specified. |
| `ENOTEMPTY` | Directory not empty (for `rmdir` behavior). |
| `EPERM` | Sticky bit set and user doesn't own file. |

## Safety Considerations

> [!CAUTION]
> `rm -rf /` or `rm -rf ~` can cause irreversible data loss. Modern implementations include safeguards:
> - Refuse to remove `/` without `--no-preserve-root` (GNU extension).
> - Some systems require confirmation for root directory operations.

## Examples

```bash
# Remove a single file
rm file.txt

# Remove multiple files
rm file1.txt file2.txt file3.txt

# Force remove without prompting
rm -f maybe_exists.txt

# Interactive removal
rm -i important.doc

# Remove directory and contents
rm -r directory/

# Remove directory tree forcefully
rm -rf temp_build/

# Remove empty directory
rm -d empty_dir/
```

## Implementation Notes for LevitateOS

### Syscalls Required

| Syscall | Purpose |
|---------|---------|
| `sys_unlink` / `sys_unlinkat` | Remove regular files and symlinks |
| `sys_rmdir` | Remove empty directories |
| `sys_openat` | Open directory for reading (recursive) |
| `sys_getdents64` | List directory contents (recursive) |
| `sys_lstat` / `sys_fstatat` | Determine file type |
| `sys_access` | Check permissions for interactive mode |
| `sys_isatty` | Check if stdin is TTY for `-i` prompts |

### Implementation Strategy

1. **Type Check First**: Use `lstat` to determine if entry is file, directory, or symlink.
2. **Post-Order Traversal**: For `-r`, recursively remove children before parent directory.
3. **Stack-Based Recursion**: Use explicit stack to avoid stack overflow on deep directories.
4. **Error Collection**: Continue removing remaining files after one error.
5. **Symlink Safety**: Never follow symlinks when descending.

### Unlink vs Unlinkat

| Syscall | Flags | Use |
|---------|-------|-----|
| `unlink` | — | Remove file by path |
| `unlinkat` | `0` | Remove file relative to dirfd |
| `unlinkat` | `AT_REMOVEDIR` | Remove directory (like rmdir) |

Using `unlinkat` with appropriate flags provides a unified interface.

### ABI Compatibility

- `unlink` syscall number: 87 (x86_64), 1026 (aarch64)
- `unlinkat` syscall number: 263 (x86_64), 35 (aarch64)
- `AT_REMOVEDIR` flag value: 0x200

## Help and Version Output

### `rm --help`

```
Usage: rm [OPTION]... [FILE]...
Remove (unlink) the FILE(s).

  -f, --force       ignore nonexistent files and arguments, never prompt
  -i                prompt before every removal
  -I                prompt once before removing more than three files, or
                      when removing recursively
  -r, -R, --recursive  remove directories and their contents recursively
  -d, --dir         remove empty directories
  -v, --verbose     explain what is being done
      --help        display this help and exit
      --version     output version information and exit

By default, rm does not remove directories.  Use the --recursive (-r or -R)
option to remove each listed directory, too, along with all of its contents.
```

### `rm --version`

```
rm (LevitateOS levbox) 0.1.0
```
