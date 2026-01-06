# Utility Specification: `ls`

The `ls` utility lists directory contents.

## Specification References

| Standard | Link |
|----------|------|
| **POSIX.1-2017** | [ls - list directory contents](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/ls.html) |
| **Linux man-pages** | [ls(1)](https://man7.org/linux/man-pages/man1/ls.1.html) |
| **GNU Coreutils** | [ls invocation](https://www.gnu.org/software/levbox/manual/html_node/ls-invocation.html) |

## Synopsis

```bash
ls [-aAlhFR1] [file...]
```

## Options

| Option | Description |
|--------|-------------|
| `-a`, `--all` | Include directory entries whose names begin with a dot (`.`). |
| `-A`, `--almost-all` | Like `-a`, but do not list implied `.` and `..`. |
| `-l` | Use a long listing format. |
| `-h`, `--human-readable` | With `-l`, print sizes in human readable format (e.g., 1K, 234M, 2G). |
| `-F`, `--classify` | Append indicator (one of `*/=>@\|`) to entries. |
| `-R`, `--recursive` | List subdirectories recursively. |
| `-1` | List one file per line. |
| `--help` | Display usage help and exit. |
| `--version` | Output version information and exit. |

## Operands

- **file**: A pathname of a file to be written. If no operands are specified, the current directory shall be used.

## Description

For each operand that names a file of a type other than directory, `ls` shall write the name of the file as well as any information requested.

For each operand that names a file of type directory, `ls` shall write the names of files contained within the directory as well as any requested associated information.

### Default Behavior

1. **Sorting**: Output is sorted alphabetically by default.
2. **Hidden Files**: Files beginning with `.` are excluded unless `-a` or `-A` is specified.
3. **Output Format**: Names are written as a sequence separated by newlines (multi-column output is an extension).

## Long Output Format (`-l`)

The long format shall include the following fields, separated by spaces:

```
drwxr-xr-x  2 user group   4096 Jan  6 15:30 dirname
-rw-r--r--  1 user group    123 Jan  6 15:30 filename
lrwxrwxrwx  1 user group     11 Jan  6 15:30 linkname -> target
```

| Field | Description |
|-------|-------------|
| **File Mode** | First character indicates type (`d`=dir, `-`=file, `l`=symlink), followed by 9 permission characters. |
| **Link Count** | Number of hard links to the file. |
| **Owner** | Owner name (or UID if name unavailable). |
| **Group** | Group name (or GID if name unavailable). |
| **Size** | File size in bytes (or device numbers for special files). |
| **Timestamp** | Modification time in `MMM DD HH:MM` or `MMM DD  YYYY` format. |
| **Name** | Filename, with symlink target shown as `name -> target`. |

### File Type Indicators

| Character | Type |
|-----------|------|
| `d` | Directory |
| `-` | Regular file |
| `l` | Symbolic link |
| `c` | Character special file |
| `b` | Block special file |
| `p` | FIFO (named pipe) |
| `s` | Socket |

### Permission Characters

For each of owner, group, others (3 sets of 3 characters):

| Position | Character | Meaning |
|----------|-----------|---------|
| 1, 4, 7 | `r` / `-` | Read permission |
| 2, 5, 8 | `w` / `-` | Write permission |
| 3, 6, 9 | `x` / `-` | Execute permission |

Special bits:
- `s` in owner execute position: setuid
- `s` in group execute position: setgid
- `t` in others execute position: sticky bit

## Color Output (Extension)

While not POSIX, colorized output based on file type is widely expected:

| File Type | Color | ANSI Code |
|-----------|-------|-----------|
| Directory | Blue | `\x1b[34m` |
| Executable | Green | `\x1b[32m` |
| Symbolic Link | Cyan | `\x1b[36m` |
| Regular File | Default | `\x1b[0m` |
| Broken Symlink | Red | `\x1b[31m` |

Coloring should be enabled when stdout is a TTY, or explicitly via `--color=always`.

## Exit Status

| Value | Condition |
|-------|-----------|
| `0` | Successful completion. |
| `1` | A minor problem (e.g., cannot access a file). |
| `2` | A serious problem (e.g., cannot access command-line argument). |

## Errors

| Error | Condition |
|-------|-----------|
| `ENOENT` | File or directory does not exist. |
| `EACCES` | Permission denied. |
| `ENOTDIR` | A component of the path is not a directory. |

## Examples

```bash
# List current directory
ls

# List with hidden files
ls -a

# Long format with human-readable sizes
ls -lh

# Long format of specific directory
ls -l /home/user

# Recursive listing with indicators
ls -RF /etc

# One file per line
ls -1
```

## Implementation Notes for LevitateOS

### Syscalls Required

| Syscall | Purpose |
|---------|---------|
| `sys_open` / `sys_openat` | Open directory for reading |
| `sys_getdents` / `sys_getdents64` | Read directory entries |
| `sys_fstat` / `sys_lstat` | Get file metadata |
| `sys_readlink` | Read symbolic link target |
| `sys_close` | Close file descriptor |
| `sys_isatty` | Check if stdout is a TTY (for color) |

### Implementation Strategy

1. **Directory Reading**: Use `getdents64` syscall to read `linux_dirent64` structures.
2. **Stat Information**: For `-l`, call `lstat` on each entry (not `stat`, to not follow symlinks).
3. **Sorting**: Collect entries into a Vec, sort alphabetically, then output.
4. **Column Width**: For `-l`, pre-calculate maximum widths for alignment.
5. **Symlink Handling**: Use `readlink` to get target for display.

### ABI Compatibility

- Directory entries use `linux_dirent64` structure with `d_ino`, `d_off`, `d_reclen`, `d_type`, and `d_name`.
- Stat structure must match Linux `struct stat` layout.

## Help and Version Output

### `ls --help`

```
Usage: ls [OPTION]... [FILE]...
List information about the FILEs (the current directory by default).

  -a, --all            do not ignore entries starting with .
  -A, --almost-all     do not list implied . and ..
  -F, --classify       append indicator (one of */=>@|) to entries
  -h, --human-readable with -l, print sizes like 1K 234M 2G
  -l                   use a long listing format
  -R, --recursive      list subdirectories recursively
  -1                   list one file per line
      --help           display this help and exit
      --version        output version information and exit

Exit status:
 0  if OK,
 1  if minor problems,
 2  if serious trouble.
```

### `ls --version`

```
ls (LevitateOS levbox) 0.1.0
```
