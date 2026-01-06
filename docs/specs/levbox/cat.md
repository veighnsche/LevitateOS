# Utility Specification: `cat`

The `cat` utility concatenates and prints files to standard output.

## Specification References

| Standard | Link |
|----------|------|
| **POSIX.1-2017** | [cat - concatenate and print files](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cat.html) |
| **Linux man-pages** | [cat(1)](https://man7.org/linux/man-pages/man1/cat.1.html) |
| **GNU Coreutils** | [cat invocation](https://www.gnu.org/software/levbox/manual/html_node/cat-invocation.html) |

## Synopsis

```bash
cat [-u] [file...]
```

## Options

| Option | Description |
|--------|-------------|
| `-u` | Write bytes from the input file to the standard output without delay as each is read. (Unbuffered mode) |
| `--help` | Display usage help and exit. |
| `--version` | Output version information and exit. |

## Operands

- **file**: A pathname of an input file. If no file operands are specified, or if a file operand is `-`, the standard input shall be used.

## Description

The `cat` utility shall read files in sequence and shall write their contents to the standard output in the same sequence.

### Behavior

1. If no files are specified, `cat` reads from standard input.
2. If a file operand is `-`, `cat` reads from standard input at that point in the sequence.
3. The utility does not close and reopen standard input when it is referenced in this way, but accepts multiple occurrences of `-` as a file operand.

## Exit Status

| Value | Condition |
|-------|-----------|
| `0` | All input files were output successfully. |
| `>0` | An error occurred. |

## Errors

The following errors shall be detected:

| Error | Condition |
|-------|-----------|
| `ENOENT` | The specified file does not exist. |
| `EACCES` | Permission denied to read the file. |
| `EISDIR` | The specified path is a directory. |
| `EMFILE` | Too many open file descriptors. |

## Examples

```bash
# Print a file to stdout
cat file.txt

# Concatenate multiple files
cat file1.txt file2.txt file3.txt

# Read from stdin (type content, then Ctrl+D)
cat

# Read from stdin and a file
cat - file.txt

# Concatenate stdin between two files
cat header.txt - footer.txt
```

## Implementation Notes for LevitateOS

### Syscalls Required

| Syscall | Purpose |
|---------|---------|
| `sys_open` | Open file for reading |
| `sys_read` | Read file contents |
| `sys_write` | Write to stdout (fd 1) |
| `sys_close` | Close file descriptor |

### Implementation Strategy

1. **Buffer Size**: Use a 4096-byte buffer for balanced read/write operations.
2. **EOF Handling**: Continue reading until `sys_read` returns 0.
3. **Error Propagation**: On any read/write error, print diagnostic to stderr and continue with next file.
4. **stdin Detection**: Check if operand is `-` or no operands provided.

### ABI Compatibility

- File descriptor 0 is stdin, 1 is stdout, 2 is stderr.
- Binary-safe: must handle NUL bytes in files correctly.

## Help and Version Output

### `cat --help`

```
Usage: cat [OPTION]... [FILE]...
Concatenate FILE(s) to standard output.

With no FILE, or when FILE is -, read standard input.

  -u                  (ignored)
      --help          display this help and exit
      --version       output version information and exit

Examples:
  cat f - g  Output f's contents, then standard input, then g's contents.
  cat        Copy standard input to standard output.
```

### `cat --version`

```
cat (LevitateOS levbox) 0.1.0
```
