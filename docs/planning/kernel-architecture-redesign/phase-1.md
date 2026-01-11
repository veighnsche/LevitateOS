# Phase 1: Discovery & Safeguards

## TEAM_422: Kernel Architecture Redesign

### Refactor Summary

**What**: Restructure the kernel from a monolithic binary with supporting crates into a modular workspace where each subsystem (memory, scheduler, VFS, syscalls) is its own crate with clear boundaries.

**Why**:
- Current structure has grown organically, leading to tight coupling
- Large files (700+ lines in `memory/user.rs`, 600+ in `syscall/fs/fd.rs`)
- Task/Process/Thread concerns are intermingled
- HAL mixes hardware abstraction with driver code
- Difficult to test subsystems in isolation
- No clear ownership boundaries between modules

**Pain Points**:
1. `src/task/mod.rs` (385 lines) mixes task struct, scheduling, and lifecycle
2. `src/memory/user.rs` (709 lines) is doing too much
3. Architecture code (`arch/{aarch64,x86_64}/mod.rs` ~580 lines each) has platform code mixed with abstractions
4. Syscall dispatcher (`syscall/mod.rs`, 519 lines) is a giant match statement
5. `init.rs` (570 lines) is a god function with sequential initialization

### Success Criteria

| Before | After |
|--------|-------|
| Single kernel binary crate | Workspace with focused crates |
| Files > 500 lines common | All files < 500 lines (< 300 ideal) |
| Cross-module dependencies via `crate::` | Explicit crate dependencies |
| Task struct owns everything | Process/Thread separation |
| Arch code mixed with platform specifics | Clean `Platform` trait abstraction |
| No subsystem tests | Each crate testable in isolation |

### Behavioral Contracts (Must Not Change)

1. **Linux Syscall ABI**
   - `SyscallResult = Result<i64, u32>`
   - Single conversion point: `Err(e) => -(e as i64)`
   - Syscall numbers match linux-raw-sys

2. **Memory Layout**
   - Higher-half kernel at `0xFFFF_8000_0000_0000`
   - User space below `0x0000_8000_0000_0000`
   - `Stat` struct exactly 128 bytes on AArch64

3. **Process Model**
   - fork() creates new process with copied memory
   - clone() creates thread sharing address space
   - exec() replaces process image
   - wait() reaps zombie processes

4. **VFS Semantics**
   - File descriptors are per-process
   - Open files have independent seek positions
   - Pipes are blocking with 4KB buffer

### Golden/Regression Tests to Lock In

**Build Tests** (must pass throughout refactor):
```bash
cargo build --target x86_64-unknown-none --release
cargo build --target aarch64-unknown-none --release
```

**Behavior Tests** (from main repo):
```bash
cargo xtask test behavior  # Boot log comparison
cargo xtask test unit      # Unit tests
```

**Key Behaviors to Preserve**:
- [ ] Boot to shell prompt on both architectures
- [ ] `echo hello` prints "hello"
- [ ] `cat /proc/self/maps` shows memory regions
- [ ] Basic fork/exec works
- [ ] File read/write on tmpfs works

### Current Architecture Notes

**Dependency Graph**:
```
kernel (binary)
├── los_hal (hardware abstraction)
│   ├── los_utils (spinlock, ringbuffer, cpio)
│   └── los_error (error types)
├── los_term (terminal emulator)
├── los_pci (PCI bus)
└── los_gpu (VirtIO GPU)
```

**Internal Module Structure** (src/):
```
├── arch/           # Platform-specific (boot, exceptions, syscall ABI)
├── boot/           # Boot protocol handling
├── fs/             # Filesystem (vfs/, tmpfs/, tty/)
├── loader/         # ELF loader
├── memory/         # Memory management
├── syscall/        # Syscall implementations
└── task/           # Task/Process/Thread management
```

**Coupling Issues**:
1. `task/mod.rs` imports from `memory`, `fs`, `syscall`
2. `memory/user.rs` imports from `task` (circular concern)
3. `syscall/*` imports directly from `task` internals
4. `init.rs` knows about everything

### Constraints

1. **No Breaking Changes to ABI**: External programs must continue to work
2. **Incremental Migration**: Must be able to build at each step
3. **No Shims**: Clean breaks, fix all call sites
4. **Preserve History**: Document breaking changes in team files
5. **Test Coverage**: Each phase must pass existing tests
6. **File Size Limits**: Target < 500 lines per file, max 1000

### Current File Size Analysis

**Files Exceeding 500 Lines** (immediate targets):
| File | Lines | Issue |
|------|-------|-------|
| `memory/user.rs` | 709 | User memory management sprawl |
| `syscall/fs/fd.rs` | 638 | FD operations + dup logic |
| `loader/elf.rs` | 579 | ELF loading + relocation |
| `init.rs` | 570 | Boot sequence god function |
| `arch/x86_64/mod.rs` | 592 | Platform code mixed in |
| `arch/aarch64/mod.rs` | 557 | Platform code mixed in |
| `fs/path.rs` | 528 | Path resolution complexity |
| `syscall/epoll.rs` | 524 | Event polling implementation |
| `syscall/mod.rs` | 519 | Dispatcher sprawl |
| `syscall/helpers.rs` | 505 | Utility functions grab bag |

### Risk Assessment

| Risk | Mitigation |
|------|------------|
| Breaking existing functionality | Lock in golden tests before starting |
| Circular dependencies in new structure | Design dependency graph first |
| Merge conflicts with ongoing work | Coordinate with other teams |
| Performance regression | Benchmark boot time before/after |
| Incomplete migration | Each phase must be atomic and complete |
