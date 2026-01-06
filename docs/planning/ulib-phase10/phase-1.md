# Phase 10: Userspace Standard Library (`ulib`) — Phase 1: Discovery

**Team:** TEAM_164  
**Created:** 2026-01-06  
**Status:** Draft

---

## 1. Feature Summary

### 1.1 Short Description
Create `ulib`, a `std`-like library for LevitateOS userspace that provides essential abstractions for heap allocation, file I/O, environment access, time, and error handling.

### 1.2 Problem Statement
Currently, userspace programs in LevitateOS must use raw syscall wrappers from `libsyscall`. This works for simple programs like the shell, but prevents development of more sophisticated applications. Programs cannot:
- Dynamically allocate memory (heap)
- Open or manipulate files
- Read command-line arguments
- Use time/sleep functionality
- Use standard Rust idioms (`Result`, `Box`, `Vec`, `String`)

### 1.3 Who Benefits
- **Application developers**: Can write idiomatic Rust with familiar abstractions
- **Future levbox**: Phase 11 tools (`ls`, `cat`, etc.) require file I/O
- **Shell enhancement**: Can support redirection, pipes, and scripting
- **OS maturity**: Moves LevitateOS toward POSIX-like usability

---

## 2. Success Criteria

### 2.1 Definition of Done
The feature is complete when:
1. Userspace programs can use `#[global_allocator]` backed by `sbrk`
2. `Box`, `Vec`, `String` work in userspace
3. Basic file operations are possible (`open`, `read`, `write`, `close`)
4. Programs can access their arguments and environment
5. All existing tests pass
6. At least one new userspace program demonstrates the new capabilities

### 2.2 Acceptance Criteria
- [ ] `ulib` crate exists in `userspace/ulib/`
- [ ] Global allocator works (heap allocation via `sbrk`)
- [ ] `File` abstraction wraps file syscalls
- [ ] `args()` returns command-line arguments
- [ ] Shell or demo program uses new library
- [ ] Zero regressions in existing functionality

---

## 3. Current State Analysis

### 3.1 How the System Works Today

**Userspace structure:**
```
userspace/
├── Cargo.toml           (workspace)
├── libsyscall/          (raw syscall wrappers)
├── init/                (PID 1 - spawns shell)
├── shell/               (interactive shell)
└── repro_crash/         (test binary)
```

**Available syscalls (custom ABI):**
| NR | Name      | Status       | Notes                          |
|----|-----------|--------------|--------------------------------|
| 0  | read      | Implemented  | stdin only (fd 0)              |
| 1  | write     | Implemented  | stdout/stderr (fd 1, 2)        |
| 2  | exit      | Implemented  | Terminates task                |
| 3  | getpid    | Implemented  | Returns task ID                |
| 4  | sbrk      | **STUB**     | Returns ENOSYS                 |
| 5  | spawn     | Implemented  | Spawn from initramfs           |
| 6  | exec      | **STUB**     | Returns ENOSYS                 |
| 7  | yield     | Implemented  | Cooperative scheduling         |
| 8  | shutdown  | Implemented  | PSCI system halt               |

**Key gaps:**
- No heap allocation (sbrk is stub)
- No file open/close
- No directory listing
- No time syscalls
- No argument/environment passing

### 3.2 Existing Workarounds
- Shell uses stack buffers only (no heap)
- All output goes to hardcoded fd 1
- No file access beyond stdin/stdout

### 3.3 Related Specification
`docs/specs/userspace-abi.md` defines a target Linux-compatible ABI, but the current implementation uses a custom ABI. This creates a decision point: migrate to Linux ABI or continue with custom?

---

## 4. Codebase Reconnaissance

### 4.1 Code Areas to Modify

**Kernel (`kernel/src/syscall.rs`):**
- Implement `sys_sbrk` properly
- Add file syscalls: `openat`, `close`, `fstat`, `getdents64`
- Add time syscalls: `nanosleep`, `clock_gettime`

**HAL (`crates/hal/`):**
- Timer access for time syscalls
- Potential RTC driver for wall-clock time

**Userspace (`userspace/`):**
- Create `ulib/` crate
- Global allocator implementation
- File abstraction layer
- Environment parsing

### 4.2 Public APIs Involved
- `libsyscall::*` - raw syscall wrappers
- `los_hal::timer::*` - kernel timer access
- `crate::fs::*` - kernel filesystem (currently initramfs only)

### 4.3 Tests/Golden Files Impacted
- Shell behavior tests (if shell uses new allocator)
- Boot sequence golden files (new log messages)
- New tests for allocator and file operations

### 4.4 Non-Obvious Constraints
- **TTBR0 page tables**: Each process has its own page tables; `sbrk` must map new pages
- **No VFS yet**: Only initramfs exists; "files" are read-only archives
- **Single address space model**: Current design has shared kernel/user mapping complexity
- **Custom ABI lock-in**: Switching to Linux ABI would require updating all existing code

---

## 5. Constraints

### 5.1 Performance
- Allocator must be fast for typical small allocations
- Avoid kernel calls for every allocation (batch with sbrk)

### 5.2 Compatibility
- Must not break existing shell or init
- Should follow existing coding patterns (Rule 0)

### 5.3 Architecture
- AArch64 only (for now)
- QEMU virt machine target

### 5.4 UX
- Library should feel idiomatic to Rust developers
- Error messages should be helpful

---

## 6. Dependencies

### 6.1 Blocking Dependencies
None - can start immediately after Phase 8d completion.

### 6.2 Soft Dependencies
- Full VFS would enhance file operations but not required for MVP
- FAT32 filesystem exists but is for block device, not initramfs

---

## 7. Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| sbrk complexity with page tables | Medium | High | Start with fixed heap region, dynamic later |
| ABI incompatibility decisions | Medium | Medium | Document decisions, defer Linux compat |
| Breaking shell during development | Low | Medium | Feature-flag new code, test frequently |

---

## 8. Questions Generated (Phase 1)

Few questions at this stage (discovery). Major behavioral questions will emerge in Phase 2.

1. **Q1**: Should we continue with custom ABI or migrate to Linux ABI?
   - **Recommendation**: Continue custom ABI for Phase 10; Linux ABI is a future goal
   - **Rationale**: Breaking change to all userspace code; not needed for MVP

---

## Next Steps

1. Proceed to Phase 2 (Design)
2. Define `ulib` API surface
3. Design sbrk/heap implementation
4. Generate behavioral questions for uncertain areas
