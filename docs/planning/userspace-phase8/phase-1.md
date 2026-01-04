# Phase 1 — Discovery: Userspace & Syscalls

## Purpose
Understand the problem domain and analyze the existing codebase before designing the userspace implementation.

---

## Feature Intent

### Problem Statement
LevitateOS currently runs all code in kernel mode (EL1). There is no mechanism for:
- Running untrusted code safely
- Isolating processes from each other
- Limiting what user programs can do

### Who Benefits
- **Application developers**: Can write programs without worrying about crashing the kernel
- **The kernel**: Protected from bugs in user code
- **Security**: Foundation for future sandboxing and permissions

### How We Know It's Done
1. A simple ELF binary runs in EL0
2. It can call `write` syscall to print to console
3. It can call `exit` syscall to terminate cleanly
4. Kernel continues running after user process exits

---

## Current State Analysis

### What Exists Today
- **Multitasking**: Phase 7 provides task primitives (`TaskControlBlock`, `Context`, scheduler)
- **Address Spaces**: MMU supports TTBR0 (user) and TTBR1 (kernel) separation
- **Exception Handling**: Vector table exists in `kernel/src/exceptions.rs`
- **Timer/Interrupts**: GIC and timer drivers handle IRQs

### What's Missing
- No EL0 transition mechanism
- No syscall handler (SVC exception is not routed)
- No per-process address space (all tasks share kernel space)
- No ELF parser
- No user-facing API (syscall table)

### Current Code Paths

| Function | Location | Relevance |
|----------|----------|-----------|
| `TaskControlBlock::new` | `kernel/src/task/mod.rs` | Creates kernel tasks, needs user variant |
| `cpu_switch_to` | `kernel/src/task/mod.rs` | Context switch, will switch TTBR0 for user |
| `sync_handler_entry` | `kernel/src/exceptions.rs` | Sync exceptions, will handle SVC |
| `map_page` | `levitate-hal/src/mmu.rs` | Page mapping, needs user-mode flags |

---

## Codebase Reconnaissance

### Modules to Create
- `kernel/src/syscall.rs` — Syscall dispatch table and handlers
- `kernel/src/syscall/` — Individual syscall implementations
- `kernel/src/loader/elf.rs` — ELF binary parser
- `kernel/src/task/user.rs` — User process management

### Modules to Modify
- `kernel/src/exceptions.rs` — Route SVC to syscall handler
- `kernel/src/task/mod.rs` — Add `UserTask` variant or extend `TaskControlBlock`
- `levitate-hal/src/mmu.rs` — Add user-mode page flags (`AP_RW_EL0`)

### Tests/Golden Files Impacted
- Behavior tests may need update if boot sequence changes
- New behavior inventory entries for syscall behaviors

---

## Constraints

### Performance
- Syscall overhead should be minimal (fast path for common syscalls)
- Context switch must save/restore user registers efficiently

### Compatibility
- Syscall ABI should be Linux-compatible where practical (eases porting)
- ELF format should be standard ELF64 for AArch64

### Security
- User pages must not be executable in kernel mode
- User cannot access kernel memory
- SMEP/SMAP equivalents if available on target

---

## Open Questions (Discovery Phase)

These questions should be answered before design:

> **Q1**: Should we use Linux syscall numbers or define our own ABI?
> - *Option A*: Linux-compatible (easier porting of tools)
> - *Option B*: Custom ABI (simpler, no legacy baggage)
> - *Recommendation*: Start with custom, document migration path

> **Q2**: What's the minimum set of syscalls for "Hello World"?
> - Likely: `write(fd, buf, len)`, `exit(code)`
> - Maybe: `getpid()` for testing

> **Q3**: How do we handle the first user process?
> - *Option A*: Kernel spawns `/init` from initramfs
> - *Option B*: Special `exec_init()` syscall from kernel task
> - *Recommendation*: Kernel spawns `/init` (matches Linux model)

---

## Steps

### Step 1 — Capture Feature Intent ✅
- Problem statement defined above
- Success criteria listed

### Step 2 — Analyze Current State ✅
- Documented existing multitasking infrastructure
- Identified gaps (no EL0, no syscalls, no ELF)

### Step 3 — Source Code Reconnaissance ✅
- Listed modules to create and modify
- Identified test impact

---

## Next Phase

Proceed to [Phase 2 — Design](file:///home/vince/Projects/LevitateOS/docs/planning/userspace-phase8/phase-2.md) to define the syscall ABI, EL0 transition mechanism, and address space layout.
