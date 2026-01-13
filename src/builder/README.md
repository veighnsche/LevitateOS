# build/

Build commands for LevitateOS kernel and userspace.

## Files

| File | Description |
|------|-------------|
| `mod.rs` | Module exports |
| `commands.rs` | Build command implementations |

## Commands

| Command | Description |
|---------|-------------|
| `cargo xtask build all` | Build kernel + userspace + initramfs |
| `cargo xtask build kernel` | Build kernel only |
| `cargo xtask build userspace` | Build userspace + initramfs |
| `cargo xtask build initramfs` | Create initramfs only |
| `cargo xtask build iso` | Build bootable Limine ISO |

## Architecture Support

Both `aarch64` and `x86_64` are supported via `--arch` flag.

## History

- TEAM_322: Organized into submodule
