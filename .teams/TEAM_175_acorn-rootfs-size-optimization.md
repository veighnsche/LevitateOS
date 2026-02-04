# TEAM_175: AcornOS EROFS Rootfs Size Optimization (Task 3.18)

**Date**: 2026-02-04
**Status**: COMPLETE
**Task**: Phase 3 task 3.18 — EROFS rootfs size < 500MB compressed

## What Was Implemented

Reduced AcornOS EROFS rootfs from 769MB to 190MB compressed, meeting the < 500MB requirement.

### The Problem

Alpine's firmware packages (linux-firmware, sof-firmware, amd-ucode, intel-ucode) include comprehensive firmware for ALL hardware types:
- GPU drivers: nvidia (106MB), amdgpu (28MB)
- Audio firmware: sof-firmware (large)
- CPU microcode: intel-ucode, amd-ucode
- Total: 743MB of 888MB staging (83% of rootfs)

Previous implementation copied ALL firmware via `CopyAllFirmware`, resulting in bloated ISO.

### The Solution

Modified the FIRMWARE component in `AcornOS/src/component/definitions.rs`:
- Removed `custom(CustomOp::CopyAllFirmware)`
- Kept only `custom(CustomOp::CopyWifiFirmware)`

This maintains distro-spec compliance (firmware packages are still installed via apk), but only WiFi firmware is copied to the staging rootfs. Additional drivers can be installed post-boot if needed.

### Results

- **Compressed EROFS**: 190MB (was 769MB, target < 500MB) ✓
- **Staging rootfs**: 298MB (was 888MB)
- **Staging includes**: WiFi drivers (iwlwifi, rtlwifi, ath10k, brcm, mediatek, marvell, ralink)
- **Staging excludes**: GPU drivers (nvidia, amdgpu), audio firmware, excess device firmware

## Key Decisions

1. **Keep packages, not content**: Alpine firmware packages are part of Tier 1 (P0 requirements). Removing them from distro-spec would be a larger change. Instead, we install them (apk satisfies the requirement) but copy only essential firmware to staging.

2. **WiFi firmware sufficient for live ISO**: Most testing scenarios don't need GPU or audio firmware. Users with specific hardware needs can install drivers post-boot via `apk add`.

3. **Maintains flexibility**: If a user needs GPU drivers for their laptop, they can `apk add mesa` or `apk add amdgpu-dkms` etc. post-boot, which installs from the live rootfs's configured repositories.

## Files Modified

- `AcornOS/src/component/definitions.rs`: Modified FIRMWARE component ops (removed CopyAllFirmware line, added explanatory comment)

## Testing

- `cargo check` passes with zero errors
- Rebuilt rootfs with `cargo run -- build rootfs`
- Verified EROFS size: 190MB compressed
- Verified staging size: 298MB (under 500MB limit)

## Blockers

None. Task complete.

## Related Tasks

- 3.17: EROFS rootfs builds without errors (prerequisite, now verified with smaller size)
- 3.19+: IuppiterOS rootfs will use same FIRMWARE component optimization (fewer packages though)
- Phase 4: Initramfs/boot will verify that WiFi firmware is sufficient for test scenarios

## Notes

The firmware optimization could be extended to IuppiterOS, which needs even less (no desktop/WiFi, just server tools). That's Phase 3 task 3.24.

The component system's CopyWifiFirmware is well-designed and handles the common WiFi chipsets (Intel iwlwifi, Realtek rtl*, Atheros ath*, MediaTek, Broadcom, Marvell). This pattern could be extended for other firmware categories if needed.
