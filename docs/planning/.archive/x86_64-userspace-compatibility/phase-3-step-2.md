# Phase 3 — Step 2: Kernel Syscall Handling

## Parent
[Phase 3: Implementation](phase-3.md)

## Goal
Configure the x86_64 CPU to handle the `syscall` instruction and dispatch to the kernel.

## Tasks
1. **MSR Configuration**:
   - In `kernel/src/arch/x86_64/syscall.rs`, implement `init()` to set:
     - `IA32_STAR`: Kernel/User segment selectors.
     - `IA32_LSTAR`: Address of `syscall_entry_asm`.
     - `IA32_FMASK`: Mask for RFLAGS (clear IF, etc.).
2. **Assembly Entry Point**:
   - Create `syscall_entry_asm` in a new assembly file or within `boot.S`.
   - Swap `GS` to access per-CPU data (kernel stack pointer).
   - Save user registers to the stack.
   - Call Rust `syscall_handler(frame: &mut SyscallFrame)`.
3. **Syscall Frame implementation**:
   - Update `SyscallFrame` in `kernel/src/arch/x86_64/mod.rs` to correctly map registers for the dispatcher.

## Exit Criteria
- Kernel correctly traps `syscall` instructions from Ring 3.
- Register state is preserved and passed to the dispatcher.
