# Utility Specification: `touch`

The `touch` utility updates file access and modification times, or creates files.

## Specification References

| Standard | Link |
|----------|------|
| **POSIX.1-2017** | [touch - change file timestamps](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/touch.html) |
| **Linux man-pages** | [touch(1)](https://man7.org/linux/man-pages/man1/touch.1.html) |
| **GNU Coreutils** | [touch invocation](https://www.gnu.org/software/levbox/manual/html_node/touch-invocation.html) |

## Synopsis

```bash
touch [-acm] [-r ref_file | -t time | -d date] file...
```

## Options

| Option | Description |
|--------|-------------|
| `-a` | Change only the access time. |
| `-c`, `--no-create` | Do not create the file if it does not exist. |
| `-m` | Change only the modification time. |
| `-r`, `--reference=FILE` | Use the times from `FILE` instead of current time. |
| `-t STAMP` | Use the specified time in `[[CC]YY]MMDDhhmm[.SS]` format. |
| `-d`, `--date=STRING` | Parse STRING and use it instead of current time. |
| `--help` | Display usage help and exit. |
| `--version` | Output version information and exit. |

## Operands

- **file**: A pathname of a file whose times shall be modified.

## Description

The `touch` utility affects each file in one of the following ways:

1. **Update Timestamps**: If the file exists, set access/modification times.
2. **Create File**: If the file does not exist (and `-c` not specified), create an empty file.

### Time Selection

| Options | Access Time | Modification Time |
|---------|-------------|-------------------|
| (none) | Updated | Updated |
| `-a` | Updated | Unchanged |
| `-m` | Unchanged | Updated |
| `-a -m` | Updated | Updated |

### Time Sources

| Option | Source |
|--------|--------|
| (none) | Current system time |
| `-r ref_file` | Times from reference file |
| `-t [[CC]YY]MMDDhhmm[.SS]` | Explicit time specification |
| `-d string` | Parsed date string (GNU extension) |

### Time Format (`-t`)

Format: `[[CC]YY]MMDDhhmm[.SS]`

| Component | Meaning | Example |
|-----------|---------|---------|
| CC | Century (first two digits of year) | 20 |
| YY | Year within century | 25 |
| MM | Month (01-12) | 01 |
| DD | Day (01-31) | 06 |
| hh | Hour (00-23) | 15 |
| mm | Minute (00-59) | 30 |
| SS | Second (00-60, 60 for leap second) | 00 |

Example: `-t 202501061530.00` = January 6, 2025, 15:30:00

## Exit Status

| Value | Condition |
|-------|-----------|
| `0` | All files processed successfully. |
| `>0` | An error occurred. |

## Errors

| Error | Condition |
|-------|-----------|
| `ENOENT` | File does not exist and `-c` was specified. |
| `EACCES` | Permission denied (to create or modify). |
| `EROFS` | File is on a read-only filesystem. |
| `EINVAL` | Invalid time specification. |
| `ENOTDIR` | A path component is not a directory. |

## Examples

```bash
# Create file or update timestamps to now
touch newfile.txt

# Update only access time
touch -a file.txt

# Update only modification time
touch -m file.txt

# Don't create if doesn't exist
touch -c maybe_exists.txt

# Use timestamps from another file
touch -r reference.txt target.txt

# Set specific time
touch -t 202501061530.00 file.txt

# Set time using date string (GNU)
touch -d "2025-01-06 15:30:00" file.txt
touch -d "yesterday" file.txt
touch -d "2 hours ago" file.txt
```

## Implementation Notes for LevitateOS

### Syscalls Required

| Syscall | Purpose |
|---------|---------|
| `sys_open` | Create file if doesn't exist |
| `sys_close` | Close newly created file |
| `sys_utimes` / `sys_utimensat` | Set file timestamps |
| `sys_stat` | Get reference file timestamps for `-r` |
| `sys_access` | Check file existence for `-c` |

### Implementation Strategy

1. **File Creation**: Use `open(path, O_CREAT | O_WRONLY, 0666)` then immediately `close()`.
2. **Time Update**: Use `utimensat` for nanosecond precision.
3. **Reference Times**: `stat` the reference file, extract `st_atim` and `st_mtim`.
4. **Current Time**: Pass `UTIME_NOW` to utimensat.
5. **Selective Update**: Use `UTIME_OMIT` to skip one timestamp.

### Utimensat Details

```c
int utimensat(int dirfd, const char *pathname,
              const struct timespec times[2], int flags);
```

| times[] | Meaning |
|---------|---------|
| `{UTIME_NOW, UTIME_NOW}` | Both to current time |
| `{UTIME_OMIT, ...}` | Don't change access time |
| `{..., UTIME_OMIT}` | Don't change modification time |
| Specific values | Set to that time |

Special values:
- `UTIME_NOW` = `((1l << 30) - 1l)`
- `UTIME_OMIT` = `((1l << 30) - 2l)`

### ABI Compatibility

- `utimensat` syscall number: 280 (x86_64), 88 (aarch64)
- `AT_FDCWD` for relative paths: -100
- `AT_SYMLINK_NOFOLLOW` flag: 0x100 (to not follow symlinks)
- Times use `struct timespec { time_t tv_sec; long tv_nsec; }`

## Help and Version Output

### `touch --help`

```
Usage: touch [OPTION]... FILE...
Update the access and modification times of each FILE to the current time.

A FILE argument that does not exist is created empty, unless -c or -h
is supplied.

  -a                     change only the access time
  -c, --no-create        do not create any files
  -d, --date=STRING      parse STRING and use it instead of current time
  -m                     change only the modification time
  -r, --reference=FILE   use this file's times instead of current time
  -t STAMP               use [[CC]YY]MMDDhhmm[.ss] instead of current time
      --help             display this help and exit
      --version          output version information and exit
```

### `touch --version`

```
touch (LevitateOS levbox) 0.1.0
```
