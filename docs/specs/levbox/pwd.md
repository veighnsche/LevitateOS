# Utility Specification: `pwd`

The `pwd` utility prints the absolute pathname of the current working directory.

## Specification References

| Standard | Link |
|----------|------|
| **POSIX.1-2017** | [pwd - return working directory name](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/pwd.html) |
| **Linux man-pages** | [pwd(1)](https://man7.org/linux/man-pages/man1/pwd.1.html) |
| **GNU Coreutils** | [pwd invocation](https://www.gnu.org/software/levbox/manual/html_node/pwd-invocation.html) |

## Synopsis

```bash
pwd [-LP]
```

## Options

| Option | Description |
|--------|-------------|
| `-L`, `--logical` | Use `$PWD` environment variable, even if it contains symlinks. (Default in POSIX) |
| `-P`, `--physical` | Resolve all symbolic links to produce the canonical path. |
| `--help` | Display usage help and exit. |
| `--version` | Output version information and exit. |

## Description

The `pwd` utility writes to standard output an absolute pathname of the current working directory.

### Logical vs Physical Paths

Consider: `/home/user/link` → `/var/data/actual`

| Mode | Current if cd'd via link | Output |
|------|--------------------------|--------|
| `-L` | Yes | `/home/user/link` |
| `-P` | Yes | `/var/data/actual` |

### Behavior Details

1. **No Arguments**: Use default mode (POSIX: `-L`, many implementations: `-P`).
2. **Output Format**: Absolute path followed by newline.
3. **No Dot/Dot-Dot**: The path shall not contain `.` or `..` components.
4. **Trailing Slash**: Path should not end with `/` (except for root `/`).

## Exit Status

| Value | Condition |
|-------|-----------|
| `0` | Current directory successfully determined and printed. |
| `>0` | An error occurred (e.g., current directory was deleted). |

## Errors

| Error | Condition |
|-------|-----------|
| `ENOENT` | Current directory has been removed (rare but possible). |
| `EACCES` | Search permission denied for a component of the path. |
| `ERANGE` | Path exceeds maximum length (unlikely with dynamic allocation). |

## Examples

```bash
# Print current directory
pwd
# Output: /home/user/projects

# Physical path (resolve symlinks)
cd /tmp/mylink  # where mylink -> /var/data
pwd -P
# Output: /var/data

# Logical path (honor symlink name)
pwd -L
# Output: /tmp/mylink
```

## Implementation Notes for LevitateOS

### Syscalls Required

| Syscall | Purpose |
|---------|---------|
| `sys_getcwd` | Get current working directory |
| `sys_readlink` | Resolve symlinks for `-P` (if needed) |
| `sys_stat` | Validate path components |
| `sys_write` | Output to stdout |

### Implementation Strategy

#### Simple Implementation (`-P` only)

Just use `getcwd`, which returns the physical path:

```rust
let mut buf = [0u8; 4096];
let len = sys_getcwd(&mut buf)?;
sys_write(1, &buf[..len])?;
sys_write(1, b"\n")?;
```

#### Full Implementation (with `-L`)

For `-L` support:
1. Check `$PWD` environment variable.
2. Verify it points to same directory as `.` (compare device/inode).
3. If valid, use `$PWD`; otherwise fall back to `getcwd`.

### Getcwd Details

```c
char *getcwd(char *buf, size_t size);
// Returns: pointer to buf on success, NULL on error
```

Syscall interface:
```c
long sys_getcwd(char *buf, unsigned long size);
// Returns: length of path (including NUL) on success, -errno on error
```

### Path Length

| Constant | Value | Use |
|----------|-------|-----|
| `PATH_MAX` | 4096 | Maximum path length (Linux) |
| `NAME_MAX` | 255 | Maximum filename component length |

### ABI Compatibility

- `getcwd` syscall number: 79 (x86_64), 17 (aarch64)
- Buffer must be large enough for path + NUL terminator
- Returns the length of the path string on success
- Common buffer size: 4096 bytes (PATH_MAX)

### Environment Variable Access

For `-L` mode, the implementation needs to read `$PWD`:

1. Parse environment from `envp` passed to `main()`.
2. Or use `getenv("PWD")` if libc is available.
3. Verify `$PWD` is valid (stat both `$PWD` and `.`, compare `st_dev`/`st_ino`).

## Help and Version Output

### `pwd --help`

```
Usage: pwd [OPTION]...
Print the full filename of the current working directory.

  -L, --logical   use PWD from environment, even if it contains symlinks
  -P, --physical  resolve all symlinks
      --help      display this help and exit
      --version   output version information and exit

If no option is specified, -P is assumed.
```

### `pwd --version`

```
pwd (LevitateOS levbox) 0.1.0
```
