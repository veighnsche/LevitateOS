# TEAM_424: Syscall Conformance Testing Strategy

## Objective

Ensure LevitateOS syscalls behave identically to Linux, enabling unmodified Linux programs to run correctly.

## Status: Phase 1 Complete

### What Was Created

#### syscall-conformance Crate

Location: `crates/userspace/eyra/syscall-conformance/`

A userspace test binary that runs on LevitateOS and verifies syscall behavior against Linux expectations.

**Structure:**
```
syscall-conformance/
├── Cargo.toml
├── build.rs
└── src/
    ├── main.rs      # Test framework, macros, runner
    ├── fs/mod.rs    # Filesystem syscall tests (10 tests)
    ├── mem/mod.rs   # Memory syscall tests (6 tests)
    ├── process/mod.rs # Process syscall tests (6 tests)
    └── sync/mod.rs  # Synchronization tests (3 tests)
```

**Total: 25 syscall conformance tests**

### Test Categories

#### Filesystem Tests (fs/mod.rs)
- `write_stdout` - write() to stdout
- `read_stdin_setup` - fstat on stdin
- `openat_nonexistent` - openat() returns ENOENT
- `close_invalid_fd` - close() returns EBADF
- `close_valid_fd` - close() and double-close
- `fstat_stdout` - fstat() returns char device mode
- `lseek_pipe_espipe` - lseek() returns ESPIPE
- `dup_stdout` - dup() creates valid fd
- `dup2_redirect` - dup2() behavior
- `writev_basic` - writev() vectored write

#### Memory Tests (mem/mod.rs)
- `mmap_anonymous` - MAP_ANONYMOUS | MAP_PRIVATE
- `mmap_write_without_read` - edge case handling
- `munmap_basic` - munmap() cleanup
- `mprotect_basic` - mprotect() permission change
- `brk_query` - brk(0) returns current break
- `brk_expand` - brk() heap expansion

#### Process Tests (process/mod.rs)
- `getpid` - getpid() consistency
- `getppid` - getppid() validity
- `gettid` - gettid() equals getpid() for single-threaded
- `clock_gettime_monotonic` - monotonic time
- `clock_gettime_realtime` - realtime clock
- `getrandom` - getrandom() returns random bytes

#### Synchronization Tests (sync/mod.rs)
- `pipe_basic` - pipe2() creates valid fds
- `pipe2_cloexec` - O_CLOEXEC flag
- `pipe_readwrite` - pipe data transfer

### Output Format

Tests produce structured output for machine parsing:
```
TEST:<category>:<name>:PASS
TEST:<category>:<name>:FAIL:<reason>
```

### Building

```bash
# x86_64
cargo build --target x86_64-unknown-linux-gnu -p syscall-conformance

# aarch64
cargo build --target aarch64-unknown-linux-gnu -p syscall-conformance
```

### Key APIs Used

The tests use `libsyscall::arch::{syscall0, syscall1, ...}` for raw syscall invocation, enabling direct kernel behavior testing without libc abstraction.

## Next Steps (Phases 2-6)

### Phase 2: xtask Integration
- Add `cargo xtask test syscall` command
- Include test binary in initramfs
- Parse and report results

### Phase 3: Memory Management Testing
- Expand MMU unit tests
- Add mmap edge cases
- Test brk/sbrk semantics

### Phase 4: Architecture Parity Testing
- Run same tests on x86_64 and aarch64
- Diff results to find inconsistencies
- Verify ABI structure sizes

### Phase 5: Fill Testing Gaps
- Driver unit tests
- VFS/filesystem tests
- TTY/PTY behavior

### Phase 6: CI/CD Enhancement
- Matrix builds for both architectures
- Test timing tracking
- Coverage reporting

## References

- Plan file: `docs/planning/synchronous-doodling-dongarra.md`
- Linux syscall ABI: `docs/specs/LINUX_ABI_GUIDE.md`
- libsyscall: `crates/userspace/eyra/libsyscall/`
