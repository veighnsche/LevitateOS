# Phase 3 — Step 1: libsyscall Arch Abstraction

## Parent
[Phase 3: Implementation](phase-3.md)

## Goal
Decouple `libsyscall` from AArch64 and implement x86_64 `syscall` instruction support.

## Tasks
1. **Create arch module structure**:
   - Create `userspace/libsyscall/src/arch/mod.rs`.
   - Move AArch64 `asm!` blocks into `userspace/libsyscall/src/arch/aarch64.rs`.
   - Create `userspace/libsyscall/src/arch/x86_64.rs`.
2. **Implement x86_64 syscalls**:
   - Implement `syscall0` through `syscall6` in `x86_64.rs` using the `syscall` instruction.
   - Use registers: RAX (num), RDI, RSI, RDX, R10, R8, R9.
3. **Refactor sysno.rs**:
   - Add `#[cfg(target_arch = "x86_64")]` constants to `userspace/libsyscall/src/sysno.rs` matching the kernel's `SyscallNumber` enum.
4. **Update syscall wrappers**:
   - Ensure all modules (`io.rs`, `fs.rs`, etc.) use the new arch-abstracted syscall functions.

## Exit Criteria
- `libsyscall` builds successfully for `x86_64-unknown-none`.
- No AArch64 registers are referenced when building for x86_64.
