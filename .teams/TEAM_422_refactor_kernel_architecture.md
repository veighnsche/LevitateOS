# TEAM_422: Kernel Architecture Redesign

## Status: Planning

## Objective

Restructure the kernel from a monolithic binary with supporting crates into a modular workspace where each subsystem (memory, scheduler, VFS, syscalls) is its own crate with clear boundaries.

## Problem Statement

The current kernel structure has grown organically, leading to:
- Tight coupling between subsystems
- Large files (700+ lines in `memory/user.rs`, 600+ in `syscall/fs/fd.rs`)
- Task/Process/Thread concerns intermingled
- HAL mixing hardware abstraction with driver code
- Difficult to test subsystems in isolation
- No clear ownership boundaries between modules

## Target Architecture

```
kernel/                       # Thin kernel binary
├── arch/aarch64/            # Platform-specific crate
├── arch/x86_64/             # Platform-specific crate
├── mm/                      # Memory management
├── sched/                   # Process/thread scheduler
├── vfs/                     # Virtual filesystem
├── syscall/                 # Syscall dispatch
├── drivers/                 # Device drivers
├── hal/                     # Hardware abstraction (slimmed)
├── utils/                   # Core utilities
└── error/                   # Error types
```

## Success Criteria

| Before | After |
|--------|-------|
| Single kernel binary crate | Workspace with focused crates |
| Files > 500 lines common | All files < 500 lines |
| Cross-module deps via `crate::` | Explicit crate dependencies |
| Task struct owns everything | Process/Thread separation |
| No subsystem tests | Each crate testable in isolation |

## Plan Location

Detailed implementation plan: `docs/planning/kernel-architecture-redesign/`

- **Phase 1**: Discovery & Safeguards - Lock in golden tests, document contracts
- **Phase 2**: Structural Extraction - Design new crate layout
- **Phase 3**: Migration - Step-by-step code movement
- **Phase 4**: Cleanup - Dead code removal, documentation sync
- **Phase 5**: Hardening - Test coverage, static analysis, performance verification

## Behavioral Contracts (Must Not Change)

1. **Linux Syscall ABI**: `SyscallResult = Result<i64, u32>`, syscall numbers match linux-raw-sys
2. **Memory Layout**: Higher-half kernel at `0xFFFF_8000_0000_0000`, `Stat` exactly 128 bytes
3. **Process Model**: fork/clone/exec/wait semantics preserved
4. **VFS Semantics**: FDs per-process, independent seek positions

## Migration Order

1. Extract `mm/` crate (memory management)
2. Extract `sched/` crate (scheduler)
3. Extract `vfs/` crate (virtual filesystem)
4. Extract `syscall/` crate
5. Extract `arch/` crates
6. Slim down `kernel/` binary

Each step must leave the kernel in a buildable, testable state.

## Rollback Strategy

- Each migration step is one commit
- Branch per subsystem: `refactor/mm`, `refactor/sched`, etc.
- Merge only when all tests pass on both architectures
- Abort if build broken > 4 hours

## Files Exceeding 500 Lines (Immediate Targets)

| File | Lines | Issue |
|------|-------|-------|
| `memory/user.rs` | 709 | User memory management sprawl |
| `syscall/fs/fd.rs` | 638 | FD operations + dup logic |
| `loader/elf.rs` | 579 | ELF loading + relocation |
| `init.rs` | 570 | Boot sequence god function |
| `arch/x86_64/mod.rs` | 592 | Platform code mixed in |
| `arch/aarch64/mod.rs` | 557 | Platform code mixed in |
| `fs/path.rs` | 528 | Path resolution complexity |

## Related Teams

- TEAM_406: General Purpose OS (depends on stable kernel API)
- TEAM_407: Refactor/Consolidate Scattered Code (overlapping goals)

## Log

### 2026-01-11: Plan Created
- Created comprehensive 5-phase refactor plan
- Documented current architecture issues
- Defined target workspace structure
- Established migration order and rollback strategy
