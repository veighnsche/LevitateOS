# Phase 3: Implementation — x86_64 Support (Intel NUC)

## Implementation Plan

This phase covers the initial implementation of the x86_64 architecture support, focusing on reaching a bootable state with serial output.

> **Note**: Each step has a detailed breakdown in its own file with SLM-sized units of work.

### Step 1: Toolchain and Build Support
- [x] Add `x86_64-unknown-none` target to `rust-toolchain.toml`.
- [x] Update `xtask` to support `--arch x86_64`.
- [x] Add a `q35` QEMU profile to `xtask`.

### Step 2: Early Boot (Assembly)
**Detailed Plan**: [phase-3-step-2.md](phase-3-step-2.md) — 8 UoWs

- [x] UoW 2.1: Create x86_64 linker script
- [x] UoW 2.2: Implement Multiboot2 header
- [x] UoW 2.3: Create GDT64 for long mode
- [x] UoW 2.4: Implement protected-to-long mode transition
- [x] UoW 2.5: Set up early identity page tables
- [x] UoW 2.6: Implement 64-bit entry point
- [x] UoW 2.7: Create Rust entry point stub
- [x] UoW 2.8: Wire build system for boot.S

### Step 3: Architecture-Specific Stubs implementation
- [x] Fill in `kernel/src/arch/x86_64/cpu.rs` with GDT and basic CPU initialization.
- [x] Implement `kernel/src/arch/x86_64/exceptions.rs` with a basic IDT.
- [x] Implement `kernel/src/arch/x86_64/task.rs` context switching logic.

> **Note**: Above items are marked complete as stubs exist, but functional GDT/IDT/context-switch implementation is still pending.

### Step 4: HAL Implementation (x86_64)
**Detailed Plan**: [phase-3-step-4.md](phase-3-step-4.md) — 10 UoWs

- [x] UoW 4.1: Implement Serial Console (COM1)
- [x] UoW 4.2: Implement VGA Text Mode Console
- [x] UoW 4.3: Implement IDT Structure
- [x] UoW 4.4: Implement CPU Exception Handlers
- [x] UoW 4.5: Implement IDT Loading and Initialization
- [x] UoW 4.6: Detect and Initialize Local APIC
- [x] UoW 4.7: Implement I/O APIC for External IRQs
- [x] UoW 4.8: Implement PIT Timer
- [x] UoW 4.9: Implement InterruptController Trait for APIC
- [x] UoW 4.10: Create x86_64 HAL Module Structure

### Step 5: MMU & Higher-Half
**Detailed Plan**: [phase-3-step-5.md](phase-3-step-5.md) — 9 UoWs

- [x] UoW 5.1: Define Page Table Entry Structures
- [x] UoW 5.2: Implement Page Table Walker
- [x] UoW 5.3: Implement 4KB Page Mapper
- [x] UoW 5.4: Implement Page Unmapper
- [x] UoW 5.5: Implement Frame Allocator Interface
- [x] UoW 5.6: Create Higher-Half Kernel Mappings
- [x] UoW 5.7: Implement CR3 Switching
- [x] UoW 5.8: Implement MmuInterface Trait for PML4
- [x] UoW 5.9: Transition to Higher-Half at Boot

## Progress Tracking
- [x] Step 1: Toolchain
- [x] Step 2: Early Boot
- [/] Step 3: Arch Stubs (stubs exist, not functional)
- [x] Step 4: HAL Backends
- [x] Step 5: MMU
