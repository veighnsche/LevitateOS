# Phase 2 — Design: Userspace & Syscalls

## Purpose
Define the solution architecture for userspace support. This phase generates the behavioral questions that must be answered before implementation.

> **THIS IS THE MOST IMPORTANT PHASE.**
> Do not rush to implementation. A well-designed feature with answered questions is faster to implement than a poorly-designed one with surprises.

---

## Proposed Solution

### 1. Exception Level Transition (EL1 → EL0)

The kernel will use `eret` to transition from kernel mode (EL1) to user mode (EL0).

```rust
/// Enter user mode at the specified entry point with the given stack.
///
/// # Safety
/// - `entry_point` must be a valid user-space address
/// - `user_sp` must point to a valid user stack
pub unsafe fn enter_user_mode(entry_point: usize, user_sp: usize) -> ! {
    // Configure SPSR for EL0 with interrupts enabled
    let spsr = 0b0000_0000; // EL0, SP_EL0, no flags
    
    asm!(
        "msr elr_el1, {entry}",
        "msr spsr_el1, {spsr}",
        "msr sp_el0, {sp}",
        "eret",
        entry = in(reg) entry_point,
        spsr = in(reg) spsr,
        sp = in(reg) user_sp,
        options(noreturn)
    );
}
```

### 2. Syscall Interface

User programs invoke syscalls using the `svc #0` instruction:

| Register | Purpose |
|----------|---------|
| `x8` | Syscall number |
| `x0`-`x5` | Arguments 1-6 |
| `x0` | Return value |

**Syscall Table (Initial)**:

| Number | Name | Signature |
|--------|------|-----------|
| 0 | `read` | `read(fd: u64, buf: *mut u8, len: u64) -> i64` |
| 1 | `write` | `write(fd: u64, buf: *const u8, len: u64) -> i64` |
| 2 | `exit` | `exit(code: u64) -> !` |
| 3 | `getpid` | `getpid() -> u64` |
| 4 | `sbrk` | `sbrk(increment: i64) -> *mut u8` |

### 3. User Address Space Layout

```
0x0000_0000_0000_0000  ┌─────────────────────────┐
                       │  NULL Guard (unmapped)  │ 4KB
0x0000_0000_0000_1000  ├─────────────────────────┤
                       │                         │
                       │     User Code (.text)   │ Loaded from ELF
                       │                         │
                       ├─────────────────────────┤
                       │     User Data (.data)   │
                       ├─────────────────────────┤
                       │          Heap           │ Grows up via sbrk
                       │           ↓             │
                       │                         │
                       │           ↑             │
                       │         Stack           │ Grows down
0x0000_7FFF_FFFF_F000  ├─────────────────────────┤
                       │  Stack Guard (unmapped) │ 4KB
0x0000_8000_0000_0000  └─────────────────────────┘
                       ┌─────────────────────────┐
                       │     Kernel (TTBR1)      │
0xFFFF_8000_0000_0000  └─────────────────────────┘
```

### 4. User TaskControlBlock Extension

```rust
pub struct UserTask {
    /// Base kernel task info
    pub base: TaskControlBlock,
    
    /// User's TTBR0 (L0 page table physical address)
    pub ttbr0: usize,
    
    /// User's stack pointer (SP_EL0)
    pub user_sp: usize,
    
    /// Program break (end of heap)
    pub brk: usize,
    
    /// Entry point from ELF
    pub entry_point: usize,
}
```

### 5. Syscall Handler Flow

```
User calls svc #0
        │
        ▼
┌───────────────────────────────┐
│ sync_handler_entry (vector.s) │
│ - Save all GPRs to stack      │
│ - Check ESR_EL1 for SVC       │
│ - Call syscall_dispatch(x8)   │
└───────────────────────────────┘
        │
        ▼
┌───────────────────────────────┐
│ syscall_dispatch (syscall.rs) │
│ - Match syscall number        │
│ - Call handler with args      │
│ - Store result in x0          │
└───────────────────────────────┘
        │
        ▼
┌───────────────────────────────┐
│ Return to user (eret)         │
│ - Restore GPRs from stack     │
└───────────────────────────────┘
```

---

## Behavioral Decisions (EXPECTED FOR USER REVIEW)

### Q1: Syscall ABI — Linux-compatible or custom?

- **Option A**: Linux-compatible syscall numbers
  - *Pro*: Easier to port existing tools
  - *Con*: Must implement many syscalls to be useful
  
- **Option B**: Custom minimal ABI
  - *Pro*: Simple, only implement what we need
  - *Con*: No code portability

- **Decision**: **Option B (Custom)** initially, with documentation of Linux equivalents.
- **Justification**: We're building from scratch. A minimal ABI lets us ship faster. Document a migration path for Linux compatibility later.

### Q2: File Descriptors — How to handle?

- **Option A**: Hardcoded FD 0/1/2 for stdin/stdout/stderr only
  - *Pro*: Simplest possible implementation
  - *Con*: No file I/O
  
- **Option B**: Global FD table in kernel
  - *Pro*: Matches POSIX model
  - *Con*: More complex

- **Decision**: **Option A** for Phase 8. Real FD table deferred.
- **Justification**: Goal is "Hello World", not full POSIX. Hardcoded console FDs are sufficient.

### Q3: Process Isolation — Copy-on-write or eager copy?

- **Option A**: Eager copy (fork copies all pages immediately)
  - *Pro*: Simple to implement
  - *Con*: Slow, uses more memory

- **Option B**: Copy-on-write (pages shared until written)
  - *Pro*: Fast fork, memory efficient
  - *Con*: Complex page fault handling

- **Decision**: **Defer fork entirely** for Phase 8. Only support `exec` of new processes.
- **Justification**: HelloWorld doesn't need fork. Add fork in a later phase.

### Q4: ELF Loading — Static or dynamic linking?

- **Option A**: Static linking only
  - *Pro*: Simple, self-contained binaries
  - *Con*: Larger binaries, no shared libs

- **Option B**: Dynamic linking with ld.so
  - *Pro*: Shared libraries, smaller binaries
  - *Con*: Very complex (relocations, symbol resolution)

- **Decision**: **Option A (Static only)** for Phase 8.
- **Justification**: Dynamic linking is a significant feature. Static binaries are sufficient for initial userspace.

### Q5: What happens on invalid syscall number?

- **Option A**: Return -ENOSYS (error)
- **Option B**: Kill the process
- **Option C**: Kernel panic (debug builds only)

- **Decision**: **Option A** (Return error).
- **Justification**: **Rule 14 (Fail Fast)** applies to invariants, not user errors. User errors should be reported, not crash the system.

### Q6: Stack size for user processes?

- **Decision**: **64KB** initially (matching kernel tasks).
- **Justification**: **Rule 20 (Simplicity > Perfection)**. Larger stacks avoid stack overflow debugging. Optimize later.

---

## Design Alternatives Considered

### Alternative 1: Run user code in EL1 with software isolation
- **Rejected**: Violates security principles. Hardware isolation (EL0/EL1) is the standard approach.

### Alternative 2: Use a microkernel approach (messages instead of syscalls)
- **Rejected**: More complex for initial implementation. Traditional syscalls are well-understood.

### Alternative 3: Use Linux syscall numbers from the start
- **Deferred**: Adds complexity. Custom ABI first, migrate later if needed.

---

## Open Questions — Answered (TEAM_073)

> **RESOLVED**: Questions answered during plan review.

1. **Syscall ABI**: ✅ **Custom ABI** — Confirmed. Document Linux equivalents for future migration.

2. **First User Program**: ✅ **Build as part of project** — Create `userspace/hello/` with Rust `#![no_std]` binary.

3. **Error Handling**: ✅ **Option A (Print error and kill process)** — Matches Rule 14 (Fail Fast). Log exception details to UART before terminating.

4. **Console I/O**: ✅ **Option C (Both UART and GPU)** — Matches current kernel behavior. `write(1, ...)` outputs to both console backends.

---

## Steps

### Step 1 — Draft Initial Design ✅
- Defined EL0 transition, syscall interface, address space layout

### Step 2 — Define Behavioral Contracts ✅
- Answered Q1-Q6 with decisions and justifications
- Listed remaining open questions for user

### Step 3 — Review Design Against Architecture
- [ ] Verify fits with existing MMU and task infrastructure
- [ ] Confirm no conflicts with scheduler

### Step 4 — Finalize Design After Questions Answered
- [ ] Incorporate user answers
- [ ] Update behavioral contracts
- [ ] Begin Phase 3 (Implementation)

---

## Next Phase

After user answers open questions, proceed to [Phase 3 — Implementation](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-3.md).
