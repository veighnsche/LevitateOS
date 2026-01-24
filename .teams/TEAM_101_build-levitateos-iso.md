# TEAM_101: Build LevitateOS ISO

## Objective
Build the latest `levitateos.iso` at `/home/vince/Projects/LevitateOS/leviso/output/levitateos.iso`.

## Plan
1. Check `leviso/README.md` for build instructions. (Done)
2. Execute the necessary command to build the ISO. (Done - `cargo run -- build`)
   - Fixed `preflight` check for `init_tiny` (updated to `init_tiny.template`).
   - Fixed `preflight` check for `recipe` binary path (updated to `tools/recipe`).
3. Verify the ISO exists at the requested path. (Done)

## Results
- Build succeeded.
- ISO created at `/home/vince/Projects/LevitateOS/leviso/output/levitateos.iso`.
- Size: 522MB.

## Warnings
- Hardware compatibility verification failed for 13 profiles (missing firmware files). This did not prevent ISO creation.

## Critical Analysis (NUC 7i3 Bare Metal)
We performed a deep code audit to assess confidence in bare metal booting.

**Confidence Rating:**
- **Boot to Shell:** High (90%).
- **Graphical/WiFi/Audio:** Zero (0%).

**Major Issues Found:**
1. **Critical Version Mismatch:**
   - The ISO includes a **Custom Kernel** (built from `linux` submodule, e.g., v6.12+).
   - The ISO includes **Rocky Linux Modules** (extracted from Rocky RPMs, e.g., v6.12.0-el8).
   - The Initramfs includes **Rocky Linux Modules**.
   - **Result:** The Custom Kernel will **reject** all modules due to version mismatch. No WiFi, No GPU, No Audio.

2. **Boot Success Factors:**
   - Fortunately, the Custom Kernel has `CONFIG_USB_STORAGE`, `CONFIG_SQUASHFS`, `CONFIG_OVERLAY_FS` set to `y` (Built-in).
   - This means `initramfs` DOES NOT need modules to mount the ISO and boot.
   - The system WILL boot to a prompt.
   - USB Keyboard will work (`CONFIG_USB_HID=y`).

3. **Missing Firmware:**
   - `amdgpu` firmware is missing (irrelevant for Intel NUC).
   - `iwlwifi` firmware IS present.

**Next Steps / Fixes Required:**
- Modify `initramfs` builder to copy modules from `output/staging` (Custom Kernel) instead of `downloads/rootfs` (Rocky).
- Modify `squashfs` builder to install modules from `output/staging` (Custom Kernel).
