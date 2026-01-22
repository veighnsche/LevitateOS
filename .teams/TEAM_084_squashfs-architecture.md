# TEAM_084: Squashfs Architecture Implementation

## Status: COMPLETE

## Goal
Replace the large initramfs (~250MB loaded into RAM) with a squashfs-based architecture:
- Tiny initramfs (~5MB) - just busybox + mount logic
- Single squashfs image (~350MB) - complete system for both live boot AND installation

## Architecture

```
ISO Contents:
├── boot/
│   ├── vmlinuz              # Kernel
│   └── initramfs.img        # Tiny (~5MB) - busybox + mount logic
├── live/
│   └── filesystem.squashfs  # COMPLETE system (~350MB)
├── efiboot.img              # EFI boot image
└── EFI/BOOT/
    ├── BOOTX64.EFI
    ├── grubx64.efi
    └── grub.cfg

Live Boot Flow:
1. GRUB loads kernel + tiny initramfs
2. Tiny init mounts ISO by LABEL (LEVITATEOS)
3. Mounts filesystem.squashfs read-only via loop device
4. Creates overlay: squashfs (lower) + tmpfs (upper)
5. switch_root to overlay
6. systemd boots as PID 1
```

## Benefits
- **RAM savings**: ~400MB → ~50MB at boot
- **Single source of truth**: Live = Installed (no duplication)
- **Simpler installation**: unsquashfs to disk instead of complex copy logic

## Key Implementation Details

1. **No modprobe needed**: squashfs/loop/overlay are kernel built-ins
2. **ISO must have LABEL**: xorriso `-V LEVITATEOS` critical
3. **Kernel cmdline**: `root=LABEL=LEVITATEOS`
4. **Squashfs content**: Merges rootfs binaries + initramfs networking

## Module Structure (Final)

```
src/
├── squashfs/              # NEW DEFAULT: Squashfs system builder
│   ├── mod.rs             # Entry point
│   ├── context.rs         # BuildContext with adapters
│   ├── system.rs          # Complete system orchestration
│   └── pack.rs            # mksquashfs wrapper
├── initramfs/             # NEW DEFAULT: Tiny busybox initramfs (~5MB)
│   └── mod.rs             # Downloads busybox, creates cpio
├── initramfs_depr/        # DEPRECATED: Large initramfs (~250MB)
│   └── (legacy code)      # Still used for network/dbus/chrony functions
├── rootfs_depr/           # DEPRECATED: Rootfs tarball builder
│   └── (legacy code)      # Still used for binaries/systemd/PAM functions
└── iso.rs                 # ISO builder with create_squashfs_iso()
```

## Key Design Decision: Adapter Pattern

The squashfs module reuses existing functionality from deprecated modules:

```rust
// src/squashfs/context.rs
impl BuildContext {
    pub fn as_initramfs_context(&self) -> crate::initramfs_depr::context::BuildContext
    pub fn as_rootfs_context(&self) -> crate::rootfs_depr::context::BuildContext
}
```

This allows `squashfs/system.rs` to call:
- `crate::rootfs_depr::parts::*` for binaries, systemd, PAM, etc.
- `crate::initramfs_depr::*` for network, dbus, chrony

## Files Created
- `src/squashfs/mod.rs` - Entry point
- `src/squashfs/context.rs` - BuildContext with adapters
- `src/squashfs/system.rs` - Complete system orchestration
- `src/squashfs/pack.rs` - mksquashfs wrapper
- `src/initramfs/mod.rs` - Tiny initramfs builder (renamed from initramfs_tiny)
- `profile/init_tiny` - Busybox init script

## Module Renames
- `src/initramfs/` → `src/initramfs_depr/` (legacy, deprecated)
- `src/rootfs/` → `src/rootfs_depr/` (legacy, deprecated)
- `src/initramfs_tiny/` → `src/initramfs/` (new default)

## Files Modified
- `src/main.rs` - New module declarations, default build uses squashfs
- `src/lib.rs` - Export new modules
- `src/iso.rs` - Added `create_squashfs_iso()` with ISO_LABEL constant
- All files in `src/rootfs_depr/` - Updated `crate::rootfs::` → `crate::rootfs_depr::`
- `tests/*.rs` - Updated imports to use `leviso::initramfs_depr::`

## CLI Commands

```
Build Commands:
  kernel     Build only the Linux kernel
  squashfs   Build squashfs system image (complete live system)  ← NEW DEFAULT
  initramfs  Build tiny initramfs (mounts squashfs, ~5MB)        ← NEW DEFAULT
  iso        Build only the ISO image

Hidden (deprecated):
  rootfs           [DEPRECATED] Build legacy rootfs tarball
  initramfs-legacy [DEPRECATED] Build legacy large initramfs
```

## Default Build Flow

```bash
cargo run -- build           # Runs all 3 steps below:
# 1. squashfs::build_squashfs()      → output/filesystem.squashfs
# 2. initramfs::build_tiny_initramfs() → output/initramfs-tiny.cpio.gz
# 3. iso::create_squashfs_iso()      → output/levitateos.iso
```

## Testing

```bash
cargo run -- build squashfs  # Build squashfs only
cargo run -- build initramfs # Build tiny initramfs only
cargo run -- build iso       # Build ISO only
cargo run -- build           # Build everything
cargo run -- run             # Boot in QEMU
```

All tests pass:
- 5 initramfs_depr tests (legacy)
- 5 rootfs_depr tests (legacy)
- 18 squashfs tests
- 26 validation tests (ignored, require built artifacts)

## Feature Regression Fixes (2026-01-21)

Identified and fixed feature regressions compared to legacy initramfs:

### 1. Keymaps
- **Problem**: `loadkeys` command wouldn't work without keymaps
- **Fix**: Added `copy_keymaps()` to `squashfs/system.rs` (copies `/usr/lib/kbd/keymaps`)

### 2. Missing Binaries
Added to `rootfs_depr/parts/binaries.rs`:
- **BIN**: `clear`, `stty`, `tty`, `vmstat`, `watch`, `loadkeys`, `localedef`, `udevadm`
- **SBIN**: `parted`

### 3. /etc/issue
- **Problem**: No pre-login prompt
- **Fix**: Added `/etc/issue` creation in `create_welcome_message()`

### 4. Shell Config
- **Problem**: Missing power aliases and motd display
- **Fix**: Added to `/etc/profile`:
  - `alias poweroff='systemctl poweroff --force'`
  - `alias reboot='systemctl reboot --force'`
  - `alias halt='systemctl halt --force'`
  - `cat /etc/motd` on interactive login

### 5. RPM Extraction (src/extract.rs)
Added supplementary RPM packages to extraction:
- `coreutils`, `coreutils-common` (printf, stty, etc.)
- `procps-ng` (vmstat, watch)
- `which`, `file`, `file-libs`, `diffutils` (utilities)
- `ncurses`, `ncurses-libs` (clear, tput)
- `sudo` (sudo, sudoedit)
- `util-linux`, `util-linux-core` (su, sulogin)
- `shadow-utils` (useradd, passwd, etc.)
- `iproute`, `iproute-tc` (ip, ss, bridge)
- `parted` (disk partitioning)
- `kbd` (loadkeys)

### 6. Timezone Data (src/rootfs_depr/parts/etc.rs)
- **Problem**: Only 5 timezone regions copied (UTC, America, Europe, Asia, Etc)
- **Missing**: 65 regions including Africa, Australia, Pacific, Japan, Brazil, etc.
- **Fix**: Changed `copy_timezone_data()` to copy ALL timezone data instead of just "essential zones"

## Verified Working (2026-01-21)
- Build completes: squashfs (427 MB), initramfs (685 KB), ISO (466 MB)
- Boot test passes: kernel boots, systemd starts, commands work
- New binaries verified: `loadkeys`, `clear`, `stty`, etc.
- `/etc/issue` and `/etc/motd` display correctly
- Power aliases work for live environment
- All 70 timezone regions available (Africa, Australia, Pacific, etc.)
