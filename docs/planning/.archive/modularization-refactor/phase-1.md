# Phase 1: Discovery and Safeguards - Modularization Refactor

**TEAM_372: Refactoring oversized modules for Rule 7 compliance.**

## Refactor Summary
The LevitateOS codebase has several modules that have grown beyond the ideal 500-line limit (Rule 7). These "god modules" often mix multiple responsibilities (e.g., `init.rs` handling both boot stages and the failsafe shell). This refactor aims to split these modules into smaller, more focused files and consolidate subsystem wrappers into a dedicated directory.

## Success Criteria
- [gic.rs](file:///home/vince/Projects/LevitateOS/crates/hal/src/aarch64/gic.rs) split into `hal/src/aarch64/gic/`
- [init.rs](file:///home/vince/Projects/LevitateOS/crates/kernel/src/init.rs) split into `kernel/src/init/`
- [process.rs](file:///home/vince/Projects/LevitateOS/crates/kernel/src/syscall/process.rs) split into `kernel/src/syscall/process/`
- [arch/x86_64/mod.rs](file:///home/vince/Projects/LevitateOS/crates/kernel/src/arch/x86_64/mod.rs) cleaned up.
- All files < 500 lines preferred.
- No regressions in boot behavior or syscall functionality.

## Behavioral Contracts
- **Boot Sequence**: Must complete all stages (EarlyHAL, MemoryMMU, BootConsole, Discovery) and spawn `init`.
- **Syscalls**: `sys_spawn`, `sys_waitpid`, `sys_clone`, `sys_arch_prctl` must continue to work exactly as before.
- **HAL Interface**: `InterruptController` trait implementations for GIC must remain compatible.

## Golden/Regression Tests
- `cargo xtask test behavior` -> Must pass.
- `cargo xtask run` -> Must reach shell on both architectures.
- `cargo xtask run-pixel6` -> Must reach shell.

## Current Architecture Notes
- `init.rs` is the main entry point for `kmain_unified`. It has deep dependencies on `fs`, `task`, `arch`, and `hal`.
- `gic.rs` contains both GICv2 and GICv3 logic, as well as the IRQ handler registry.
- `process.rs` contains all process-related syscalls, including threading and arguments parsing.

## Constraints
- **Zero Behavior Change**: This is a pure refactor. No logic should change.
- **Architecture Symmetry**: Refactoring must be done for both `aarch64` and `x86_64` where applicable.

## Open Questions
- None currently.

---

## Steps and UoWs

### Step 1: Baseline Verification
- [ ] Run `cargo xtask test` and record results.
- [ ] Verify both architectures boot to shell.

### Step 2: GIC Refactor (aarch64)
- [ ] Extract GICv2/v3 logic.
- [ ] Move IRQ registry to `handlers.rs`.

### Step 3: Kernel Init Refactor
- [ ] Extract Failsafe Shell.
- [ ] Extract Device Discovery.
- [ ] Extract FS Mount logic.

### Step 4: Syscall Process Refactor
- [ ] Extract Spawn/Exec.
- [ ] Extract Waitpid.
- [ ] Extract Threading/Clone.

### Step 5: Subsystem Reorganization
- [ ] Move gpu, net, input, block, virtio to `subsystems/`.
