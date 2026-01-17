# LevitateOS

A Linux distribution with an AI-powered installer and self-sufficient package manager.

## Features

### AI-Powered Installer
- **SmolLM3-3B** runs locally - no internet required
- Natural language commands: "use the whole 500gb drive", "create user vince with sudo"
- Multi-turn conversation context understands "it", "that one", "yes"
- TUI chat interface built with Ratatui
- 7,000+ training examples for installation workflows

### S-Expression Package Recipes
Lisp-like syntax designed for small LLMs to generate reliably:

```lisp
(package "ripgrep" "14.1.0"
  (acquire (binary (x86_64 "URL")))
  (build (extract tar-gz))
  (install (to-bin "rg")))
```

- 30-line parser - simple and reliable
- Single recipe handles both binary and source builds
- Version constraints: `>=`, `<=`, `~=` (compatible release)
- Conditional features: `(if vulkan "vulkan-loader >= 1.3")`
- Split packages for -dev files

### Self-Sufficient Package Manager (`levitate`)
- **No apt, dnf, or pacman dependency**
- Full lifecycle: acquire → build → install → configure → start/stop → remove
- SHA256 verification, patches support

```bash
levitate install ripgrep
levitate deps firefox
levitate desktop  # Install Sway stack
```

### Pure Wayland Desktop
- Complete Sway compositor stack (17 recipes)
- wayland, wlroots, sway, foot, waybar, wofi, mako
- XWayland disabled by default
- No X11 bloat

### musl + GNU Stack
Most distros use glibc+GNU (Fedora) or musl+busybox (Alpine).
LevitateOS uses **musl libc + GNU tools** = lightweight + full-featured.

- ~1MB libc vs ~10MB glibc
- Better static linking
- Full GNU coreutils

## Quick Start

```bash
# Build the ISO
./build-iso.sh

# Boot in VM
./run-vm.sh
```

## Development

```bash
# Build
cargo run --bin builder -- initramfs

# VM control
cargo xtask vm start
cargo xtask vm stop
cargo xtask vm send "command"
cargo xtask vm log
```

## Structure

```
installer/        # AI-powered TUI installer
recipe/           # S-expression recipe parser + levitate CLI
xtask/            # Dev tasks (VM control, tests)
vendor/           # Reference implementations
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
- **Fedora components**: GPL-2.0/GPL-3.0/LGPL-2.1

See [LICENSE](LICENSE) for details.

## Credits

**Core Technologies**
- [SmolLM3](https://huggingface.co/HuggingFaceTB/SmolLM3-3B) - 3B parameter LLM by Hugging Face
- [Fedora](https://fedoraproject.org) - Base distribution
- [Rust](https://www.rust-lang.org) - Systems programming language

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
