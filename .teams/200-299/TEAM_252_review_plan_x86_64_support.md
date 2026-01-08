# Team 252 - Review Plan x86_64 Support

## Objective
Review and refine the x86_64 support plan to ensure architectural alignment, completeness, and adherence to global rules.

## Status
- [x] Register Team 252
- [ ] Locate and read plan files
- [ ] Audit questions and answers

## Findings

### Decentralized AArch64 Conditional Code
I have identified several areas where AArch64-specific logic is "leaking" into generic kernel modules. These should be centralized in `kernel/src/arch/` or abstracted via `los_hal` traits.

1.  **`kernel/src/loader/elf.rs`**: Hardcoded machine type `EM_AARCH64` (183). The loader should be agnostic and query the architecture for the expected machine type.
2.  **`kernel/src/syscall/mod.rs`**: 
    - `SyscallNumber` enum uses Linux AArch64 numbers directly.
    - `Stat` struct layout matches AArch64.
    - `EC_SVC_AARCH64` (0b010101) is defined and used for exception class checking.
3.  **`kernel/src/task/mod.rs`**: `run_task` loop contains `#[cfg(target_arch = "aarch64")]` with a fallback to `spin_loop()`. This should be a `cpu::idle()` abstraction.
4.  **`kernel/src/fs/tty/mod.rs`**: `Termios` structure is explicitly noted to match Linux AArch64 layout.
5.  **`kernel/src/task/user.rs`**: Documentation and logic assume AArch64-specific registers/EL levels.
6.  **`kernel/src/init.rs`**: High coupling with AArch64/GIC/FDT.
    - Explicitly imports `los_hal::gic` and `los_hal::fdt`.
    - Hardcoded GIC registration for `VirtualTimer` and `Uart`.
    - `dtb_phys` discovery via memory scanning (AArch64 specific).
    - `initrd` range extraction from FDT.
7.  **`kernel/src/input.rs`**: Imports `los_hal::gic` and registers `VirtioInput` IRQ via `gic` module.
8.  **`kernel/src/syscall/signal.rs`**: Directly accesses `task.ttbr0`.
9.  **`crates/hal/src/mmu.rs`**: Contains AArch64-specific constants (GIC, UART, VirtIO base addresses) and bitflags (`AP_RW_EL1`, `MAIR`, etc.).
10. **`xtask/src/`**: Build and run logic is strictly AArch64/QEMU-virt focused (hardcoded `qemu-system-aarch64`, `aarch64-unknown-none`, `aarch64-linux-gnu-objcopy`).

### Plan Review Findings
- **Phase 1**: Comprehensive, but lacks explicit verification of `cpio`/`tar` endianness requirements mentioned in Phase 4.
- **Phase 2**: HAL traits are well-defined. Recommendation: Ensure `SyscallFrame` trait also handles argument retrieval which differs significantly between architectures.
- **Phase 3**: Good breakdown.
- **Phase 4**: Crucial for behavioral regression.
- **Phase 5**: Cleanup tasks are present.

### Phase 6 — Final Refinements and Handoff

### Final Review Summary
The x86_64 support plan is architecturally sound and follows the necessary phases for a complex feature. The decentralized AArch64 code identified provides a clear "hit list" for the Phase 1 & 2 audit and abstraction tasks.

### Changes Made
- Identified 10 key areas of decentralized AArch64 code.
- Added recommendations for `init.rs` refactoring and `SyscallFrame` trait expansion.
- Verified Multiboot2 decision is documented and consistent.

### Remaining Risks
- **Endianness**: Phase 4 mentions `cpio`/`tar` endianness; since x86_64 is little-endian (same as AArch64), this is likely a non-issue, but should be verified for metadata structures.
- **Hardware Variation**: The Intel NUC (7th Gen) is the target, but initial development will be in QEMU. Bare-metal transition may reveal NUC-specific quirks (e.g., NVMe, newer APIC features).

## Handoff Checklist
- [x] Project builds cleanly (AArch64)
- [x] All tests pass (AArch64)
- [x] Behavioral regression tests pass
- [x] Team file updated
- [x] Remaining TODOs documented

### Recommendations
- Centralize `SyscallNumber` and `Stat` in `kernel/src/arch/`.
- Introduce a `MachineType` abstraction in the ELF loader.
- Move TTY `Termios` layout to architecture-specific modules if it differs on x86_64.
- **Refactor `kernel/src/init.rs`**: Create an `arch::init()` or `arch::discover_devices()` hook to hide GIC/FDT/DTB details from generic initialization logic.
- **Abstract IRQ Registration**: Move `los_hal::gic` usage behind an `InterruptController` trait as planned in Phase 2, and update `input.rs` and `init.rs` to use this trait.
- **MMU Abstraction**: `crates/hal/src/mmu.rs` needs to be split into a trait-based interface and arch-specific implementations (AArch64, x86_64).
- **Update `xtask`**: Generalize build and run commands to take an `--arch` parameter and use appropriate QEMU binaries and flags.
