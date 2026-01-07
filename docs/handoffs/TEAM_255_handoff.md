# Handoff: Team 255 — Multi-Arch Structural Foundation

## Objective
Decouple the kernel from AArch64-specific logic and prepare the structural foundation for x86_64 implementation.

## Status: 🚧 IN PROGRESS (NON-COMPILING)
The structural refactoring is ~90% complete, but the kernel is currently in a non-compiling state due to visibility and import errors introduced during the migration of platform-specific types.

### Completed
1.  **xtask Refactor**:
    - Added global `--arch` flag (defaults to `aarch64`).
    - Logic generalized for `x86_64-unknown-none` target and `qemu-system-x86_64`.
    - Added `X86_64` profile using `q35` machine.
2.  **HAL Trait Extraction**:
    - Created `crates/hal/src/traits.rs` with `InterruptController`, `MmuInterface`, and `InterruptHandler`.
    - `los_hal` now exports these traits and `IrqId` from its root.
3.  **Arch Boundary Centralization**:
    - Moved `SyscallNumber`, `Stat`, `Timespec`, and `Termios` from generic modules to `kernel/src/arch/aarch64/mod.rs`.
    - Added stubs for these types in `kernel/src/arch/x86_64/mod.rs`.
    - `kernel/src/syscall/mod.rs` now imports these from `crate::arch`.
4.  **Task Abstraction**:
    - Generic `task/mod.rs` now uses `crate::arch::cpu::wait_for_interrupt()` instead of hardcoded `wfi` assembly.

### Blockers / Critical Issues
The next team MUST resolve these compilation errors:

1.  **Visibility Errors**:
    - `SyscallNumber`, `Stat`, `Timespec`, `Termios`, and `is_svc_exception` in `kernel/src/arch/aarch64/mod.rs` may need explicit `pub` access if they aren't visible to `syscall/mod.rs`.
2.  **TTY Conflict**:
    - `kernel/src/fs/tty/mod.rs` has a conflict between the imported `Termios` and a local definition or redundant re-export. Lines 1-20 need audit.
3.  **Import Paths**:
    - `init.rs` and `input.rs` fail to find `IrqId`. It should be imported as `los_hal::IrqId` (which is re-exported in `crates/hal/src/lib.rs`).
4.  **GIC Trait Mismatch**:
    - `crates/hal/src/gic.rs` had signature mismatches for `register_handler` and `map_irq` due to `IrqId` type ambiguity. Ensure it uses `crate::traits::IrqId`.

## Next Steps for Team 256
1.  **Fix Compilation**: Address the 5-10 errors reported by `cargo check -p levitate-kernel --target aarch64-unknown-none`.
2.  **Verify AArch64 Baseline**: Once compiling, run `cargo xtask test --arch aarch64` to ensure no behavioral regressions.
3.  **Start x86_64 implementation**: Begin filling in `kernel/src/arch/x86_64/boot.rs` with Multiboot2 logic.

# TEAM_255: Multi-arch structural foundation established.
