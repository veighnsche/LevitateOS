# TEAM_260: HAL Crate Reorganization

## Summary
Reorganize `los_hal` to have a symmetrical architecture with `aarch64/` and `x86_64/` subdirectories, moving AArch64-specific root modules into the new subdirectory.

## Status
- [ ] Create `crates/hal/src/aarch64/` directory
- [ ] Move AArch64-specific modules (`gic.rs`, `mmu.rs`, `timer.rs`, `uart_pl011.rs`, `fdt.rs`)
- [ ] Create `crates/hal/src/aarch64/mod.rs`
- [ ] Refactor `crates/hal/src/interrupts.rs` into arch-specific parts
- [ ] Update `crates/hal/src/lib.rs` to use symmetrical modules
- [ ] Fix kernel imports

## Team Members
- TEAM_260 (Cascade)

## Decisions
- Use `#[cfg(target_arch = "...")]` at the module level in `lib.rs` to expose the correct arch module.
- Keep truly generic code (traits, allocator, console wrapper) at the root.
