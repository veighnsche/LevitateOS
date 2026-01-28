# TEAM_142: Build QCOW2 (sudo-free) and Run rootfs-tests

## Goal
Build the LevitateOS qcow2 VM image without sudo and run the rootfs-tests suite.

## Status: BLOCKED - Initramfs udevd fails to create /dev/disk/by-partuuid symlinks

## Completed Work

### Phase 1: Sudo-free QCOW2 Builder
- [x] Redesigned qcow2.rs to not require sudo
- [x] Uses `mkfs.ext4 -d` to create populated ext4 from directory (no mount needed)
- [x] Uses `mtools` (mcopy, mmd) for EFI partition (no mount needed)
- [x] Uses `sfdisk` on regular files (no losetup)
- [x] Generates UUIDs upfront with `uuidgen`
- [x] Splices partition images into disk with `dd`
- [x] Converts to qcow2 with `qemu-img`

### Phase 2: OVMF Boot Fix
- [x] Identified that rootfs-tests wasn't using OVMF_VARS.fd
- [x] Updated `rootfs-tests/src/lib.rs` to use `find_ovmf_vars()` and `uefi_vars()`
- [x] Created temp copy of OVMF_VARS for each test (NVRAM is modified during boot)

### Phase 3: Serial Console Output
- [x] Updated `distro-spec/src/shared/boot.rs` to add `console=ttyS0,115200 console=tty0`
- [x] Removed `quiet` option to see boot output

### Phase 4: Preflight Simplification
- [x] Removed sudo-dependent fsdbg verification from preflight
- [x] Preflight now uses `qemu-img info` to validate qcow2 format (no sudo)

## Blocking Issue: udevd Not Creating /dev/disk/by-partuuid

The kernel boots successfully, systemd starts in the initramfs, but stalls waiting for the root device:
```
systemd[1]: Expecting device dev-disk-by\x2dpartuuid-xxx...
```

### What the boot output shows
```
[    0.730215] systemd[1]: Listening on systemd-journald.socket - Journal Sockets.
[    0.731482] systemd[1]: Reached target sockets.target - Socket Units.
[    0.733709] systemd[1]: Mounting dev-hugepages.mount - Huge Pages File System...
...
[    0.756119] systemd[1]: initrd-udevadm-cleanup-db.service: Deactivated successfully.
[    0.757237] systemd[1]: Finished initrd-udevadm-cleanup-db.service - Cleanup udev Database.
[    0.759325] systemd[1]: Finished kmod-static-nodes.service - Create List of Static Device Nodes.
...
[    0.788208] systemd[1]: Reached target local-fs.target - Local File Systems.
```

Then it stalls with no further output for 90 seconds.

### What's WORKING in the initramfs

Verified via extraction (`zcat initramfs-installed.img | cpio -idm`):

1. **systemd generators present** (`/usr/lib/systemd/system-generators/`):
   - systemd-fstab-generator (parses root= kernel param)
   - systemd-gpt-auto-generator
   - systemd-debug-generator

2. **udev rules present** (`/usr/lib/udev/rules.d/`, 100 files):
   - 60-persistent-storage.rules (creates /dev/disk/by-partuuid symlinks)
   - All standard device rules

3. **udev helper programs** (`/usr/lib/udev/`):
   - ata_id, scsi_id, cdrom_id, mtd_probe, v4l_id

4. **systemd unit symlinks** in `sysinit.target.wants/`:
   - systemd-udevd.service -> ../systemd-udevd.service
   - systemd-udev-trigger.service -> ../systemd-udev-trigger.service

5. **Sockets** in `sockets.target.wants/`:
   - systemd-udevd-control.socket
   - systemd-udevd-kernel.socket

6. **Libraries**:
   - libblkid.so.1 (needed for partition UUID probing)
   - libuuid.so.1 (UUID handling)

7. **Virtio drivers** are built-in (from `modules.builtin`):
   - virtio.ko, virtio_blk.ko, virtio_pci.ko - all statically compiled

### What's NOT WORKING

Despite having all the right components:
- udevd starts (because initrd-udevadm-cleanup-db runs, which only runs after udevd)
- But `/dev/disk/by-partuuid/` symlinks are NOT being created
- This causes systemd to wait forever for the root device

### Hypotheses (for next team)

1. **udevd silent failure**: udevd may be starting but failing to process devices.
   Check with: add `systemd.log_level=debug` to kernel cmdline

2. **Missing udev binary dependency**: udevadm/systemd-udevd may be missing a library.
   Test: Run `ldd usr/lib/systemd/systemd-udevd` in extracted initramfs

3. **ID_PART_ENTRY_UUID not being set**: The 60-persistent-storage.rules rely on this env var.
   The var is set by libblkid during device probing. Might be a libblkid issue.

4. **GPT partition not being recognized**: The qcow2 builder creates GPT partitions via sfdisk.
   The partition UUID is set via `uuid=<uuid>` in sfdisk script.
   Test: Verify partition UUID is set by reading GPT in the qcow2 image

5. **blkid not working in initramfs**: libblkid needs magic files or config.
   Check if `/etc/blkid.conf` or `/etc/blkid.tab` are needed

6. **Device not appearing at all**: Maybe virtio-blk not initializing properly.
   Check: Look for /dev/vda in the boot log (it should appear very early)

### Debugging approach for next team

1. **Add debug output to kernel cmdline**:
   ```
   root=PARTUUID=xxx rw console=ttyS0,115200 systemd.log_level=debug rd.debug
   ```

2. **Check if /dev/vda appears**:
   Boot manually and run `ls /dev/vd*` in emergency shell

3. **Manually run udevadm**:
   ```bash
   udevadm info --query=env --name=/dev/vda
   udevadm trigger
   udevadm settle
   ls -la /dev/disk/by-partuuid/
   ```

4. **Check if blkid works**:
   ```bash
   blkid /dev/vda2
   ```

5. **Verify GPT UUIDs in qcow2**:
   ```bash
   qemu-nbd -c /dev/nbd0 output/levitateos.qcow2
   sgdisk -i 2 /dev/nbd0  # Shows partition 2 GUID
   qemu-nbd -d /dev/nbd0
   ```

## Files Modified
- `leviso/src/artifact/qcow2.rs` - Complete rewrite for sudo-free build
- `testing/rootfs-tests/src/lib.rs` - Added OVMF_VARS support
- `testing/rootfs-tests/src/preflight.rs` - Removed sudo-dependent fsdbg
- `testing/rootfs-tests/Cargo.toml` - Added tempfile dependency
- `distro-spec/src/shared/boot.rs` - Added serial console options

## recinit Analysis

The initramfs is built by `tools/recinit`:

### Key files:
- `tools/recinit/src/install.rs` - Build logic for install initramfs
- `tools/recinit/src/systemd.rs` - Copies systemd, udev, generators

### Install initramfs architecture:
1. Uses systemd as init (`/init -> /usr/lib/systemd/systemd`)
2. Includes systemd generators for parsing root= parameter
3. Includes all udev rules (copied recursively)
4. Patches udev units to remove `ConditionPathIsReadWrite=/sys`
5. Creates /etc/initrd-release (required for systemd to recognize initrd)
6. Creates /etc/passwd, /etc/group (required for udevd permissions)
7. Creates /etc/nsswitch.conf (required for glibc lookups)

### What recinit DOES copy:
- systemd-fstab-generator, systemd-gpt-auto-generator
- All files from /usr/lib/udev/rules.d/
- udev helpers (ata_id, scsi_id, etc.)
- libblkid.so.1, libuuid.so.1

### Potential recinit gaps:
- May be missing hwdb (hardware database for udev)
- May need firmware loading support
- Might need explicit handling for virtio devices

## Commands

```bash
# Build qcow2 (no sudo needed!)
cargo run -p leviso -- build qcow2 --disk-size 8

# Copy to expected location
cp leviso/output/levitateos.qcow2 output/

# Run tests (single-threaded to avoid qcow2 lock contention)
cargo test -p rootfs-tests -- --ignored --test-threads=1

# Inspect initramfs contents
mkdir -p /tmp/initramfs-inspect && cd /tmp/initramfs-inspect
zcat /path/to/initramfs-installed.img | cpio -idmv
ls -la usr/lib/udev/rules.d/
cat usr/lib/systemd/system/systemd-udevd.service

# Manual boot test with debug
cp /usr/share/edk2/ovmf/OVMF_VARS.fd /tmp/ovmf_vars.fd
qemu-system-x86_64 -enable-kvm -m 4G -cpu host \
  -drive if=pflash,format=raw,readonly=on,file=/usr/share/edk2/ovmf/OVMF_CODE.fd \
  -drive if=pflash,format=raw,file=/tmp/ovmf_vars.fd \
  -drive file=output/levitateos.qcow2,format=qcow2,if=virtio \
  -nographic -no-reboot
```

## Timeline
- 2026-01-27: Sudo-free qcow2 builder complete
- 2026-01-28: OVMF boot fix, serial console, debugging initramfs
- Current: Blocked on udevd not creating device symlinks
