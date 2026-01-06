# Phase 1: Discovery and Safeguards - Arch Abstraction

## Refactor Summary
The kernel currently has architecture-specific code (AArch64) tightly coupled with generic kernel logic in `boot.rs`, `exceptions.rs`, `syscall.rs`, and `task/mod.rs`. This refactor will extract this logic into a dedicated `crate::arch` module to enable multi-architecture support.

## Success Criteria
- [ ] Kernel builds for `aarch64` with the new abstraction.
- [ ] Behavior tests pass (Boot, Syscalls, Task Switching).
- [ ] `aarch64` specific code is isolated in `src/arch/aarch64`.
- [ ] Generic kernel code has no direct dependencies on `aarch64-cpu` or AArch64 registers.
- [ ] x86_64 "Stub" target enables with `unimplemented!()` but compiles, proving the boundary is clean.
- [ ] Early-boot diagnostic interface (e.g., `arch::early_println!`) is defined for new arch bringup.

## Behavioral Contracts
- **Boot Entry:** Kernel expects to be loaded at physical address base + 0x80000.
- **Syscall ABI:** 
  - Syscall number in `x8`.
  - Arguments in `x0-x5`.
  - Return in `x0`.
- **Exception Table:** Vectors must be correctly set in `VBAR_EL1`.

## Golden/Regression Tests
- `tests/golden_boot.txt`: Must match exactly after refactor.
- `cargo xtask test behavior`: All tests must pass.

## Current Architecture Notes
- `boot.rs`: Contains `global_asm!` block with AArch64 boot sequence and page table setup.
- `exceptions.rs`: Contains AArch64 vector table and exception handlers.
- `task/mod.rs`: Defines AArch64 `Context` and context switching assembly.
- `syscall.rs`: Defines `SyscallFrame` with AArch64 register layout.

## Open Questions & Decisions
- **Decision:** `los_hal` will remain AArch64-centric for this phase. The kernel will isolate its dependencies on `los_hal` through the `crate::arch` abstraction. A future task will refactor `los_hal` for true multi-arch.
- **Decision:** The abstraction will use a combination of conditional module exports (`#[cfg(target_arch = "...")]`) and common traits where dynamic dispatch or unified naming is required.
- **Decision: Debuggability Policy:** Every architecture MUST implement an `EarlyConsole` trait or provide an `early_write` function that the kernel can use before the full HAL/UART is discoverable via FDT/ACPI.

## Steps
1. **Step 1: Map Current Behavior and Boundaries**
   - Identify every file using `core::arch::global_asm!`, `aarch64-cpu`, or AArch64-specific `los_hal::mmu` functions.
2. **Step 2: Lock in Golden Tests**
   - Verify `tests/golden_boot.txt` is current.
3. **Step 3: Define Initial Arch Interface**
   - Draft the `Arch` trait or common module exports needed for `main.rs` and `task/mod.rs`.
