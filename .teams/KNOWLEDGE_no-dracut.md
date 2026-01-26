# KNOWLEDGE: Why LevitateOS Does NOT Use Dracut

**Created:** 2026-01-26 (TEAM_125)
**Status:** Active - This is current architectural guidance

## The Problem with Dracut

1. **Requires root** - Uses chroot with bind mounts (/dev, /proc, /sys)
2. **Broken defaults** - `--kmoddir` flag doesn't work, still checks `/lib/modules/`
3. **External dependency** - Another tool to package, configure, debug
4. **Complexity** - Dracut modules, dependencies, omit lists

## What We Use Instead

Custom initramfs builder in `leviso/src/artifact/initramfs.rs`:

- `build_tiny_initramfs()` - Live ISO boot (busybox, mounts squashfs)
- `build_install_initramfs()` - Installed system boot (systemd, mounts disk)

Both use the same approach:
1. Create directory structure
2. Copy kernel modules from staging
3. Copy binaries + shared libraries (using leviso-elf)
4. Build CPIO archive with gzip compression

**NO root required. NO external tools. Full control.**

## Module Lists

- `BOOT_MODULES` - Live ISO boot (CDROM, squashfs, overlay, virtio)
- `INSTALL_BOOT_MODULES` - Installed system (NVMe, SATA, USB, ext4, xfs, btrfs, vfat)

Both defined in `distro-spec/src/shared/boot_modules.rs`.

## Historical Context

- **TEAM_030**: Original design used dracut (required root)
- **TEAM_107**: Moved dracut from install-time to build-time (still required root)
- **TEAM_125**: Removed dracut entirely, extended custom builder

## If You're Tempted to Use Dracut

**DON'T.** The custom builder:
- Already works (proven by live ISO boot)
- Doesn't require root
- Gives us full control
- Has no broken defaults to work around

If the custom builder is missing something, **extend it** rather than bringing back dracut.

## Key Files

| File | Purpose |
|------|---------|
| `leviso/src/artifact/initramfs.rs` | Both initramfs builders |
| `distro-spec/src/shared/boot_modules.rs` | Module lists |
| `leviso-elf/` | Library dependency resolution |

## What About Booster?

Booster was considered but:
- Not packaged for Fedora/Rocky
- Would need code modifications for external module dirs
- We already have a working solution

The custom builder is simpler, requires no external dependencies, and gives us full control.
