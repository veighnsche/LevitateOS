# Phase 2: Design — x86_64 Userspace Compatibility

## Proposed Solution
Introduce a multi-arch abstraction in `libsyscall` and implement the x86_64 `syscall` instruction path in the kernel.

### User-Facing Behavior
- Userspace developers can compile for `x86_64-unknown-none`.
- The build system (`xtask`) produces x86_64 initramfs images.

### System Behavior
1. **Userspace**: `libsyscall` uses `syscall` instruction with RAX (syscall number) and RDI, RSI, RDX, R10, R8, R9 (arguments).
2. **Kernel Entry**: Configure `LSTAR` MSR to point to a common assembly entry point.
3. **Context Switch**: Implement `cpu_switch_to` in assembly to swap registers and stacks.
4. **Task Entry**: Implement `enter_user_mode` to transition to Ring 3 via `sysret` or `iretq`.
5. **ELF Loading**: Ensure the ELF loader correctly handles x86_64 binaries (EM_X86_64) and maps them at appropriate virtual addresses.

## API Design

### 1. libsyscall Arch Trait
Internal abstraction to select syscall mechanism:
```rust
#[cfg(target_arch = "x86_64")]
pub unsafe fn syscall(num: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "syscall",
        in("rax") num,
        in("rdi") a0,
        in("rsi") a1,
        in("rdx") a2,
        in("r10") a3,
        in("r8") a4,
        in("r9") a5,
        lateout("rax") ret,
        out("rcx") _, // destroyed by syscall
        out("r11") _, // destroyed by syscall
        options(nostack)
    );
    ret
}
```

## Behavioral Decisions
- **Syscall ABI**: Use Linux-compatible registers (RDI, RSI, RDX, R10, R8, R9) to leverage existing knowledge and potential tool compatibility.
- **Segmentation**: Use flat model (0 base, 2^64 limit) for userspace segments.

## Open Questions
- **Q2.1**: Should we use `sysret` or `iretq` for the initial transition to userspace?
  - *Recommendation*: Use `iretq` for the very first entry as it allows setting the full context more easily; use `sysret` for returning from subsequent syscalls if performance becomes a concern.
- **Q2.2**: How to handle TLS (Thread Local Storage) on x86_64?
  - *Recommendation*: Use `FS_BASE` MSR for the TLS pointer, matching AArch64's `TPIDR_EL0` usage.

## Steps
### Step 1: Draft Initial Design
- **Goal**: Formalize the syscall and entry point contracts.
- **Tasks**: Define the register mapping and MSR setup sequence.

### Step 2: Define Behavioral Contracts
- **Goal**: Specify how the kernel transitions between Ring 0 and Ring 3.
- **Tasks**: Document the stack switching logic and GDT requirements for userspace segments.

### Step 3: Review Design Against Architecture
- **Goal**: Ensure the new `libsyscall` structure fits the existing `ulib`.
- **Tasks**: Verify that `_start` can be implemented without platform-specific hacks in `main.rs`.
