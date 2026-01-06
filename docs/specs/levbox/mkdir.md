# Utility Specification: `mkdir`

The `mkdir` utility creates directories.

## Specification References

| Standard | Link |
|----------|------|
| **POSIX.1-2017** | [mkdir - make directories](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/mkdir.html) |
| **Linux man-pages** | [mkdir(1)](https://man7.org/linux/man-pages/man1/mkdir.1.html) |
| **GNU Coreutils** | [mkdir invocation](https://www.gnu.org/software/levbox/manual/html_node/mkdir-invocation.html) |

## Synopsis

```bash
mkdir [-pvm mode] directory...
```

## Options

| Option | Description |
|--------|-------------|
| `-p`, `--parents` | Create intermediate directories as required. Do not error if directory exists. |
| `-m`, `--mode=MODE` | Set file mode (permissions) for created directories. |
| `-v`, `--verbose` | Print a message for each created directory. |
| `--help` | Display usage help and exit. |
| `--version` | Output version information and exit. |

## Operands

- **directory**: A pathname of a directory to create.

## Description

The `mkdir` utility creates the directories specified by the operands, in the order specified.

### Default Behavior

- Created directories have permission `(0777 & ~umask)` unless `-m` is specified.
- Parent directories must exist unless `-p` is specified.
- It is an error if the directory already exists (unless `-p`).

### Parent Creation (`-p`)

When `-p` is specified:

1. All missing intermediate directories are created.
2. No error occurs if directory already exists.
3. Each created directory gets the default permissions.

Example: `mkdir -p /a/b/c/d` creates `/a`, `/a/b`, `/a/b/c`, and `/a/b/c/d` as needed.

### Mode Specification (`-m`)

The mode argument can be:

| Format | Example | Meaning |
|--------|---------|---------|
| Octal | `755` | rwxr-xr-x |
| Symbolic | `u=rwx,go=rx` | Same as 755 |
| Symbolic delta | `+x` | Add execute to default |

## Exit Status

| Value | Condition |
|-------|-----------|
| `0` | All directories were created successfully. |
| `>0` | An error occurred. |

## Errors

| Error | Condition |
|-------|-----------|
| `EEXIST` | Directory already exists (error without `-p`). |
| `ENOENT` | Parent directory does not exist (error without `-p`). |
| `ENOTDIR` | A component of the path is not a directory. |
| `EACCES` | Permission denied to create in parent directory. |
| `ENOSPC` | No space left on device. |
| `EROFS` | Read-only filesystem. |
| `ENAMETOOLONG` | Pathname too long. |

## Examples

```bash
# Create a single directory
mkdir newdir

# Create multiple directories
mkdir dir1 dir2 dir3

# Create nested directories
mkdir -p /path/to/deep/nested/directory

# Create with specific permissions
mkdir -m 700 private_dir

# Create with verbose output
mkdir -v -p project/src/main
# Output:
# mkdir: created directory 'project'
# mkdir: created directory 'project/src'
# mkdir: created directory 'project/src/main'
```

## Implementation Notes for LevitateOS

### Syscalls Required

| Syscall | Purpose |
|---------|---------|
| `sys_mkdir` / `sys_mkdirat` | Create directory |
| `sys_stat` / `sys_access` | Check if directory exists (for `-p`) |
| `sys_umask` | Get current umask (optional) |

### Implementation Strategy

1. **Simple Case**: Without `-p`, just call `mkdir(path, mode)`.
2. **Path Parsing for -p**: Split path by `/`, create each missing component.
3. **Existence Check**: For `-p`, check if each component exists before creating.
4. **Mode Handling**: Parse symbolic or octal mode, apply umask if not specified.
5. **Verbose Output**: Print to stdout as directories are created.

### Mkdir Variants

| Syscall | Use |
|---------|-----|
| `mkdir(path, mode)` | Create directory at absolute/relative path |
| `mkdirat(dirfd, path, mode)` | Create relative to directory fd |

`mkdirat` with `AT_FDCWD` is equivalent to `mkdir`.

### Path Handling for `-p`

```
Input: /a/b/c/d
Steps:
1. Try mkdir("/a") - may exist (OK with -p)
2. Try mkdir("/a/b") - may exist
3. Try mkdir("/a/b/c") - may exist
4. Try mkdir("/a/b/c/d") - create (or exist OK)
```

Ignore `EEXIST` errors when `-p` is specified.

### ABI Compatibility

- `mkdir` syscall number: 83 (x86_64), 1030 (aarch64 via mkdirat)
- `mkdirat` syscall number: 258 (x86_64), 34 (aarch64)
- Default mode: `0777` (before umask)

## Help and Version Output

### `mkdir --help`

```
Usage: mkdir [OPTION]... DIRECTORY...
Create the DIRECTORY(ies), if they do not already exist.

  -m, --mode=MODE   set file mode (as in chmod), not a=rwx - umask
  -p, --parents     no error if existing, make parent directories as needed
  -v, --verbose     print a message for each created directory
      --help        display this help and exit
      --version     output version information and exit
```

### `mkdir --version`

```
mkdir (LevitateOS levbox) 0.1.0
```
