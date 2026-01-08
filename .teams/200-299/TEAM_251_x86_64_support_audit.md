# Team 251 - x86_64 Support Audit

## Task
Audit architecture abstractions between AArch64 and x86_64 to prepare for Intel NUC target support. Update documentation to reflect the new hardware target.

## Target Hardware
- **Model**: Intel NUC (x86_64 i3 7th Gen)
- **RAM**: 32GB
- **Storage**: 1TB NVMe

## Progress
- [x] Claimed team number 251.
- [x] Audit `kernel/src/arch/` abstractions.
- [x] Audit `crates/hal/` and other HAL-related crates.
- [x] Update `README.md` and `docs/` with the new target.
- [x] Identify gaps in x86_64 support.
- [x] Initialize x86_64 Support Feature Plan (Phases 1-5).

## Findings & Recommendations
...
(Existing findings)
...

## Planning Artifacts
- **Phase 1 (Discovery)**: `docs/planning/x86_64-support/phases/phase-1.md`
- **Phase 2 (Design)**: `docs/planning/x86_64-support/phases/phase-2.md`
- **Phase 3 (Implementation)**: `docs/planning/x86_64-support/phases/phase-3.md`
- **Phase 4 (Integration)**: `docs/planning/x86_64-support/phases/phase-4.md`
- **Phase 5 (Polish)**: `docs/planning/x86_64-support/phases/phase-5.md`

### 1. Architecture Abstractions
The kernel has a clean split between `aarch64` and `x86_64` in `kernel/src/arch/mod.rs`. Most core logic (scheduling, VFS, syscall dispatch) is architecture-independent.

### 2. HAL Readiness
The `los_hal` crate needs significant work to support x86_64:
- **MMU**: Currently hardcoded for AArch64 (TTBR0/TTBR1). Needs a trait-based approach or `#[cfg]` for PML4/CR3.
- **Interrupts**: `gic.rs` is AArch64-specific. An APIC implementation is required for x86_64.
- **Timer**: `timer.rs` uses AArch64 generic timers. PIT/APIC Timer support needed.
- **Console**: `uart_pl011.rs` is ARM-specific. VGA/COM1 support needed for x86_64.

### 3. Missing Drivers
- **NVMe**: The Intel NUC uses NVMe storage. No NVMe driver currently exists.
- **APIC**: Required for interrupt handling on x86_64.

### 4. Boot Protocol
AArch64 uses FDT/PSCI. x86_64 will require Multiboot2/UEFI support as documented in `docs/design/x86_64_support.md`.

## Handoff Notes
- All root `README.md` files have been updated to include the Intel NUC target.
- `docs/design/x86_64_support.md` now includes specific hardware specs for the NUC.
- Next team should focus on implementing the Multiboot2 entry point and a basic VGA/Serial logger for x86_64.
