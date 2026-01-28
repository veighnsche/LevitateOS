# TEAM_140: qcow2 VM Image Artifact

## Goal
Add a new artifact type to leviso that builds bootable qcow2 disk images for local VM use.

## Status: COMPLETE

## Design Decisions
| Decision | Choice | Rationale |
|----------|--------|-----------|
| Use case | Local VMs only | No cloud-init complexity |
| Root filesystem | ext4 | Simple, stable, already in distro-spec |
| Root password | Empty | Consistent with live ISO pattern |
| Loop device | `losetup -Pf` | Native partition support, no kpartx dependency |
| Build approach | Raw → convert to qcow2 | Simplest, avoids qemu-nbd/fuse complexity |
| Disk size | Configurable, default 256GB | Uses existing `QEMU_DISK_GB` constant |

## Files Created
- [x] `leviso/src/artifact/qcow2.rs` - Main implementation (~500 lines)
  - RAII guards: `LoopDeviceGuard`, `MountGuard`
  - `check_host_tools()` - verifies qemu-img, sfdisk, losetup, mkfs.*, bootctl, blkid
  - `build_qcow2()` - full build pipeline
  - `verify_qcow2()` - basic verification

## Files Modified
- [x] `distro-spec/src/shared/qemu.rs` - Added QCOW2_IMAGE_FILENAME, RAW_DISK_FILENAME constants
- [x] `distro-spec/src/shared/mod.rs` - Export new constants
- [x] `leviso/src/artifact/mod.rs` - Added module export and public API
- [x] `leviso/src/main.rs` - Added BuildTarget::Qcow2 enum variant with disk_size arg
- [x] `leviso/src/commands/build.rs` - Added BuildTarget::Qcow2 variant and build_qcow2_only() handler
- [x] `leviso/src/rebuild.rs` - Added qcow2_artifact definition and helper functions

## Usage
```bash
# Build rootfs first (if not already built)
cargo run -- build rootfs

# Build qcow2 with default 256GB size
cargo run -- build qcow2

# Build with custom size
cargo run -- build qcow2 --disk-size 64

# Boot the image
qemu-system-x86_64 -enable-kvm -m 4G -cpu host \
  -drive if=pflash,format=raw,readonly=on,file=/usr/share/edk2/ovmf/OVMF_CODE.fd \
  -drive file=output/levitateos.qcow2,format=qcow2 \
  -device virtio-vga -device virtio-net-pci,netdev=net0 \
  -netdev user,id=net0
```

## Build Steps (in order)
1. Check host tools availability
2. Create raw disk image with qemu-img
3. Partition with sfdisk (GPT: 1GB EFI + remaining root)
4. Setup loop device with `losetup -Pf --show`
5. Format partitions (vfat for EFI, ext4 for root)
6. Mount root, then EFI at root/boot
7. Extract EROFS rootfs via mount + cp -aT
8. Get partition UUIDs with blkid
9. Generate /etc/fstab
10. Install systemd-boot and boot entry
11. Copy kernel and initramfs to /boot
12. Remove existing SSH host keys (regenerated on first boot)
13. Enable services (NetworkManager, sshd, chronyd)
14. Unmount and detach loop device
15. Convert raw to qcow2 with compression
16. Remove raw disk

## Bugs Fixed (Round 2)
1. **Root password was locked** - Now sets empty password in /etc/shadow (like live ISO)
2. **fstab had wrong fsck pass for vfat** - vfat now has pass 0 (doesn't support fsck)
3. **Missing machine-id handling** - Creates empty machine-id for first-boot regeneration
4. **Missing hostname** - Sets default hostname from distro-spec
5. **Sleep instead of proper partition discovery** - Now uses partprobe + udevadm settle
6. **No sync before unmount** - Added sync before unmounting filesystems
7. **EROFS extraction didn't use RAII guard** - Now properly uses MountGuard
8. **Incorrect live initramfs filename in fallback** - Fixed to use INITRAMFS_LIVE_OUTPUT constant
9. **Broken symlinks in enable_services** - Now removes broken symlinks before creating new ones

## Implementation Statistics
- **File size**: 948 lines
- **Functions**: 19 (including 2 public: build_qcow2, verify_qcow2)
- **Unit tests**: 8 (all passing)
- **RAII guards**: 2 (LoopDeviceGuard, MountGuard)

## Progress Log
- 2026-01-28: Team file created, implementation started
- 2026-01-28: Implementation complete, all tests pass
- 2026-01-28: Bug review and fixes applied, 8 unit tests now pass
- 2026-01-28: All 110 leviso tests pass (39+35+17+19 unit + 17 integration)
