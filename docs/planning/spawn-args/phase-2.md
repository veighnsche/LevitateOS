# Phase 2: Design — Spawn Argument Passing

**TEAM_186** | 2026-01-06

## Proposed Solution

Add a new syscall `sys_spawn_args` that accepts a path plus an argv array. The kernel will parse the user-space argv array, copy strings to kernel space, and pass them to the existing `spawn_from_elf_with_args`.

### High-Level Flow

```
Shell: "cat /hello.txt"
   │
   ├─► Parse command line into ["cat", "/hello.txt"]
   │
   ├─► Build argv array in userspace memory:
   │   argv[0] → "cat\0"
   │   argv[1] → "/hello.txt\0"
   │   argv[2] → NULL
   │
   └─► spawn_args("/cat", argv_ptr, argc=2)
          │
          ▼
Kernel: sys_spawn_args(path_ptr, path_len, argv_ptr, argc)
   │
   ├─► Validate path buffer
   ├─► Validate argv array (argc pointers)
   ├─► For each argv[i]: read string from userspace
   │
   └─► spawn_from_elf_with_args(elf_data, args_vec, [])
          │
          ▼
       setup_stack_args() writes to user stack:
          argc = 2
          argv[0] → "cat"
          argv[1] → "/hello.txt"
          NULL
```

## Syscall API Design

### New Syscall: `SYS_SPAWN_ARGS` (Number 15)

```rust
/// Spawn a process with command-line arguments.
///
/// # Arguments
/// - x0: path_ptr     - Pointer to executable path string
/// - x1: path_len     - Length of path string
/// - x2: argv_ptr     - Pointer to array of (ptr, len) pairs
/// - x3: argc         - Number of arguments
///
/// # Argv Format
/// The argv_ptr points to an array of `ArgvEntry` structs:
/// ```c
/// struct ArgvEntry {
///     const char* ptr;  // 8 bytes
///     size_t len;       // 8 bytes
/// };
/// ```
///
/// # Returns
/// PID of spawned process on success, negative errno on failure.
pub fn sys_spawn_args(
    path_ptr: usize,
    path_len: usize,
    argv_ptr: usize,
    argc: usize,
) -> i64;
```

### Why This Design?

| Alternative | Pros | Cons |
|-------------|------|------|
| **Chosen: Explicit ptr+len pairs** | No null scanning, safe lengths | Slightly more complex userspace struct |
| Linux-style null-terminated | Compatible | Requires scanning for nulls, strlen() |
| Single packed buffer | Simple | Hard to parse, alignment issues |

### Userspace Wrapper

```rust
/// TEAM_186: Argument entry for spawn_args syscall.
#[repr(C)]
pub struct ArgvEntry {
    pub ptr: *const u8,
    pub len: usize,
}

/// TEAM_186: Spawn a process with arguments.
pub fn spawn_args(path: &str, argv: &[&str]) -> isize {
    // Build ArgvEntry array on stack
    let mut entries = [ArgvEntry { ptr: core::ptr::null(), len: 0 }; 16];
    let argc = argv.len().min(16);
    for (i, arg) in argv.iter().take(argc).enumerate() {
        entries[i] = ArgvEntry {
            ptr: arg.as_ptr(),
            len: arg.len(),
        };
    }
    
    // Syscall
    unsafe {
        core::arch::asm!(
            "svc #0",
            in("x8") SYS_SPAWN_ARGS,
            in("x0") path.as_ptr(),
            in("x1") path.len(),
            in("x2") entries.as_ptr(),
            in("x3") argc,
            lateout("x0") ret,
            options(nostack)
        );
    }
    ret as isize
}
```

### Shell Command Parsing

```rust
/// Parse a command line into path and arguments
fn parse_command(line: &[u8]) -> Option<(&str, Vec<&str>)> {
    let parts: Vec<&[u8]> = split_whitespace(line);
    if parts.is_empty() {
        return None;
    }
    
    let path = core::str::from_utf8(parts[0]).ok()?;
    let args: Vec<&str> = parts.iter()
        .filter_map(|p| core::str::from_utf8(p).ok())
        .collect();
    
    Some((path, args))
}
```

## Behavioral Decisions

| Behavior | Decision | Rationale |
|----------|----------|-----------|
| Max argc | 16 | Simple fixed limit, expandable later |
| Max arg length | 256 bytes each | Reasonable for CLI use |
| argv[0] | Program name (from shell) | Matches POSIX convention |
| Empty argv | Allowed (argc=0) | Kernel will set argc=0 on stack |
| Invalid argv pointer | Return EFAULT | Standard errno |
| Path not found | Return ENOENT | Standard errno |

## Error Handling

| Condition | Error Code |
|-----------|------------|
| path_ptr invalid | `EFAULT` (-3) |
| argv_ptr invalid | `EFAULT` (-3) |
| argc > 16 | `EINVAL` (-4) |
| Any argv[i].ptr invalid | `EFAULT` (-3) |
| Path not UTF-8 | `EINVAL` (-4) |
| Executable not found | `ENOENT` (-5) |
| ELF load failure | `-1` |

## Data Model Changes

### New Constants

```rust
// kernel/src/syscall/mod.rs
pub const SYS_SPAWN_ARGS: u64 = 15;

// Limits
const MAX_ARGC: usize = 16;
const MAX_ARG_LEN: usize = 256;
```

### New Types (Kernel)

```rust
/// TEAM_186: User-space argv entry (matches libsyscall::ArgvEntry)
#[repr(C)]
struct UserArgvEntry {
    ptr: usize,  // User pointer to string
    len: usize,  // String length
}
```

## Open Questions

**None** — The design is straightforward given existing infrastructure.

## Verification Plan

### Unit Tests (Kernel)

1. Test `setup_stack_args` with various argc values
2. Test argv pointer validation

### Integration Tests

1. Boot system, run `cat /hello.txt`, verify output
2. Run `cat` without args, verify stdin mode
3. Test error cases (nonexistent file)

### Manual Verification

```bash
# In shell:
cat /hello.txt       # Should print file contents
cat                  # Should wait for stdin
cat /nonexistent     # Should print error
```
