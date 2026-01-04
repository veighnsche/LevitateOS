# Phase 8: Userspace & Syscalls — Overview

## Feature Summary

**Phase 8** introduces userspace execution to LevitateOS. Currently, all code runs in privileged kernel mode (EL1). This phase enables unprivileged user programs to run in user mode (EL0) with controlled access to kernel services via system calls.

### Problem Statement
Without userspace, all code runs with full kernel privileges, making the system:
- Vulnerable to bugs crashing the entire system
- Unable to run untrusted code safely
- Missing the process isolation expected of a real OS

### Who Benefits
- Developers writing user applications for LevitateOS
- The kernel, which gains protection from buggy user code
- Future security features (sandboxing, permissions)

---

## Success Criteria

1. **EL0 Execution**: User code runs in EL0 (unprivileged) mode.
2. **Syscall Interface**: User code can request kernel services via `svc` instruction.
3. **ELF Loading**: The kernel can load ELF binaries from initramfs/disk.
4. **Process Isolation**: User processes have separate address spaces (TTBR0).
5. **Basic Syscalls**: At minimum: `write`, `read`, `exit`, `getpid`.
6. **Hello World**: A minimal ELF binary prints "Hello from userspace!" and exits cleanly.

---

## Technical Strategy

### 1. Exception Level Transition (EL1 → EL0)
- Configure `SPSR_EL1` for EL0 execution
- Set `ELR_EL1` to user entry point
- Use `eret` to enter user mode
- User stack must be separate from kernel stack

### 2. Syscall Handling (SVC)
- `svc #0` triggers synchronous exception
- Vector table routes to syscall handler
- Syscall number in `x8`, arguments in `x0-x5`
- Return value in `x0`

### 3. User Address Space
- Each process gets its own TTBR0 page table
- Kernel space (TTBR1) remains shared
- User pages mapped with `AP_RW_EL0` flags

### 4. ELF Loading
- Parse ELF64 header and program headers
- Map PT_LOAD segments into user address space
- Set up initial stack with argc/argv/envp

---

## Key Components

| Component | File | Description |
|-----------|------|-------------|
| Syscall Table | `kernel/src/syscall.rs` | Syscall dispatch and handlers |
| User Task | `kernel/src/task/user.rs` | User process creation |
| ELF Loader | `kernel/src/loader/elf.rs` | ELF binary parsing |
| User VMM | `kernel/src/task/user_mm.rs` | User address space management |

---

## Recommended Sequence

1. **Task 8.1**: EL0 Transition — Implement `enter_user_mode()` and verify basic EL0 execution.
2. **Task 8.2**: Syscall Interface — Implement SVC handler and minimal syscall (e.g., `exit`).
3. **Task 8.3**: User Address Space — Create per-process TTBR0 page tables.
4. **Task 8.4**: ELF Loader — Parse and load ELF binaries.
5. **Task 8.5**: Integration — Run a "Hello World" ELF from initramfs.

---

## Phase Structure

- [Phase 1 — Discovery](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-1.md)
- [Phase 2 — Design](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-2.md)
- [Phase 3 — Implementation](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-3.md) (future)

---

## References

- ARM Architecture Reference Manual: Exception Levels
- Linux kernel: `arch/arm64/kernel/entry.S` (exception vectors)
- Redox OS: `kernel/src/syscall/mod.rs`
- Theseus OS: `kernel/spawn/src/lib.rs`
- OSDev Wiki: [System Calls](https://wiki.osdev.org/System_Calls)

---

## Current Status: Discovery (Phase 1)

See `phase-1.md` for the discovery phase.
