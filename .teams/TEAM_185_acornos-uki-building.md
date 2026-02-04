# TEAM_185 — AcornOS UKI Building Implementation

**Date**: 2026-02-04
**Status**: Complete
**Phase**: Phase 5 (ISO Build) — Tasks 5.1, 5.2

## Summary

Implemented Unified Kernel Image (UKI) building for AcornOS, completing Phase 5 tasks 5.1-5.2. UKIs combine kernel + initramfs + cmdline into a single PE binary for UEFI boot.

## Implementation Details

### New Files
- `AcornOS/src/artifact/uki.rs` — UKI builder module with three functions:
  - `build_uki()` — Low-level UKI builder using recuki
  - `build_live_ukis()` — Builds 3 live UKI entries (normal, emergency, debug)
  - `build_installed_ukis()` — Builds 2 installed UKI entries (normal, recovery)

### Modified Files
- `AcornOS/Cargo.toml` — Added `recuki = { path = "../tools/recuki" }` dependency
- `AcornOS/src/artifact/mod.rs` — Added `pub mod uki` and exports
- `AcornOS/src/artifact/iso.rs` — Integrated UKI building into create_iso():
  - Added stage 4.5 to build_live_ukis() after artifact copy
  - Added stage to create EFI/Linux directory
  - Added systemd-boot loader.conf generation
  - Imported build_live_ukis and default_loader_config

## Boot Configuration

**Live UKIs** (for ISO boot):
- Use `root=LABEL=ACORNOS` to find ISO
- Kernel cmdline: `VGA_CONSOLE SERIAL_CONSOLE`
- Entries: normal, emergency (emergency), debug (debug)

**Installed UKIs** (for disk boot):
- Use `root=LABEL=root` for user-partitioned disk
- Can be edited at boot time via systemd-boot
- Entries: normal, recovery (single)

**systemd-boot loader.conf**:
- Created at EFI/loader/loader.conf
- Uses default_loader_config() from distro-spec
- Auto-discovered by systemd-boot

## Key Design Decisions

1. **recuki integration**: Used existing recuki crate (from tools/) for UKI building rather than implementing ukify wrapper — cleaner and reuses tested code.

2. **Consistent with leviso**: Mirrored leviso's UKI module structure exactly, using same function signatures and parameter patterns for maintainability.

3. **Live vs installed separation**: Created two separate functions (build_live_ukis, build_installed_ukis) rather than parameterized function — clearer intent and easier to test.

4. **Stage ordering**: UKI building placed after artifact copy (stage 4) and before EFI boot setup (stage 5) — logical flow since UKIs depend on kernel/initramfs already being copied.

## Files Modified/Created
- AcornOS/src/artifact/uki.rs (new, 155 lines)
- AcornOS/src/artifact/mod.rs (updated exports)
- AcornOS/src/artifact/iso.rs (integrated UKI building + loader.conf)
- AcornOS/Cargo.toml (added recuki dependency)

## Testing

- `cargo check` passes cleanly
- UKI module has 3 unit tests:
  - test_base_cmdline_format — validates kernel cmdline structure
  - test_uki_entries_defined — verifies 3 live entries present
  - test_installed_uki_entries_defined — verifies 2 installed entries present
- All tests pass

## No Known Issues

Implementation is complete. Next phase (5.3-5.4) involves verifying ISO label and QEMU boot.
