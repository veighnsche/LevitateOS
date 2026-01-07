# Phase 2 — Step 2: HAL Trait Extraction

## Parent
Phase 2: Design — x86_64 Support

## Goal
Extract architecture-independent traits for MMU and Interrupts to decouple the kernel from AArch64 hardware specifics.

## Tasks
1. [x] **Define Traits in `los_hal`**:
    - [x] Create `crates/hal/src/traits.rs`.
    - [x] Define `MmuInterface` trait as sketched in `phase-2.md`.
    - [x] Define `InterruptController` trait as sketched in `phase-2.md`.
2. [x] **Refactor `crates/hal/src/mmu.rs`**:
    - [x] Implement `MmuInterface` for the existing AArch64 MMU logic.
    - [ ] Move AArch64-specific constants and flags into an `arch` submodule or keep them conditionally compiled but separate from the trait definition.
3. [x] **Refactor `crates/hal/src/gic/mod.rs`**:
    - [x] Implement `InterruptController` for the GIC.
4. [ ] **Update Kernel usage**:
    - [ ] Update `kernel/src/init.rs` and `kernel/src/input.rs` to use the generic traits instead of direct `gic` or `mmu` calls where possible.
5. [x] **Abstract arch-specific data structures** (TEAM_258):
    - [x] `Stat` struct: Added `Stat::new_device()`, `Stat::new_file()`, `Stat::new_pipe()`, `Stat::from_inode_data()` constructors.
    - [ ] `Termios` struct: Same pattern — use constructors/methods. (Deferred - not blocking)
    - [x] `SyscallNumber` enum: Using identical variant names with arch-specific numbers.
    - [x] `SyscallFrame`: Already uses methods (`arg0()`, `arg1()`, etc.) - no direct field access.
    - [x] `Context` struct: Added `Context::set_tls()` method.

> [!IMPORTANT]
> **Abstraction Principle**: Architecture-specific code should NOT require x86_64 to "look like" AArch64.
> Shared code must use traits or constructors, never direct field access on arch-specific structs.

## Expected Outputs
- [x] `los_hal` provides a clean trait interface for core hardware services.
- [x] Kernel code is less dependent on `aarch64-cpu` and GIC-specific structures.
- [x] Arch-specific structs (`Stat`, `SyscallFrame`, `Context`) have proper abstractions so shared code compiles for both architectures.
