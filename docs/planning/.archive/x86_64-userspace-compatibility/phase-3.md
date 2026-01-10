# Phase 3: Implementation — x86_64 Userspace Compatibility

## Implementation Plan

This phase covers porting the userspace libraries and implementing the kernel-side syscall handling.

### Step 1: libsyscall Refactoring
- [ ] Create `userspace/libsyscall/src/arch/mod.rs` to abstract syscall invocation.
- [ ] Implement `x86_64.rs` with `syscall` instruction wrapper.
- [ ] Update all syscall wrappers (read, write, etc.) to use the abstracted `syscall!` macro.

### Step 2: Kernel Syscall Handling
- [ ] Implement `kernel/src/arch/x86_64/syscall.rs` to configure MSRs.
- [ ] Implement the assembly entry point for `syscall`.
- [ ] Wire the assembly entry to the existing Rust `syscall_dispatch`.

### Step 3: Process Lifecycle (Arch Stubs)
- [ ] Implement `cpu_switch_to` in assembly for x86_64.
- [ ] Implement `enter_user_mode` in `kernel/src/arch/x86_64/task.rs`.
- [ ] Implement `task_entry_trampoline`

### Step 4: Kernel Loader Updates
- [ ] Update ELF loader to support `EM_X86_64`.
- [ ] Verify page alignment requirements for x86_64 userspace.

### Step 5: ulib and Entry Point
- [ ] Implement `_start` for x86_64 in `userspace/ulib/src/arch/x86_64.rs`.
- [ ] Ensure correct stack alignment (16-byte) before calling `main`.
- [ ] Provide arch-specific `linker.ld` for x86_64 userspace if needed.

## Progress Tracking
- [ ] Step 1: libsyscall
- [ ] Step 2: Kernel Syscall
- [ ] Step 3: Context Switching
- [ ] Step 4: Kernel Loader
- [ ] Step 5: ulib Entry
