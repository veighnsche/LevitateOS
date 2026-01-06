# Utility Specification: `cp`

The `cp` utility copies files and directories.

## Specification References

| Standard | Link |
|----------|------|
| **POSIX.1-2017** | [cp - copy files](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cp.html) |
| **Linux man-pages** | [cp(1)](https://man7.org/linux/man-pages/man1/cp.1.html) |
| **GNU Coreutils** | [cp invocation](https://www.gnu.org/software/levbox/manual/html_node/cp-invocation.html) |

## Synopsis

```bash
cp [-fipRr] source_file target_file
cp [-fipRr] source_file... target_directory
```

## Options

| Option | Description |
|--------|-------------|
| `-f` | Force. If the target file cannot be opened, remove it and try again. (Do not prompt.) |
| `-i` | Interactive. Prompt before overwriting existing files. |
| `-p` | Preserve file attributes (mode, ownership, timestamps). |
| `-R`, `-r` | Recursive. Copy directories and their contents. |
| `-v`, `--verbose` | Explain what is being done. |
| `--help` | Display usage help and exit. |
| `--version` | Output version information and exit. |

## Operands

- **source_file**: A pathname of a file to be copied.
- **target_file**: A pathname of the new file created by the copy.
- **target_directory**: A pathname of a directory to contain the copied files.

## Description

### Single File Copy

When `cp` is invoked with two operands, the first is the source and the second is the target:

- If the target exists and is a directory, the source is copied into that directory with its original name.
- If the target exists and is a file, it is overwritten (unless `-i` is specified).
- If the target does not exist, it is created.

### Multiple Files to Directory

When more than two operands are given, the last operand must be a directory. All preceding operands are copied into that directory.

### Copying Behavior

1. **Regular Files**: Content is copied byte-for-byte.
2. **Directories** (with `-R`): Recursively copies contents, recreating the directory structure.
3. **Symbolic Links**: By default, the link target is copied. With `-R -P`, the link itself is copied.
4. **Special Files**: Behavior is implementation-defined.

### Preserve Mode (`-p`)

When `-p` is specified:

| Attribute | Preserved |
|-----------|-----------|
| Access time | Yes |
| Modification time | Yes |
| User ID | Yes (if privileged) |
| Group ID | Yes (if privileged) |
| File mode bits | Yes |

## Exit Status

| Value | Condition |
|-------|-----------|
| `0` | All files were copied successfully. |
| `>0` | An error occurred. |

## Errors

| Error | Condition |
|-------|-----------|
| `ENOENT` | Source file does not exist. |
| `ENOTDIR` | Target is not a directory when multiple sources given. |
| `EISDIR` | Source is a directory but `-R` not specified. |
| `EACCES` | Permission denied. |
| `ENOSPC` | No space left on device. |
| `EXDEV` | Cross-device copy (may require fallback behavior). |

## Examples

```bash
# Copy a single file
cp source.txt dest.txt

# Copy file into directory
cp file.txt /home/user/

# Copy multiple files to directory
cp file1.txt file2.txt file3.txt /backup/

# Recursive copy of directory
cp -R /source/dir /dest/dir

# Preserve attributes
cp -p important.doc backup.doc

# Force overwrite
cp -f config.old config.new

# Interactive mode
cp -i newfile existingfile
```

## Implementation Notes for LevitateOS

### Syscalls Required

| Syscall | Purpose |
|---------|---------|
| `sys_open` | Open source for reading, target for writing |
| `sys_read` | Read source file |
| `sys_write` | Write to target file |
| `sys_close` | Close file descriptors |
| `sys_fstat` / `sys_stat` | Check file type, get metadata |
| `sys_mkdir` | Create directories for `-R` |
| `sys_getdents64` | Read directory contents for `-R` |
| `sys_chmod` | Set file mode for `-p` |
| `sys_chown` | Set ownership for `-p` |
| `sys_utimes` | Set timestamps for `-p` |
| `sys_unlink` | Remove target for `-f` |

### Implementation Strategy

1. **Buffer Size**: Use 64KB buffer for efficient large file copies.
2. **Stat First**: Determine if source is file or directory before proceeding.
3. **Directory Copy**: Build list of entries, create target directory, recursively copy.
4. **Atomic Operations**: For safety, consider copy-to-temp then rename pattern.
5. **Error Continuation**: On error with multiple sources, continue with remaining files.

### ABI Compatibility

- File permissions use standard Unix mode bits (e.g., 0644, 0755).
- Timestamps are in `struct timespec` format.
- O_CREAT | O_WRONLY | O_TRUNC flags for creating target file.

## Help and Version Output

### `cp --help`

```
Usage: cp [OPTION]... SOURCE DEST
  or:  cp [OPTION]... SOURCE... DIRECTORY
Copy SOURCE to DEST, or multiple SOURCE(s) to DIRECTORY.

  -f, --force        if an existing destination file cannot be opened,
                       remove it and try again
  -i, --interactive  prompt before overwrite
  -p                 same as --preserve=mode,ownership,timestamps
  -R, -r, --recursive  copy directories recursively
  -v, --verbose      explain what is being done
      --help         display this help and exit
      --version      output version information and exit
```

### `cp --version`

```
cp (LevitateOS levbox) 0.1.0
```
