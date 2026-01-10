# Phase 1: Understanding and Scoping

## Bug Summary
- **Invalid Opcode Panic**: Kernel panics with `INVALID OPCODE` at RIP `0x100b9` (userspace shell).
- **Excessive Logging**: `[SYSCALL]` and `[TTY_READ]` logs.
- **Goal**: Fix the panic, silence the logs.

## Findings
- **Shell Binary**: Located at `userspace/target/x86_64-unknown-none/release/shell`. Disassembly needed to interpret `0x100b9`.
- **Syscall Logs**: Found in `kernel/src/arch/x86_64/syscall.rs` lines 186-218. They appear commented out in source but executing in kernel? (Needs verification of clean build).
- **TTY_READ Log**: Reference found in Team 300 logs. Still searching source. Suspect `pty.rs` or `crates/hal`.

## Hypothesis
- **Invalid Opcode**: The instruction at `0x100b9` might be `ud2`, a bad branch, or data interpreted as code.
- **Build Consistency**: The `syscall.rs` issue suggests the running kernel might be older or built with different flags than current source.

## Plan
1.  **Analyze Shell**: Check instructions at `0x100b9`.
2.  **Locate TTY_READ**: Check `pty.rs`.
3.  **Gate Syscall Logging**: Implement feature flag `verbose-syscalls` in `kernel/Cargo.toml` and usage in `syscall.rs`.
4.  **Fix Invalid Opcode**:
    - If `ud2` or panic: Shell panicked.
    - If valid instruction: CPU mismatch/corruption.
5.  **Clean up Unsafe**: Apply Rust 2024 fix (`unsafe` block inside `unsafe fn`).
