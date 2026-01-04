# Phase 3: Implementation — Userspace & Syscalls

This phase implements userspace support following the design established in Phase 2.

> **IMPORTANT**: Do not start this phase until all open questions in Phase 2 are answered by the user.

## Implementation Overview

The implementation is broken down into five major steps, each addressing a core component.

### [Step 1: EL0 Transition](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-3-step-1.md)
Implement the mechanism to enter user mode (EL0) from kernel mode (EL1).

### [Step 2: Syscall Handler](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-3-step-2.md)
Implement the SVC exception handler and syscall dispatch table.

### [Step 3: User Address Space](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-3-step-3.md)
Implement per-process TTBR0 page tables and user memory management.

### [Step 4: ELF Loader](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-3-step-4.md)
Implement ELF64 binary parsing and loading.

### [Step 5: Integration & HelloWorld](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-3-step-5.md)
Integrate all components and run the first user program.

## Design Reference

All implementation follows the decisions documented in [phase-2.md](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-2.md).

Key decisions:
- Custom syscall ABI (not Linux-compatible initially)
- Static ELF linking only
- Hardcoded FD 0/1/2 for console
- 64KB user stack
- No fork (only exec)

## Prerequisites

Before starting Phase 3, verify:
- [ ] All Phase 2 questions answered by user
- [ ] Phase 7 multitasking is complete (scheduler, context switch)
- [ ] All tests pass: `cargo xtask test`
