# TEAM_276: Implement x86_64 Syscall Primitives

## Goal
Implement real x86_64 syscall assembly in `arch/x86_64.rs` to replace stubs.

## Status: COMPLETE ✅

## x86_64 Syscall ABI
- **Instruction**: `syscall`
- **Syscall number**: `rax`
- **Arguments**: `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9` (6 args max)
- **Return value**: `rax`
- **Clobbered**: `rcx` (return addr), `r11` (saved rflags)

## Changes Made
- `arch/x86_64.rs` - Replaced stubs with real `syscall` instruction implementations

## Verification
- [x] x86_64 libsyscall compiles
- [x] AArch64 libsyscall still compiles

## Handoff
x86_64 userspace syscalls are now ready. Kernel syscall handler needs to be implemented for tests to pass.
