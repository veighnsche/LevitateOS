# disk/

Disk image management for LevitateOS.

TEAM_326: Renamed from `device/` to `disk/` for clarity.

## Files

| File | Description |
|------|-------------|
| `mod.rs` | Module exports |
| `image.rs` | Disk image creation and installation |

## Commands

| Command | Description |
|---------|-------------|
| `cargo xtask disk create` | Create disk image if missing |
| `cargo xtask disk install` | Install userspace binaries to disk |
| `cargo xtask disk status` | Show disk image status |

## Disk Image

- File: `tinyos_disk.img`
- Format: Raw (16MB FAT32)
- Used for persistent storage in QEMU

## History

- TEAM_322: Organized into submodule
- TEAM_326: Renamed from device/ to disk/, removed screenshot (moved to vm module)
