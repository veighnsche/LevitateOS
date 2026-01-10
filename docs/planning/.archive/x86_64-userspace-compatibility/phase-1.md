# Phase 1: Discovery — x86_64 Userspace Compatibility

## Feature Summary
Enable x86_64 userspace processes to run on the LevitateOS kernel. This requires porting the syscall interface, the runtime entry point, and ensuring the kernel can dispatch x86_64 system calls.

### Problem Statement
The current userspace ecosystem is exclusively AArch64. While the kernel now supports x86_64, it lacks the mechanism to handle x86_64 userspace syscalls, and the userspace libraries lack the architecture-specific logic to make them.

### Beneficiaries
- Users wanting to run shells and applications on x86_64 hardware.
- Developers building cross-platform userspace tools for LevitateOS.

## Success Criteria
- [ ] `libsyscall` builds for `x86_64-unknown-none`.
- [ ] `ulib` provides a functional `_start` for x86_64.
- [ ] The kernel can receive and dispatch a `syscall` instruction from userspace.
- [ ] `init` successfully boots to a shell on x86_64.

## Current State Analysis
- **libsyscall**: Hardcoded `svc #0` in `userspace/libsyscall/src/io.rs` and other modules.
- **ulib**: Entry point logic is missing or AArch64-specific.
- **Kernel**: `kernel/src/arch/x86_64/task.rs` has unimplemented `enter_user_mode`. No `syscall` instruction handler is configured.
- **ELF Loader**: `kernel/src/loader/elf.rs` might have AArch64-specific assumptions (e.g., page size, entry point handling).
- **xtask**: `xtask/src/image.rs` lacks support for building x86_64 initramfs.

## Codebase Reconnaissance
- **`userspace/libsyscall/src/`**: Needs `arch/x86_64.rs` and macro updates.
- **`userspace/ulib/src/`**: Needs arch-specific entry points.
- **`kernel/src/arch/x86_64/`**: Needs `syscall.rs` for MSR configuration and entry handler.
- **`kernel/src/arch/x86_64/task.rs`**: Needs implementation of `enter_user_mode` and `cpu_switch_to`.
- **`kernel/src/loader/elf.rs`**: Verify compatibility with x86_64 ELF binaries.
- **`xtask/src/image.rs`**: Needs updates to build x86_64 initramfs.

## Constraints
- Must maintain AArch64 compatibility.
- Should follow Linux x86_64 syscall ABI for potential future compatibility.

## Steps
### Step 1: Capture Feature Intent
- **Goal**: Define the scope of x86_64 userspace support.
- **Tasks**: Document syscall ABI requirements and entry point needs.

### Step 2: Analyze Current State
- **Goal**: Identify all arch-locked code in `libsyscall` and `ulib`.
- **Tasks**: Audit `userspace/` for `asm!` blocks using AArch64 registers.

### Step 3: Source Code Reconnaissance
- **Goal**: Identify kernel-side requirements for `syscall` instruction.
- **Tasks**: Map out MSR initialization (STAR, LSTAR, FMASK).
