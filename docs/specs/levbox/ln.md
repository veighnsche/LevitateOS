# Utility Specification: `ln`

The `ln` utility creates links (hard or symbolic) to files.

## Specification References

| Standard | Link |
|----------|------|
| **POSIX.1-2017** | [ln - link files](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/ln.html) |
| **Linux man-pages** | [ln(1)](https://man7.org/linux/man-pages/man1/ln.1.html) |
| **GNU Coreutils** | [ln invocation](https://www.gnu.org/software/levbox/manual/html_node/ln-invocation.html) |

## Synopsis

```bash
ln [-fs] source_file target_file
ln [-fs] source_file... target_dir
```

## Options

| Option | Description |
|--------|-------------|
| `-s`, `--symbolic` | Create a symbolic link instead of a hard link. |
| `-f`, `--force` | Force. Remove existing destination files. |
| `-n`, `--no-dereference` | Treat destination that is a symlink to a directory as if it were a normal file. |
| `-v`, `--verbose` | Print name of each linked file. |
| `--help` | Display usage help and exit. |
| `--version` | Output version information and exit. |

## Operands

- **source_file**: Pathname of a file to link to.
- **target_file**: Pathname for the new link.
- **target_dir**: A directory in which to create links.

## Description

The `ln` utility creates a new directory entry (link) for a file.

### Hard Links (default)

A hard link is an additional directory entry for the same inode:

- Source and target must be on the same filesystem.
- Cannot create hard links to directories (most systems).
- Removing the original file does not affect the link.
- Link count (`st_nlink`) is incremented.

### Symbolic Links (`-s`)

A symbolic link contains the pathname of another file:

- Can span filesystems.
- Can link to directories.
- Can link to non-existent targets (dangling symlink).
- Does not increment link count of target.
- Has its own inode with type `S_IFLNK`.

### Behavior with Multiple Sources

When multiple source files are given:
- The last operand must be an existing directory.
- Links are created in that directory with the source filenames.

## Exit Status

| Value | Condition |
|-------|-----------|
| `0` | All links were created successfully. |
| `>0` | An error occurred. |

## Errors

| Error | Condition |
|-------|-----------|
| `EEXIST` | Target already exists (without `-f`). |
| `ENOENT` | Source does not exist (for hard links). |
| `EXDEV` | Hard link across different filesystems. |
| `EPERM` | Attempt to hard link a directory. |
| `EACCES` | Permission denied. |
| `ENOSPC` | No space for new directory entry. |
| `EROFS` | Target is on a read-only filesystem. |
| `ELOOP` | Too many symbolic links encountered. |

## Hard Link vs Symbolic Link

| Property | Hard Link | Symbolic Link |
|----------|-----------|---------------|
| Inode | Same as target | Own inode |
| Cross-filesystem | No | Yes |
| Link to directory | No (usually) | Yes |
| Target must exist | Yes | No |
| Survives target deletion | Yes (data remains) | Becomes dangling |
| `readlink()` | N/A | Returns target path |
| `lstat()` | Same as target | Link metadata |
| Storage | Just directory entry | Path string |

## Examples

```bash
# Create hard link
ln file.txt hardlink.txt

# Create symbolic link
ln -s /path/to/target symlink

# Force overwrite existing link
ln -sf /new/target existing_link

# Create multiple links in directory
ln -s file1.txt file2.txt /home/user/links/

# Link to directory (symlink only)
ln -s /var/log logs

# Relative symbolic link
ln -s ../shared/data current_data
```

## Implementation Notes for LevitateOS

### Syscalls Required

| Syscall | Purpose |
|---------|---------|
| `sys_link` / `sys_linkat` | Create hard link |
| `sys_symlink` / `sys_symlinkat` | Create symbolic link |
| `sys_unlink` | Remove existing target for `-f` |
| `sys_stat` / `sys_lstat` | Check target type |
| `sys_access` | Check permissions |

### Implementation Strategy

1. **Argument Parsing**: Detect if last arg is directory or single target.
2. **Name Resolution**: For directory target, append source basename.
3. **Force Mode**: If `-f`, unlink existing target before creating link.
4. **Symlink Storage**: Store the exact string given, not canonicalized path.
5. **Error Handling**: Report errors but continue with remaining sources.

### Link vs Linkat

| Syscall | Features |
|---------|----------|
| `link(oldpath, newpath)` | Basic hard link |
| `linkat(olddirfd, oldpath, newdirfd, newpath, flags)` | With directory fds |

Flags for `linkat`:
- `AT_EMPTY_PATH`: Use empty string with fd referring to file
- `AT_SYMLINK_FOLLOW`: Follow symlinks (dereference source)

### Symlink vs Symlinkat

| Syscall | Features |
|---------|----------|
| `symlink(target, linkpath)` | Basic symbolic link |
| `symlinkat(target, newdirfd, linkpath)` | Relative to directory fd |

Note: For `symlink`, the `target` is stored verbatim in the symlink.

### Symbolic Link Content

The symlink stores the exact path string:

```bash
ln -s /absolute/path link1    # stores "/absolute/path"
ln -s relative/path link2     # stores "relative/path"
ln -s ../parent link3         # stores "../parent"
```

When following symlinks, the kernel resolves relative paths from the symlink's directory, not the current directory.

### ABI Compatibility

| Syscall | x86_64 | aarch64 |
|---------|--------|---------|
| `link` | 86 | — (use linkat) |
| `linkat` | 265 | 37 |
| `symlink` | 88 | — (use symlinkat) |
| `symlinkat` | 266 | 36 |
| `AT_FDCWD` | -100 | -100 |
| `AT_SYMLINK_FOLLOW` | 0x400 | 0x400 |

## Help and Version Output

### `ln --help`

```
Usage: ln [OPTION]... TARGET LINK_NAME
  or:  ln [OPTION]... TARGET
  or:  ln [OPTION]... TARGET... DIRECTORY
Create hard links by default, symbolic links with --symbolic.

  -f, --force           remove existing destination files
  -n, --no-dereference  treat LINK_NAME as a normal file if
                          it is a symbolic link to a directory
  -s, --symbolic        make symbolic links instead of hard links
  -v, --verbose         print name of each linked file
      --help            display this help and exit
      --version         output version information and exit
```

### `ln --version`

```
ln (LevitateOS levbox) 0.1.0
```
