# Phase 2: Design — `cat` Utility

**TEAM_182** | 2026-01-06

## Proposed Solution

### High-Level Description

Create a `levbox` crate in the userspace workspace containing a `cat` binary. The binary will follow the POSIX `cat` specification, reading files sequentially and writing their contents to stdout.

### User-Facing Behavior

```
cat [-u] [file...]
```

| Input | Output |
|-------|--------|
| `cat foo.txt` | Contents of foo.txt to stdout |
| `cat a.txt b.txt` | a.txt then b.txt concatenated |
| `cat` | Echo stdin to stdout until EOF |
| `cat -` | Same as above (stdin placeholder) |
| `cat a.txt - b.txt` | a.txt, then stdin, then b.txt |

### System Behavior

1. Parse command-line arguments
2. For each file operand (or stdin if none/dash):
   a. Open file (skip if stdin)
   b. Read in 4KB chunks
   c. Write each chunk to stdout
   d. On error: print message to stderr, set exit code, continue
3. Exit with 0 if all files processed successfully, otherwise >0

## Crate Structure

```
userspace/
├── levbox/
│   ├── Cargo.toml
│   ├── link.ld          # Linker script (copy from shell)
│   ├── build.rs         # Pass linker script to cargo
│   └── src/
│       └── bin/
│           └── cat.rs   # cat binary
```

We use `src/bin/` to create a multi-binary crate that can host all levbox (ls, pwd, etc.) in the future.

## API Design

### Argument Parsing (Simple)

```rust
struct Args {
    unbuffered: bool,     // -u flag (currently no-op)
    files: Vec<&'a str>,  // File paths or "-" for stdin
}

fn parse_args(argc: usize, argv: *const *const u8) -> Args;
```

### Core Functions

```rust
/// Output a single file to stdout. Returns success/failure.
fn cat_file(path: &str) -> bool;

/// Output stdin to stdout until EOF. Returns success/failure.
fn cat_stdin() -> bool;

/// Write an error message to stderr.
fn eprint_error(file: &str, msg: &str);
```

## Behavioral Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Buffer size | 4096 bytes | Matches spec recommendation, good for disk I/O |
| Unbuffered mode | No-op | Our implementation is already unbuffered |
| Error handling | Continue | POSIX: process all files, report errors |
| Exit code | First error | Return first non-zero error code encountered |
| stdin handling | Check for "-" or no args | Per POSIX spec |

## Design Alternatives Considered

### 1. Use `ulib::fs::File` vs Raw Syscalls

**Chose: Raw syscalls (`libsyscall::read/write/openat/close`)**

Rationale: 
- `cat` is a simple utility that doesn't need OOP abstractions
- Keeps binary size smaller
- Direct control over buffering behavior
- `ulib::fs::File` is RAII which is nice, but we can manage resources simply

### 2. Multicall Binary vs Separate Binaries

**Chose: Separate binaries (for now)**

Rationale:
- Simpler to start with
- Can add multicall wrapper later
- No shell complexity for command routing

### 3. Argument Parsing Library vs Manual

**Chose: Manual parsing**

Rationale:
- Only one option (`-u`)
- No external dependencies needed
- Keeps binary small

## Open Questions

**None** — The design is straightforward and all syscalls are available.

## Verification Plan

### Automated Tests

1. **Behavior test:** Add cat-specific behavior ID `[CAT1]` etc.
2. **Test file in initramfs:** Include `/test_files/cat_test.txt`
3. **Shell integration test:** `cat /test_files/cat_test.txt` should print content

### Manual Verification

1. Run `cargo xtask run default`
2. In shell: `cat /hello.txt` (or similar test file)
3. Verify output matches file contents
