# TEAM_186 — IuppiterOS UKI Building

**Date**: 2026-02-04 (Iteration 24)
**Status**: Complete
**Tasks**: 5.5-5.6

## What Was Implemented

Implemented Unified Kernel Image (UKI) building for IuppiterOS with serial console primary (headless appliance).

### Changes Made

1. **IuppiterOS/src/artifact/uki.rs** (new file)
   - `build_uki()` - wraps recuki library
   - `build_live_ukis()` - creates 3 live UKI entries (normal, emergency, debug)
   - `build_installed_ukis()` - creates 2 installed UKI entries (normal, recovery)
   - All entries include `console=ttyS0,115200n8` per distro-spec
   - 4 unit tests verify serial console in all entries

2. **IuppiterOS/src/artifact/mod.rs**
   - Added `pub mod uki;` to include the new module

3. **IuppiterOS/src/artifact/iso.rs**
   - Added `use distro_spec::iuppiter::UKI_EFI_DIR;`
   - Added `use super::uki::build_live_ukis;`
   - Integrated UKI building as Stage 4.5 (between artifacts copy and UEFI boot setup)
   - Fixed GRUB config to use serial-only (no VGA for headless appliance)
   - Removed VGA_CONSOLE references

4. **IuppiterOS/Cargo.toml**
   - Added `recuki = { path = "../tools/recuki" }` dependency

## Key Design Decisions

### Serial Console Primary
- IuppiterOS is a headless appliance (no display)
- All UKI entries use `console=ttyS0,115200n8` as primary
- GRUB config uses serial-only (no VGA fallback)
- This differs from AcornOS which has VGA as first console for GUI use

### Live vs Installed Boot
- Live UKIs use `root=LABEL=IUPPITER` (ISO label for live boot)
- Installed UKIs use `root=LABEL=root rw` (for disk installations)
- Both follow the same pattern as AcornOS

### Testing
- 4 new unit tests verify:
  1. Base cmdline format includes IUPPITER label
  2. All live UKI entries defined (3 expected)
  3. All installed UKI entries defined (2 expected)
  4. Serial console (`console=ttyS0`) present in all entries

## Files Modified

- IuppiterOS/src/artifact/uki.rs (new: 170 lines)
- IuppiterOS/src/artifact/mod.rs (1 line added)
- IuppiterOS/src/artifact/iso.rs (16 lines changed)
- IuppiterOS/Cargo.toml (1 line added)

## Verification

```bash
cd IuppiterOS
cargo check              # ✓ Compiles cleanly
cargo test --lib        # ✓ All 22 tests pass (4 UKI tests included)
```

## Blockers / Known Issues

None. All tests pass, UKI integration complete.

## Next Tasks

- 5.7: ISO label verification (already correct, just needs verification)
- 5.8: ISO build via reciso + xorriso (requires build artifacts)
- 5.9: `cargo run -- run --serial` QEMU support

## Implementation Notes

The UKI building follows the same pattern established by AcornOS:
- One module file (`uki.rs`) with three public functions
- Unit tests validate entry definitions and cmdline format
- Integration into ISO build happens at Stage 4.5 (after artifacts, before UEFI setup)
- All constants pulled from distro-spec (no hardcoding)

The only significant difference from AcornOS is the removal of VGA_CONSOLE since IuppiterOS is explicitly a headless appliance.
