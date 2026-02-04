# TEAM_170: AcornOS Tier 0-2 Package Installation Integration

**Date:** Iteration 14
**Status:** COMPLETE

## Summary

Integrated Tier 0-2 package installation into the AcornOS build pipeline. The packages.rhai recipe (which already existed) is now automatically executed during the download phase, installing all required packages from distro-spec before the EROFS rootfs build.

## What Was Implemented

Modified `AcornOS/src/main.rs:cmd_download_alpine()` to call `acornos::recipe::packages()` after Alpine base extraction. This ensures Tier 0-2 packages are installed into the rootfs before the rootfs build phase.

### Package Tiers Installed

**Tier 0 (Already handled by alpine.rhai):**
- alpine-base, openrc, openrc-init, linux-lts, grub, grub-efi, efibootmgr, e2fsprogs, dosfstools, util-linux

**Tier 1 (Core System):**
- eudev, eudev-openrc, linux-firmware, intel-ucode, amd-ucode
- cryptsetup, lvm2, btrfs-progs, device-mapper
- util-linux-login, bash, coreutils, doas, grep, sed, gawk, findutils

**Tier 2 (Daily Driver):**
- dhcpcd, iproute2, iputils, iwd, wireless-regdb
- ca-certificates, tzdata, chrony
- curl, less, vim, htop
- pciutils, usbutils, dmidecode, ethtool
- smartmontools, hdparm, nvme-cli
- openssh, openssh-server-common

## Files Modified

- `AcornOS/src/main.rs`: Added `acornos::recipe::packages()` call in `cmd_download_alpine()`

## Key Decisions

1. **Call packages() in download phase:** Makes semantic sense - download all dependencies before building
2. **packages.rhai already matches distro-spec:** No changes needed to the Rhai recipe, it already includes the correct package lists
3. **Recipe integration:** Uses existing infrastructure (recipe binary, JSON context parsing)

## Verification

- `cargo check` passes cleanly in AcornOS
- Commit: 167ba3f
- Task 3.14 marked complete in PRD

## Technical Details

The `packages.rhai` recipe (in AcornOS/deps/) uses apk-tools-static to install packages into the rootfs:
- Verifies alpine.rhai outputs exist (apk-tools binary, rootfs)
- Installs each tier in sequence via `apk --root <rootfs> add <packages>`
- Writes version and manifest files to track completion
- Verifies key binaries exist after installation

The recipe uses a version mechanism to invalidate cache when package lists change, allowing fast iteration when packages are modified.

## No Blockers

The existing packages.rhai recipe already handled all the required functionality. The fix was simply to invoke it during the download phase.

## Related Tasks

- Task 3.15: Test instrumentation shell marker (___SHELL_READY___)
- Task 3.16-3.18: EROFS rootfs compression and size validation
- Task 4+: Initramfs and boot phases depend on rootfs having all packages
