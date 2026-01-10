# Phase 1: Discovery — x86_64 Support (Intel NUC)

## Feature Summary
Support the Intel NUC (x86_64 i3 7th Gen) as a hardware target. This requires extending the existing AArch64-centric abstractions to accommodate x86_64 specific hardware (PML4 paging, APIC interrupts, VGA/Serial console) and implementing a Multiboot2 bootloader entry.

### Problem Statement
The current kernel is heavily coupled to AArch64 (GIC, ARM Timers, TTBR paging). To support the Intel NUC, we must decouple these into clean architecture-independent traits and implement the x86_64 backends.

### Beneficiaries
- Users wanting to run LevitateOS on commodity x86_64 hardware.
- Developers needing a multi-arch testing ground for kernel abstractions.

## Success Criteria
- [ ] Kernel builds for `x86_64-unknown-none`.
- [ ] QEMU `q35` can boot the kernel via Multiboot2 to a "Hello World" serial/VGA output.
- [ ] Memory management (PML4) initialized and kernel running in higher-half.
- [ ] Basic interrupt handling (IDT/APIC) functional.

## Current State Analysis
- **Architecture**: `kernel/src/arch/mod.rs` already has a `x86_64` module with stubs for `SyscallNumber`, `Stat`, `Termios`, `SyscallFrame`, `Context`.
- **HAL**: `los_hal` has trait abstractions (`InterruptController`, `MmuInterface`) with AArch64 implementations. x86_64 backends needed.
- **Boot**: Only FDT-based ARM boot is implemented. Multiboot2 entry needed.
- **Drivers**: No x86_64 specific drivers (VGA, PIT, APIC, NVMe).
- **Toolchain**: `x86_64-unknown-none` target and `xtask --arch x86_64` are IMPLEMENTED.

## Codebase Reconnaissance
- **`kernel/src/arch/x86_64/`**: Primary area for assembly entry and arch-specific CPU/Task logic.
- **`crates/hal/src/`**: Needs refactoring to support multiple backends for MMU, Timer, and Interrupts.
- **`xtask/`**: Needs updates to support `x86_64` targets and QEMU `q35` profiles.

## Constraints
- **Compatibility**: Must not break existing AArch64 (Pixel 6/Virt) support.
- **Memory**: NUC has 32GB RAM; page tables must handle large physical address spaces.
- **Boot**: Must adhere to Multiboot2 specification for GRUB/QEMU compatibility.

## Steps
### Step 1: Capture Feature Intent
- **Goal**: Define the scope of Intel NUC support.
- **Tasks**: Document hardware specs and initial boot goals.

### Step 2: Analyze Current State & Gaps
- **Goal**: Identify exactly which lines of code are arch-locked.
- **Tasks**: Audit `los_hal` and `kernel/src/init.rs` for ARM assumptions.

### Step 3: Source Code Reconnaissance
- **Goal**: Identify seams for abstraction.
- **Tasks**: Define traits for `InterruptController`, `Timer`, and `MmuInterface`.
