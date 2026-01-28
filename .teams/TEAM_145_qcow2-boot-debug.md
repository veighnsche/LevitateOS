# TEAM_145: qcow2 Boot Debugging

## Status: COMPLETE ✓

## Summary
Successfully fixed all qcow2 boot issues. The qcow2 image now boots to login prompt.

**Verified:** 2026-01-28
- Initramfs verification: 150/150 checks pass
- Boot test: Kernel → systemd → initrd.target → switch-root → login prompt

## Root Causes Found & Fixed

### 1. Missing PATH and LD_LIBRARY_PATH in init script
**File:** `tools/recinit/src/install.rs`
**Problem:** The init wrapper script didn't set PATH or LD_LIBRARY_PATH. The kernel doesn't provide these when running init as PID 1.
**Fix:** Added exports to init script:
```bash
export PATH=/usr/bin:/bin:/usr/sbin:/sbin
export LD_LIBRARY_PATH=/usr/lib64:/usr/lib64/systemd:/lib64
```

### 2. systemd-tmpfiles in wrong location constant
**File:** `distro-spec/src/shared/components.rs`
**Problem:** `systemd-tmpfiles` was listed in `SYSTEMD_BINARIES` (for `/usr/lib/systemd/`) but the binary is actually at `/usr/bin/systemd-tmpfiles`. The copy silently failed.
**Fix:**
- Removed from `SYSTEMD_BINARIES`
- Added to `BIN_UTILS` under the SYSTEMD section

### 3. Busybox shell for early boot
**File:** `tools/recinit/src/install.rs`
**Problem:** The init script used `/bin/bash` which is dynamically linked and fails before the dynamic linker is set up.
**Fix:** Extract busybox from live initramfs and use `#!/usr/bin/busybox sh` for the init wrapper. Replace `/bin/sh` symlink to point to busybox.
**Note:** Bash is intentionally REMOVED from initramfs (dynamically linked shell won't work in early boot).

### 4. QEMU command line piping issue
**Problem:** Piping QEMU output to `head` causes premature exit and cryptic "drive with bus=0, unit=0 exists" errors.
**Fix:** Don't pipe QEMU output when testing. Use full output capture or timeout without piping.

### 5. Missing initrd units in ESSENTIAL_UNITS
**File:** `distro-spec/src/shared/components.rs`
**Problem:** The initrd units (initrd.target, initrd-switch-root.service, etc.) were not in ESSENTIAL_UNITS, so they weren't copied to rootfs-staging.
**Discovery:** Used `fsdbg verify --type install-initramfs` to identify missing files.
**Fix:** Added initrd targets and services to ESSENTIAL_UNITS:
- `initrd.target`, `initrd-root-fs.target`, `initrd-root-device.target`, `initrd-switch-root.target`, `initrd-fs.target`
- `initrd-switch-root.service`, `initrd-cleanup.service`, `initrd-udevadm-cleanup-db.service`, `initrd-parse-etc.service`

### 6. Missing systemd-makefs
**File:** `distro-spec/src/shared/components.rs`
**Problem:** `systemd-makefs` wasn't in SYSTEMD_BINARIES, so not copied to rootfs-staging.
**Fix:** Added `systemd-makefs` to SYSTEMD_BINARIES.

### 7. Improved build workflow ergonomics
**Files:** `leviso/src/commands/build.rs`, `leviso/src/rebuild.rs`
**Problem:** `build qcow2` didn't automatically rebuild initramfs when recinit sources changed.
**Fix:**
- Modified `build_qcow2_only()` to check `install_initramfs_needs_rebuild()` and rebuild if needed
- Updated `install_initramfs_artifact` to track `distro-spec/src/shared/components.rs` and `udev.rs`
- Updated `qcow2_artifact` to track initramfs-installed.img as input
- Now a single command works: `cargo run -- build qcow2 --disk-size 4`

### 8. Updated fsdbg verification checklist
**File:** `testing/fsdbg/src/checklist/install_initramfs.rs`
**Problem:** Checklist expected bash but we intentionally use busybox.
**Fix:** Updated BINARIES list to expect `usr/bin/busybox` and `usr/bin/sh` (symlink to busybox) instead of bash.
Removed `kmod` from expected list (doesn't exist separately on Rocky - modprobe IS the kmod binary).

## Files Modified

1. `tools/recinit/src/install.rs` - Init script with PATH/LD_LIBRARY_PATH, busybox integration, bash removal
2. `tools/recinit/src/systemd.rs` - Removed kmod and bash from SYSTEMD_FILES, added comments
3. `distro-spec/src/shared/components.rs` - Moved systemd-tmpfiles to BIN_UTILS, added initrd units, added systemd-makefs
4. `leviso/src/commands/build.rs` - Auto-rebuild initramfs in qcow2 build
5. `leviso/src/rebuild.rs` - Updated artifact tracking for initramfs and qcow2
6. `testing/fsdbg/src/checklist/install_initramfs.rs` - Updated expected binaries (busybox instead of bash)

## Build Command

```bash
cd leviso && cargo run --release -- build qcow2 --disk-size 4
```

This single command now:
1. Checks if rootfs-staging exists (bails if not)
2. Rebuilds initramfs if inputs changed
3. Builds the qcow2 image
4. Verifies the result

## Verification

```bash
# Verify initramfs
cargo run --release -p fsdbg -- verify --type install-initramfs output/initramfs-installed.img

# Boot test (don't pipe to head!)
timeout 60 qemu-system-x86_64 -m 4G -enable-kvm -cpu host \
  -bios /usr/share/edk2/ovmf/OVMF_CODE.fd \
  -hda output/levitateos.qcow2 \
  -nographic
```

## Key Learnings

1. **Silent failures are dangerous:** The `if src.exists()` pattern in copy loops silently skips missing files.

2. **Binary locations vary by distro:** `systemd-tmpfiles` is in `/usr/bin/` on Rocky/Fedora, not `/usr/lib/systemd/`.

3. **Busybox for early boot:** Statically linked busybox is essential for init scripts that run before the dynamic linker is operational. Bash cannot work in initramfs early boot.

4. **QEMU piping breaks things:** Never pipe QEMU serial output through `head` or similar - it causes SIGPIPE issues.

5. **Kernel doesn't set PATH:** When the kernel runs `/init`, there's no PATH or LD_LIBRARY_PATH set.

6. **kmod doesn't exist separately:** On Rocky Linux, modprobe IS the kmod binary - kmod doesn't exist as a separate file.

7. **Use fsdbg verify:** The `fsdbg verify --type install-initramfs` command is essential for debugging initramfs issues.
