# Utility Specification: `mv`

The `mv` utility moves (renames) files and directories.

## Specification References

| Standard | Link |
|----------|------|
| **POSIX.1-2017** | [mv - move files](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/mv.html) |
| **Linux man-pages** | [mv(1)](https://man7.org/linux/man-pages/man1/mv.1.html) |
| **GNU Coreutils** | [mv invocation](https://www.gnu.org/software/levbox/manual/html_node/mv-invocation.html) |

## Synopsis

```bash
mv [-fi] source_file target_file
mv [-fi] source_file... target_directory
```

## Options

| Option | Description |
|--------|-------------|
| `-f`, `--force` | Force. Do not prompt for confirmation before overwriting the destination. |
| `-i`, `--interactive` | Interactive. Prompt before overwriting existing destination files. |
| `-v`, `--verbose` | Explain what is being done. |
| `--help` | Display usage help and exit. |
| `--version` | Output version information and exit. |

When both `-f` and `-i` are specified, the last one takes precedence.

## Operands

- **source_file**: A pathname of a file or directory to be moved.
- **target_file**: A new pathname for the file or directory.
- **target_directory**: A pathname of an existing directory to contain the moved files.

## Description

The `mv` utility performs the following operations:

### Rename (Same Filesystem)

When source and target are on the same filesystem, `mv` performs an atomic `rename()` operation:

1. If target does not exist, source is renamed to target.
2. If target exists and is a file, it is removed and source is renamed.
3. If target exists and is a directory, source is moved into that directory.

### Move (Cross-Filesystem)

When source and target are on different filesystems:

1. The source file/directory is copied to the target.
2. The source file/directory is removed.
3. This is **not atomic** and may leave partial state on failure.

### Directory Behavior

- If source is a directory and target does not exist, source is renamed.
- If source is a directory and target is an existing directory, source is moved into target.
- A directory cannot be moved into itself.

## Exit Status

| Value | Condition |
|-------|-----------|
| `0` | All files were moved successfully. |
| `>0` | An error occurred. |

## Errors

| Error | Condition |
|-------|-----------|
| `ENOENT` | Source file does not exist. |
| `ENOTDIR` | Target is not a directory when multiple sources given. |
| `EISDIR` | Cannot overwrite directory with non-directory. |
| `EACCES` | Permission denied. |
| `EBUSY` | Source or target is a mount point or otherwise busy. |
| `EINVAL` | Attempt to move a directory into itself. |
| `EXDEV` | Cross-device move requires copy+delete fallback. |
| `ENOSPC` | No space left for cross-device move. |

## Examples

```bash
# Rename a file
mv oldname.txt newname.txt

# Move file to directory
mv file.txt /home/user/documents/

# Move multiple files to directory
mv file1.txt file2.txt file3.txt /backup/

# Rename a directory
mv old_dir new_dir

# Force overwrite without prompt
mv -f source.txt existing.txt

# Interactive mode
mv -i source.txt existing.txt
```

## Implementation Notes for LevitateOS

### Syscalls Required

| Syscall | Purpose |
|---------|---------|
| `sys_rename` / `sys_renameat` / `sys_renameat2` | Atomic rename on same filesystem |
| `sys_stat` / `sys_lstat` | Check file type and get device info |
| `sys_open` | For cross-device copy fallback |
| `sys_read` / `sys_write` | For cross-device copy |
| `sys_unlink` / `sys_rmdir` | Remove source after cross-device copy |
| `sys_mkdir` | Create directories for cross-device dir move |
| `sys_getdents64` | Read directory for cross-device dir move |

### Implementation Strategy

1. **Try Rename First**: Always attempt `sys_rename` first.
2. **Check for EXDEV**: If rename fails with `EXDEV`, fall back to copy+delete.
3. **Cross-Device Copy**: Implement as recursive `cp -R` followed by recursive `rm -rf`.
4. **Preserve Semantics**: Cross-device move should preserve all attributes like `-p` copy.
5. **Directory Validation**: Check that target is not a subdirectory of source.

### Rename vs Renameat2

| Syscall | Features |
|---------|----------|
| `rename` | Basic rename |
| `renameat` | Relative to directory fd |
| `renameat2` | Adds flags: `RENAME_NOREPLACE`, `RENAME_EXCHANGE`, `RENAME_WHITEOUT` |

For basic `mv`, `renameat` is sufficient. `renameat2` with `RENAME_NOREPLACE` can implement safer `-n` (no-clobber) option.

### ABI Compatibility

- `rename()` syscall number: 38 (x86_64), 82 (aarch64)
- `renameat2` is preferred for modern Linux compatibility

## Help and Version Output

### `mv --help`

```
Usage: mv [OPTION]... SOURCE DEST
  or:  mv [OPTION]... SOURCE... DIRECTORY
Rename SOURCE to DEST, or move SOURCE(s) to DIRECTORY.

  -f, --force        do not prompt before overwriting
  -i, --interactive  prompt before overwrite
  -v, --verbose      explain what is being done
      --help         display this help and exit
      --version      output version information and exit

If you specify more than one of -i, -f, only the final one takes effect.
```

### `mv --version`

```
mv (LevitateOS levbox) 0.1.0
```
