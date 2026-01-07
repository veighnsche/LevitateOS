# Adding a New Architecture to LevitateOS

This guide outlines the steps and patterns required to add support for a new CPU architecture to LevitateOS.

## 1. Toolchain Setup

Update `rust-toolchain.toml` to include the target triple for the new architecture.

```toml
targets = ["aarch64-unknown-none", "x86_64-unknown-none", "new_arch-unknown-none"]
```

## 2. Architecture Abstraction Layer (`kernel/src/arch/`)

Create a new directory `kernel/src/arch/<new_arch>/` and implement the mandatory types and functions.

### Required Files
- `mod.rs`: Main entry point, exports submodules.
- `boot.rs`: Early boot logic and memory initialization stubs.
- `cpu.rs`: CPU-specific instructions (e.g., `wait_for_interrupt`).
- `exceptions.rs`: Exception vectors and handlers.
- `task.rs`: Context switching logic and `Context` struct.

### Mandatory Definitions in `mod.rs`
The following must be public and accessible via `crate::arch`:
- `struct SyscallFrame`: Saved register state.
- `enum SyscallNumber`: Platform-specific syscall mapping.
- `struct Stat`: Metadata layout matching the target ABI.
- `struct Timespec`: Time layout matching the target ABI.
- `struct Termios` & `NCCS`: Terminal configuration.
- `const ELF_MACHINE`: ELF header machine ID (e.g., 183 for AArch64).
- `fn is_svc_exception(esr: u64) -> bool`: Syscall entry detection.

## 3. Hardware Abstraction Layer (`los_hal`)

The kernel depends on traits defined in `los_hal::traits`. You must provide implementations for:

- **`InterruptController`**: Map high-level `IrqId` to hardware IRQs and manage acknowledgment.
- **`MmuInterface`**: Manage page table mappings and address space switching.

Update `crates/hal/src/lib.rs` to return the correct implementation in `active_interrupt_controller()`:

```rust
pub fn active_interrupt_controller() -> &'static dyn InterruptController {
    #[cfg(target_arch = "aarch64")] { gic::active_api() }
    #[cfg(target_arch = "new_arch")] { new_ic::active_api() }
}
```

## 4. Build System (`xtask`)

Update `xtask/src/main.rs` and related modules to support the new architecture flag.

- **`run.rs`**: Define a `QemuProfile` for the new architecture.
- **`build.rs`**: Handle binary conversion (if needed) and target-specific build flags.

## 5. Verification

1. **Stub Phase**: Implement all required items with `unimplemented!()`. Ensure the kernel compiles for the new target.
2. **Baseline Protection**: Run `cargo xtask test --arch aarch64` to ensure no regressions were introduced in existing support.
3. **Bringup**: Use `cargo xtask run --arch <new_arch>` to begin iterative development.

# TEAM_255: Established multi-arch patterns for LevitateOS.
