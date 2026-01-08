# x86_64 Architecture Implementation Guide

This document details the specific implementation choices, constraints, and "gotchas" for the LevitateOS x86_64 port. Future teams should reference this when modifying syscall handling, task management, or interrupt subsystems.

## 1. System Calls (ABI & Implementation)

### 1.1 Register Calling Convention
We follow the standard Linux x86_64 syscall ABI:
- **Instruction**: `syscall` (entry) / `sysretq` (exit)
- **Syscall Number**: `RAX`
- **Arguments**: `RDI`, `RSI`, `RDX`, `R10`, `R8`, `R9` (Note: `R10` is used instead of `RCX` because `syscall` clobbers `RCX`)
- **Return Value**: `RAX`
- **Clobbered by CPU**: `RCX` (return address), `R11` (saved RFLAGS)

### 1.2 The `RCX`/`R11` Critical Path
**CRITICAL:** The `syscall` instruction saves the return address into `RCX` and the RFLAGS into `R11`.
- On return (`sysretq`), the CPU restores `RIP` from `RCX` and `RFLAGS` from `R11`.
- **Bug Fix History:** Earlier versions corrupted `RAX` (return value) because the logic to sanitize `RCX`/`R11` happened *after* restoring `RAX`.
- **Constraint:** `RAX` must be popped off the stack *after* any register manipulation that uses `RAX` as a temporary.
- **Sanitization:** `R11` (RFLAGS) is forcibly sanitized to enable interrupts (`IF=1`) and set the reserved bit (bit 1).

### 1.3 `SyscallFrame` Layout
The `SyscallFrame` struct in Rust **must exactly match** the order of registers pushed in `syscall_entry` assembly.
- Current Layout (Top of Stack -> Bottom):
  1. `R11` (Saved RFLAGS)
  2. `RCX` (Saved RIP)
  3. `RBP`
  4. `RBX`
  5. `R12`, `R13`, `R14`, `R15`
  6. `RAX` (Syscall Number / Return Value)
  7. `R9`, `R8`, `R10`, `RDX`, `RSI`, `RDI` (Arguments)

## 2. Task Management & Context Switching

### 2.1 Struct Layouts (`#[repr(C)]`)
All structures shared between Rust and Assembly, or critical for memory layout integrity, **must** be marked `#[repr(C)]`.
- **Affected Structs:** `Pid`, `TaskId`, `UserTask`, `TaskControlBlock`.
- **Reason:** The default Rust layout (`#[repr(Rust)]`) allows field reordering. This caused subtle bugs where PIDs were read as entry points or vice-versa.

### 2.2 Address Space (HHDM)
- **Higher Half Direct Map (HHDM):** The kernel maps all physical memory to `0xFFFF_8000_0000_0000 + PA`.
- **Kernel Base:** The kernel code itself is loaded at `0xFFFF_FFFF_8000_0000`.
- **User Mode:** Userspace runs in the lower half (canonical addresses starting at `0x0`).

## 3. ELF Loading & Userspace

### 3.1 ELF Type Constraint (`ET_EXEC`)
- **Requirement:** Userspace binaries (like `init`, `shell`, `ls`) must be linked as `ET_EXEC` (Executable), **NOT** `ET_DYN` (PIE).
- **Reason:** Our simple ELF loader does not support dynamic relocation (ASLR) yet. It expects to load segments at fixed addresses (e.g., `0x10000`).
- **Build Config:** `userspace/build.rs` forces `-no-pie` for x86_64 targets.

### 3.2 The `_start` Entry Point
- **Problem:** Rust's default `_start` is not always available or correct for x86_64 generic targets.
- **Solution:** `ulib` explicitly implements a naked `_start` function for x86_64.
- **Responsibility:**
  1. Zero `RBP` (Frame Pointer) to mark the end of stack traces.
  2. Move `RSP` to `RDI` (first arg) for Rust entry.
  3. Align stack to 16 bytes (`and rsp, -16`).
  4. Call `_start_rust`.

## 4. Interrupts & Hardware Timers

### 4.1 Timer Source (PIT)
- **Source:** Programmable Interval Timer (PIT).
- **Frequency:** Configured to ~100Hz in `kernel/src/init.rs`.
- **Interrupt Vector:** Mapped to **Vector 32** in the IDT.

### 4.2 IOAPIC Routing
- **Legacy IRQ 0:** The PIT fires on Legacy IRQ 0.
- **Constraint:** We must explicitly route Legacy IRQ 0 to Vector 32 in the IOAPIC.
- **Implementation:** `los_hal::arch::ioapic::IOAPIC.route_irq(0, 32, 0);`

### 4.3 GPU Refresh
- **Mechanism:** The GPU framebuffer flush is triggered periodically by the timer interrupt handler in `init.rs`.
- **Symptom:** If the screen is black but `[SUCCESS] System Ready` prints to serial, the timer interrupt is likely masked or not firing.

## 5. Debugging Techniques

### 5.1 "Magic" Breakpoints
- `b syscall_entry`: Catches every syscall.
- `b *0x...`: Break at exact instruction pointer (useful for investigating `RIP=0`).

### 5.2 Verification Logs
- `[TICK]`: Enabled in `init.rs` (throttled) to verify timer interrupts.
- `[SYSCALL] ENTER/EXIT`: Enabled via `verbose-syscalls` feature to trace execution flow.
