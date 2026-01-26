# LevitateOS

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE)

Daily driver Linux for power users. Extract, configure, control. No compilation required.

## Status

**E2E tested, boots on real hardware. Installation workflow complete.**

| Working | In Progress |
|---------|-------------|
| ISO boots in QEMU (UEFI + BIOS) | AI-assisted installer integration |
| Live environment with root shell | Desktop environment packaging |
| `recstrap` extracts system to /mnt | Binary package repository |
| `recipe install` runs Rhai scripts | Extended bare metal testing |

Tested on: QEMU 8.x with OVMF (8GB RAM, NVMe virtio), bare metal (Intel/AMD desktops).

## What This Is

An ISO builder (`leviso`) that:
1. Downloads Rocky Linux 10 ISO
2. Extracts packages via `rpm --root`
3. Builds squashfs + initramfs
4. Packages hybrid UEFI/BIOS ISO

A package manager (`recipe`) where:
- Recipes are Rhai scripts, not YAML/JSON
- State is written back to recipe files (no database)
- You write/maintain your own recipes

Installation tools (`recstrap`, `recfstab`, `recchroot`) that:
- Extract squashfs to a mount point
- Generate fstab from mounted filesystems
- Set up chroot with bind mounts

## What This Isn't

- **Not a fork of Arch** - Uses Rocky packages, not Arch repos
- **Not production-ready** - Alpha software with incomplete testing
- **Not automated** - Manual partitioning, no guided installer (yet)
- **Not a package repository** - You maintain your own recipes
- **Not portable** - x86_64 only, requires Haswell (2013) or newer

## Quick Start

```bash
cd leviso
cargo run -- build    # Downloads ~2GB, takes 5-10 min
cargo run -- run      # Boots in QEMU with GUI
```

Requirements: Rust 1.75+, 50GB disk space, QEMU with OVMF.

## Project Structure

```
leviso/           # ISO builder (Rust)
tools/recipe/     # Package manager (Rust + Rhai)
tools/recstrap/   # System extractor
tools/recfstab/   # fstab generator
tools/recchroot/  # chroot helper
testing/          # E2E test suites
llm-toolkit/      # LoRA training scripts (not integrated)
```

## Installation (Manual)

From the live ISO (like Arch Linux):

```bash
# Partition (GPT + EFI) - replace nvme0n1 with your disk
fdisk /dev/nvme0n1

# Format (1GB EFI, rest for root)
mkfs.fat -F32 /dev/nvme0n1p1
mkfs.ext4 /dev/nvme0n1p2

# Mount
mount /dev/nvme0n1p2 /mnt
mkdir -p /mnt/boot
mount /dev/nvme0n1p1 /mnt/boot

# Extract system
recstrap /mnt

# Generate fstab and enter chroot
recfstab /mnt >> /mnt/etc/fstab
recchroot /mnt

# Configure (inside chroot)
passwd
useradd -m -G wheel myuser
passwd myuser
bootctl install
systemctl enable NetworkManager
exit

# Reboot into installed system
reboot
```

Full control. You partition, format, configure - just like Arch.

## Recipe Example

```rhai
let name = "ripgrep";
let version = "14.1.0";
let installed = false;

fn acquire() {
    download(`https://github.com/BurntSushi/ripgrep/releases/download/${version}/ripgrep-${version}-x86_64-unknown-linux-musl.tar.gz`);
}

fn install() {
    extract("tar.gz");
    install_bin(`ripgrep-${version}-x86_64-unknown-linux-musl/rg`);
}
```

After `recipe install ripgrep`, the engine writes `installed = true` back to the file.

## Hardware Requirements

| Resource | Minimum | Recommended (LLM-ready) |
|----------|---------|-------------------------|
| CPU | Intel Haswell / AMD Zen | 8+ cores with AVX2 |
| RAM | 16 GB | 32-64 GB |
| Storage | 512 GB NVMe | 1-2 TB NVMe |
| GPU | Integrated (CPU inference) | NVIDIA RTX 3060+ 12GB |
| Boot | UEFI required | Secure Boot disabled |

LevitateOS ships with local LLM capabilities. Recommended specs run 7B-13B models.

## Current State

- **Manual installation** - Like Arch, you control every step
- **Local recipes** - You maintain packages, no central repository (by design)
- **AI-assisted installer** - LLM toolkit in development for guided installation
- **Desktop ready** - Install Sway/Hyprland via recipe post-install

## Building from Source

```bash
# ISO builder
cd leviso && cargo build --release

# Package manager
cd tools/recipe && cargo build --release

# Installation tools
cd tools/recstrap && cargo build --release
cd tools/recfstab && cargo build --release
cd tools/recchroot && cargo build --release
```

## License

MIT (LevitateOS code), various open source (Rocky Linux packages).

## Documentation

- [Installation Guide](docs/content/src/content/01-getting-started/)
- [Architecture FAQ](docs/content/src/content/04-architecture/01-faq.ts)
- [How We Compare](docs/content/src/content/04-architecture/02-how-we-compare.ts)
- [Supply Chain](SUPPLY_CHAIN.md)

## Links

- [Rocky Linux](https://rockylinux.org) - Package source
- [Rhai](https://rhai.rs) - Recipe scripting language
