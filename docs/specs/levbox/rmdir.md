# Utility Specification: `rmdir`

The `rmdir` utility removes empty directories.

## Specification References

| Standard | Link |
|----------|------|
| **POSIX.1-2017** | [rmdir - remove directories](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/rmdir.html) |
| **Linux man-pages** | [rmdir(1)](https://man7.org/linux/man-pages/man1/rmdir.1.html) |
| **GNU Coreutils** | [rmdir invocation](https://www.gnu.org/software/levbox/manual/html_node/rmdir-invocation.html) |

## Synopsis

```bash
rmdir [-pv] directory...
```

## Options

| Option | Description |
|--------|-------------|
| `-p`, `--parents` | Remove directory and its ancestors. (e.g., `rmdir -p a/b/c` is like `rmdir a/b/c a/b a`) |
| `-v`, `--verbose` | Print a message for each removed directory. |
| `--ignore-fail-on-non-empty` | Do not error on non-empty directories. |
| `--help` | Display usage help and exit. |
| `--version` | Output version information and exit. |

## Operands

- **directory**: A pathname of an empty directory to be removed.

## Description

The `rmdir` utility removes each specified directory, provided it is empty.

### Empty Directory Requirement

A directory is considered empty if it contains only the entries `.` (self) and `..` (parent). Any other entries (files, subdirectories, symlinks) will cause `rmdir` to fail with `ENOTEMPTY`.

### Parent Removal (`-p`)

When `-p` is specified, after removing the named directory, `rmdir` attempts to remove each parent directory component up to the root or until a removal fails.

Example:
```bash
rmdir -p a/b/c
# Equivalent to:
rmdir a/b/c
rmdir a/b
rmdir a
```

Removal stops at the first failure (e.g., non-empty parent).

## Exit Status

| Value | Condition |
|-------|-----------|
| `0` | All directories were removed successfully. |
| `>0` | An error occurred. |

## Errors

| Error | Condition |
|-------|-----------|
| `ENOTEMPTY` | Directory is not empty. |
| `ENOENT` | Directory does not exist. |
| `ENOTDIR` | Path component is not a directory. |
| `EACCES` | Permission denied. |
| `EBUSY` | Directory is a mount point or in use. |
| `EINVAL` | Attempt to remove `.` (current directory). |
| `EPERM` | Permission denied due to sticky bit on parent. |

## Comparison with `rm -d`

| Command | Empty | Non-empty | Recursive |
|---------|-------|-----------|-----------|
| `rmdir dir` | ✓ | ✗ (error) | N/A |
| `rm -d dir` | ✓ | ✗ (error) | N/A |
| `rm -r dir` | ✓ | ✓ | Yes |

`rmdir` is the safe choice when you specifically want to remove only empty directories.

## Examples

```bash
# Remove a single empty directory
rmdir empty_dir

# Remove multiple empty directories
rmdir dir1 dir2 dir3

# Remove directory and its empty parents
rmdir -p ./path/to/empty/dir

# Verbose removal
rmdir -v old_temp_dir
# Output: rmdir: removing directory, 'old_temp_dir'

# Remove with parent, stops at non-empty
rmdir -p a/b/c  # removes c, b, stops if a has other contents
```

## Implementation Notes for LevitateOS

### Syscalls Required

| Syscall | Purpose |
|---------|---------|
| `sys_rmdir` | Remove an empty directory |
| `sys_unlinkat(..., AT_REMOVEDIR)` | Alternative: remove directory |

### Implementation Strategy

1. **Simple Case**: Without `-p`, just call `rmdir(path)`.
2. **Parent Removal (-p)**:
   - Remove the specified directory first.
   - Split path into components.
   - Remove parent directories in order until failure or root.
3. **Error Handling**: Report errors but continue with remaining operands.
4. **Verbose Mode**: Print to stdout before each removal attempt.

### Path Processing for `-p`

```
Input: a/b/c
Steps:
1. rmdir("a/b/c") - must succeed
2. rmdir("a/b") - may fail (ENOTEMPTY OK)
3. rmdir("a") - may fail
```

For `-p`, failures after the first successful removal are often ignored.

### Rmdir vs Unlinkat

| Syscall | Use |
|---------|-----|
| `rmdir(path)` | Simple empty directory removal |
| `unlinkat(dirfd, path, AT_REMOVEDIR)` | With directory fd reference |

### ABI Compatibility

- `rmdir` syscall number: 84 (x86_64), not present on aarch64 (use unlinkat)
- `unlinkat` with `AT_REMOVEDIR` (0x200): 263 (x86_64), 35 (aarch64)
- Error `ENOTEMPTY`: 39 (Linux)

## Help and Version Output

### `rmdir --help`

```
Usage: rmdir [OPTION]... DIRECTORY...
Remove the DIRECTORY(ies), if they are empty.

      --ignore-fail-on-non-empty
                    ignore each failure to remove a non-empty directory
  -p, --parents     remove DIRECTORY and its ancestors;
                      e.g., 'rmdir -p a/b' is similar to 'rmdir a/b a'
  -v, --verbose     output a diagnostic for every directory processed
      --help        display this help and exit
      --version     output version information and exit
```

### `rmdir --version`

```
rmdir (LevitateOS levbox) 0.1.0
```
