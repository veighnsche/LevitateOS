# TEAM_477: Wayland Desktop Support Summary

## What Works

### 1. Build Infrastructure (COMPLETE)
- **Alpine package extractor** (`src/builder/alpine.rs`)
  - Downloads and extracts Alpine Linux packages
  - Multi-mirror retry logic for reliability
  - Packages extracted to `toolchain/alpine-root/{arch}/`

- **wlroots builder** (`src/builder/wlroots.rs`)
  - Builds wlroots 0.18.2 from source via distrobox Alpine
  - Output: `toolchain/wlroots-out/x86_64/lib/libwlroots-0.18.so` (1.3 MB)

- **sway builder** (`src/builder/sway.rs`)
  - Builds sway 1.10.1 from source via distrobox Alpine
  - Output: `toolchain/sway-out/x86_64/bin/sway` (723 KB)

- **foot builder** (`src/builder/foot.rs`)
  - Builds foot 1.20.2 from source via distrobox Alpine
  - Output: `toolchain/foot-out/x86_64/bin/foot` (615 KB)

### 2. Distrobox Alpine Setup (COMPLETE)
The build requires distrobox with Alpine edge repos. Setup:
```bash
# Create Alpine distrobox (if not exists)
distrobox create -i alpine:edge -n Alpine

# Enter and upgrade to edge repos
distrobox enter Alpine
sudo sed -i 's/v3.20/edge/g' /etc/apk/repositories
sudo apk update && sudo apk upgrade

# Install build dependencies
sudo apk add meson ninja pkgconf wayland-dev wayland-protocols \
    libxkbcommon-dev libinput-dev libevdev-dev mesa-dev libdrm-dev \
    pixman-dev cairo-dev pango-dev libseat-dev eudev-dev json-c-dev \
    pcre2-dev glib-dev fcft-dev utf8proc-dev tllist scdoc
```

### 3. QEMU Integration (COMPLETE)
- **SDL display with virgl** works better than GTK (GTK has display refresh issues)
- `src/qemu/builder.rs` has `DisplayMode::Sdl` with `gl=on` for 3D acceleration
- `src/run.rs:run_qemu_wayland()` uses virtio-gpu-gl-pci

### 4. Wayland Initramfs (PARTIAL)
- Creates at `target/initramfs/x86_64-wayland.cpio` (~350 MB with LLVM)
- Contains: BusyBox, OpenRC, Alpine libs, wlroots, sway, foot, seatd

### 5. OpenRC gendepends.sh Fix (COMPLETE)
Fixed relative paths in `toolchain/openrc-out/x86_64/lib/rc/sh/gendepends.sh`:
- Line 58: `etc/init.d` → `/etc/init.d`
- Line 116-118: `etc/rc.conf` → `/etc/rc.conf`

**Note:** This fix is lost if OpenRC is rebuilt from scratch.

## What Still Needs Work

### Missing Alpine Libraries
When running `start-wayland`, sway fails with missing shared libraries:
- `libintl.so.8` - needs `musl-libintl` or `gettext`
- `libbz2.so.1` - needs `bzip2-libs` or similar
- `libmount.so.1` - needs `util-linux-libs`
- `libelf.so.1` - needs `elfutils-libelf`
- `libelogind.so.0` - needs `elogind-libs`

The Alpine package list in `src/builder/alpine.rs` was updated but some packages may have wrong names or not be extracting their .so files correctly.

### Current Package List (in alpine.rs)
```rust
LIB_PACKAGES includes:
- json-c, musl-libintl, mesa-gles, libdisplay-info
- brotli-libs, zstd-libs, util-linux-libs
- graphite2, libgcc, libstdc++, elogind-libs
- libbsd, elfutils-libelf, libpciaccess, llvm19-libs
```

### Debug Approach
1. Check what packages actually downloaded: `ls toolchain/alpine-cache/x86_64/`
2. Check what .so files were extracted: `find toolchain/alpine-root/x86_64 -name "*.so*"`
3. The package names may differ from library names - check APKINDEX

## CLI Commands

```bash
# Build all Wayland components
cargo run -- build wayland

# Build just the initramfs
cargo run -- build wayland-initramfs

# Run with Wayland
cargo run -- run --wayland

# After boot, in serial console:
start-wayland
```

## Key Files Modified

| File | Changes |
|------|---------|
| `src/builder/alpine.rs` | Package extractor, multi-mirror retry, package lists |
| `src/builder/wlroots.rs` | wlroots 0.18.2 builder |
| `src/builder/sway.rs` | sway 1.10.1 builder |
| `src/builder/foot.rs` | foot 1.20.2 builder |
| `src/builder/initramfs/mod.rs` | `create_wayland_initramfs()`, added pidof/pgrep symlinks |
| `src/builder/mod.rs` | Exports new modules |
| `src/qemu/builder.rs` | Added `DisplayMode::Sdl`, virgl support |
| `src/run.rs` | `run_qemu_wayland()` function |
| `src/main.rs` | `--wayland` flag, build commands |

## Architecture

```
Host (Fedora)
  └── cargo run -- build wayland
        ├── Downloads Alpine .apk packages
        ├── Extracts to toolchain/alpine-root/x86_64/
        └── Builds via distrobox Alpine:
              ├── wlroots → toolchain/wlroots-out/x86_64/
              ├── sway → toolchain/sway-out/x86_64/
              └── foot → toolchain/foot-out/x86_64/

  └── cargo run -- run --wayland
        ├── Creates initramfs with all components
        ├── Launches QEMU with:
        │     -display sdl,gl=on
        │     -device virtio-gpu-gl-pci
        └── User runs `start-wayland` in VM
```

## Next Steps

1. Fix missing library packages - verify Alpine package names
2. Test sway launches successfully
3. Add `--wayland-term` mode for debugging (serial + GPU window)
4. Phase 2 tools: fuzzel, waybar, mako, etc.
