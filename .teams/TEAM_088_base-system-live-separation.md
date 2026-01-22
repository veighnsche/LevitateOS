# TEAM_088: Base System / Live Overlay Separation

**Status: COMPLETE**

## Problem

The squashfs contained **live-specific configs** that got extracted to installed systems:

| Config | File | Issue |
|--------|------|-------|
| Root autologin on tty1 | `console-autologin.service` | **SECURITY: No password required** |
| Root shell on ttyS0 | `serial-console.service` | Unnecessary on real hardware |
| Passwordless root | `/etc/shadow` has `root::` | **SECURITY: Empty password** |

When `recstrap` extracted the squashfs to disk, users got an insecure system.

## Solution Architecture

```
squashfs (350MB) - BASE SYSTEM ONLY
├── Base system
├── getty@tty1.service (normal login)
├── /etc/shadow with root:! (locked)
└── NO autologin, NO serial-console

ISO structure:
├── /live/filesystem.squashfs (base system)
└── /live/overlay/ (live-specific configs)
    ├── etc/systemd/system/console-autologin.service
    ├── etc/systemd/system/serial-console.service
    ├── etc/systemd/system/getty.target.wants/console-autologin.service -> symlink
    ├── etc/systemd/system/multi-user.target.wants/serial-console.service -> symlink
    └── etc/shadow (with root::)

init_tiny:
1. Mount squashfs as lower
2. Mount /live/overlay as middle layer (from ISO)
3. Mount tmpfs as upper (runtime writes)
4. Result: Live boot = base + overlay, writes go to tmpfs

recstrap:
1. Extracts squashfs only (no overlay)
2. Result: Installed system = clean base
```

## Implementation

### Phase 1: Remove live-specific configs from squashfs
- [x] Remove `setup_autologin()` call from `squashfs/system.rs`
- [x] Remove `setup_serial_console()` call from `squashfs/system.rs`
- [x] Remove `setup_live_root_access()` call from `squashfs/system.rs`
- [x] Remove dead code (`setup_serial_console` and `setup_live_root_access` functions)

### Phase 2: Create live overlay in leviso build
- [x] Add `create_live_overlay()` function to `build/systemd.rs`
- [x] Create console-autologin.service
- [x] Create serial-console.service
- [x] Create shadow file with `root::`
- [x] Create getty@tty1.service.d/live-disable.conf (conditional disable)

### Phase 3: Include overlay in ISO
- [x] Modify `iso.rs` to call `create_live_overlay()`
- [x] Copy live-overlay to `/live/overlay/` on ISO

### Phase 4: Three-layer overlay in init_tiny
- [x] Mount overlay from ISO to /live-overlay
- [x] Use `lowerdir=/live-overlay:/squashfs` for three-layer mount
- [x] Create `/live-boot-marker` for systemd units to detect live boot

### Phase 5: Root password handling
- [x] Base system (squashfs): `root:!` (locked, no login)
- [x] Live overlay: `root::` (empty password, autologin works)
- [x] Installed system: user sets password during installation

### Phase 6: Add dracut config and missing RPMs
- [x] Add `create_dracut_config()` to `build/etc.rs`
- [x] Add grep, findutils, tar to `SUPPLEMENTARY_RPMS`

## Files Modified

| File | Change |
|------|--------|
| `leviso/src/squashfs/system.rs` | Removed autologin/serial-console/live_root_access calls and dead code |
| `leviso/src/build/systemd.rs` | Added `create_live_overlay()`, removed `setup_serial_console()` |
| `leviso/src/build/etc.rs` | Added `create_dracut_config()` |
| `leviso/src/iso.rs` | Call create_live_overlay, copy to ISO |
| `leviso/src/extract.rs` | Added grep, findutils, tar RPMs |
| `leviso/profile/init_tiny` | Three-layer overlay mount |

## Verification (PASSED)

```bash
# Build and test:
cd leviso
cargo run -- build   # Full build
cargo run -- test    # Boots to root prompt with autologin

# Verify squashfs has NO autologin:
$ unsquashfs -l output/filesystem.squashfs | grep -E "autologin|serial-console"
(no output - CORRECT)

# Verify live overlay has autologin:
$ ls output/iso-root/live/overlay/etc/systemd/system/
console-autologin.service
getty.target.wants/
getty@tty1.service.d/
multi-user.target.wants/
serial-console.service

# Verify shadow files:
$ unsquashfs ... etc/shadow
root:!:19000:0:99999:7:::   # Base: locked

$ cat output/iso-root/live/overlay/etc/shadow
root::19000:0:99999:7:::    # Live: empty password
```

**Test output confirmed:**
- `console-autologin.service` started (autologin on tty1)
- `getty@tty1.service` did NOT start (conditional disable worked)
- `serial-console.service` started (serial shell for QEMU)
- Root prompt appeared without password

## Phase 7: Cleanup

- [x] Remove dead `profile/init` script (old bash init that exec'd systemd)
- [x] Remove stale `output/initramfs.cpio.gz` (175MB, unused)
- [x] Update `leviso/README.md` to reflect current architecture
- [x] Update `leviso/CLAUDE.md` to reference `profile/init_tiny`
- [x] Remove dead fallback code from `initramfs/mod.rs` (fail-fast if init_tiny missing)

## Security Improvement

Before:
- Installed systems had autologin + empty root password (INSECURE)

After:
- Installed systems require password (SECURE)
- Live ISO still has autologin for convenience (like archiso)
