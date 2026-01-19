# LevitateOS

A Linux distribution built with Rust, combining Arch Linux's elegant build system
with Rocky Linux's enterprise-grade packages.

## Core Principles

### 1. Arch-like ISO Builder (leviso)

Borrowed from archiso - the tool that builds Arch Linux ISOs:

- **Profile-based configuration**: `profile/packages.txt` + `profile/airootfs/` overlay
- **Declarative package lists**: Just list package names, leviso handles the rest
- **SquashFS compression**: xz + BCJ filter for minimal ISO size
- **Hybrid boot**: BIOS + UEFI support via xorriso
- **dracut + dmsquash-live**: Standard Linux live boot infrastructure

### 2. Rocky Linux 10 Prebuilt Binaries

No compilation required - builds in minutes, not hours:

- **Extract RPMs directly** from Rocky Linux 10 minimal ISO
- **Enterprise-grade packages** with security patches and stability
- **glibc-based**: Rocky's packages, not musl
- **`rpm --root`** for clean installation into any target directory

### 3. Rhai-Scripted Recipe Package Manager

Executable code, not static configuration:

```rhai
// recipes/ripgrep.rhai
let pkg = #{
    name: "ripgrep",
    version: "14.1.0",
};

fn install() {
    let url = `https://github.com/BurntSushi/ripgrep/releases/download/${pkg.version}/ripgrep-${pkg.version}-x86_64-unknown-linux-musl.tar.gz`;
    download(url);
    extract("tar.gz");
    install_bin("rg");
}
```

- **State lives in recipe files**: No external database - the engine writes `installed = true` back
- **Supports variables, conditionals, loops**: Real programming, not limited DSL
- **Self-sufficient**: No apt/dnf/pacman dependency
- **Helpers**: `rpm_install()`, `install_bin()`, `install_lib()`, `install_man()`

## Three-Layer Architecture

```
ISO Builder → Live Environment → Installed System
```

1. **leviso** creates bootable ISO from Rocky packages
2. **Live environment** boots with recipe binary and tools
3. **`recipe bootstrap /mnt`** installs base system (like Arch's pacstrap)
4. **`recipe install`** adds packages post-install

## AI-Powered Installer

- **SmolLM3-3B** runs locally - no internet required
- Natural language commands: "use the whole 500gb drive", "create user vince with sudo"
- Multi-turn conversation context understands "it", "that one", "yes"
- TUI chat interface built with Ratatui

## Quick Start

```bash
# Build the ISO with leviso
cargo run -p leviso -- build

# Or use the wrapper script
./build-iso.sh

# Boot in VM
./run-vm.sh
```

## Development

```bash
# Build initramfs (for testing)
cargo run --bin builder -- initramfs

# VM control
cargo xtask vm start
cargo xtask vm stop
cargo xtask vm send "command"
cargo xtask vm log
```

## Structure

```
leviso/           # Arch-like ISO builder
recipe/           # Rhai-based package manager
crates/
  builder/        # Builds artifacts (kernel, initramfs)
  installer/      # AI-powered TUI installer
  levitate/       # Package manager CLI

xtask/            # Dev tasks (VM control, tests)
vendor/           # Reference implementations (systemd, util-linux, brush, uutils)
docs/             # Design docs
website/          # Documentation website
.teams/           # Work history
```

## Requirements

### System

- x86_64 architecture
- 20GB disk minimum
- UEFI recommended

### AI Installer (SmolLM3-3B)

The LLM requires GPU acceleration or sufficient RAM:

| Hardware | VRAM/RAM | Notes |
|----------|----------|-------|
| **NVIDIA GPU** | 6GB+ VRAM | CUDA, best compatibility |
| **NVIDIA GPU (4-bit)** | 2GB+ VRAM | With bitsandbytes quantization |
| **AMD GPU** | 6GB+ VRAM | ROCm 5.6+, RX 6000/7000 series |
| **Intel Arc** | 6GB+ VRAM | Via IPEX-LLM |
| **Apple Silicon** | 8GB+ unified | Metal/MPS acceleration |
| **CPU only** | 8GB+ RAM | Slow, fallback option |

## License

- **LevitateOS code**: MIT
- **SmolLM3 model**: Apache-2.0
- **Rocky Linux components**: Various open source licenses

See [LICENSE](LICENSE) for details.

## Credits

**Core Technologies**

- [SmolLM3](https://huggingface.co/HuggingFaceTB/SmolLM3-3B) - 3B parameter LLM by Hugging Face
- [Rocky Linux](https://rockylinux.org) - Enterprise Linux base packages
- [Arch Linux](https://archlinux.org) - Build system inspiration (archiso)
- [Rust](https://www.rust-lang.org) - Systems programming language
- [Rhai](https://rhai.rs) - Embedded scripting language for recipes

**Rust Crates**

- [Ratatui](https://ratatui.rs) - TUI framework for the installer
- [Clap](https://clap.rs) - CLI argument parsing

**Desktop Stack**

- [Sway](https://swaywm.org) - Wayland compositor
- [wlroots](https://gitlab.freedesktop.org/wlroots/wlroots) - Wayland compositor library
- [foot](https://codeberg.org/dnkl/foot) - Terminal emulator
- [waybar](https://github.com/Alexays/Waybar) - Status bar
- [wofi](https://hg.sr.ht/~scoopta/wofi) - Application launcher
- [mako](https://github.com/emersion/mako) - Notification daemon

**AI/ML**

- [PyTorch](https://pytorch.org) - ML framework
- [Transformers](https://huggingface.co/transformers) - Model inference
- [PEFT](https://github.com/huggingface/peft) - LoRA fine-tuning
