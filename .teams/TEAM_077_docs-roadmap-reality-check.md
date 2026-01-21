# TEAM_077: Docs and Roadmap Reality Check

## Objective
Fix the roadmap and documentation to be realistic, bug-free, and assumption-free by testing actual LevitateOS installation on QEMU hardware disks.

## Status: COMPLETE

## Testing Results

### What WORKS in Live Environment
- [x] **Boot to systemd** - Systemd runs as PID 1, shows multi-user target
- [x] **lsblk** - Lists block devices correctly
- [x] **blkid** - Shows UUIDs
- [x] **fdisk** - Works (util-linux 2.40.2)
- [x] **parted** - Creates GPT, partitions (GNU parted 3.6)
- [x] **wipefs** - Works
- [x] **mkfs.ext4** - Formats ext4 (mke2fs 1.47.1)
- [x] **mkfs.fat** - Formats FAT32
- [x] **mount/umount** - Mounts work correctly
- [x] **Basic coreutils** - ls, cp, mv, cat, mkdir, etc.
- [x] **tar, gzip, xz** - Archive/compression
- [x] **chroot** - Enter new system
- [x] **sr0 CDROM** - Installation media accessible via `/dev/sr0`
- [x] **Base tarball** - `mount /dev/sr0 /media/cdrom` then access `levitateos-base.tar.xz`

### What's Still Missing
- [ ] **nano** - NOT available (using heredocs as workaround in docs)
- [ ] **passwd** - NOT available (use `chpasswd` instead)
- [ ] **locale-gen** - NOT available
- [ ] **recipe** - NOT available (no package manager yet)

## Final Architecture

```
ISO boots → Initramfs (live environment)
                ↓
            /dev/sr0 (virtio-scsi CDROM) contains ISO contents
                ↓
            mount /dev/sr0 /media/cdrom
                ↓
            /media/cdrom/levitateos-base.tar.xz accessible!
```

**Key fix:** Changed QEMU from IDE CDROM (`-cdrom`) to virtio-scsi CDROM:
- Added `virtio-scsi-pci` controller
- Attached ISO as `scsi-cd` device
- Added kernel modules: `virtio_scsi`, `cdrom`, `sr_mod`, `isofs`

## Changes Made

### QEMU (`src/qemu.rs`)
1. Added ISO to `test_direct()` function - both `test` and `run` commands now have CDROM
2. Changed CDROM from IDE (`-cdrom`) to virtio-scsi for better kernel compatibility
3. Updated `run_interactive()` and `run_with_command()` to accept ISO path

### Kernel Modules (`src/initramfs/modules.rs`)
1. Added `virtio_scsi.ko.xz` - virtio SCSI host controller
2. Added `cdrom.ko.xz` - generic CDROM support
3. Added `sr_mod.ko.xz` - SCSI CDROM driver (creates /dev/sr0)
4. Added `isofs.ko.xz` - ISO 9660 filesystem
5. Removed old IDE modules (ata_piix, ata_generic) - not needed

### Init Script (`profile/init`)
1. Updated module loading to use `virtio_scsi` instead of ATA/IDE modules

### Roadmap (`leviso/ROADMAP.md`)
1. Marked Phase 2 (Systemd Init) as complete
2. Marked Phase 3.5 (Base Tarball Access) as complete
3. Updated testing section with full working installation workflow

### Docs-content
1. `02-installation.ts` - Removed "not working" status note
2. `04-installation-base.ts` - Added step 6 (mount CDROM), renumbered to steps 6-9
3. `05-installation-config.ts` - Replaced `nano` with heredocs
4. `06-installation-boot.ts` - Replaced `nano` with heredocs

## Verified Working Installation Flow

```bash
# 1. Disk preparation
parted -s /dev/vda mklabel gpt
parted -s /dev/vda mkpart EFI fat32 1MiB 513MiB
parted -s /dev/vda set 1 esp on
parted -s /dev/vda mkpart root ext4 513MiB 100%
mkfs.fat -F32 /dev/vda1
mkfs.ext4 -F /dev/vda2
mount /dev/vda2 /mnt
mkdir -p /mnt/boot
mount /dev/vda1 /mnt/boot

# 2. Mount installation media
mkdir -p /media/cdrom
mount /dev/sr0 /media/cdrom

# 3. Extract base system
tar xpf /media/cdrom/levitateos-base.tar.xz -C /mnt
```
