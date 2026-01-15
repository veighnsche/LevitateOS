# LLM Runner: Scope & Design

## Overview

FunctionGemma in LevitateOS is a **narrow, specialized tool** - NOT a general-purpose shell assistant.

## Target Users

Arch-level users who would normally read documentation instead of asking an LLM. This tool saves time on repetitive tasks, not hand-holding.

## Use Cases

### 1. Installation Commands (Arch-style)

During LevitateOS installation, translate user intent to installation commands.

**Scope:**
- Disk partitioning (GPT/UEFI only)
- Filesystem formatting (btrfs, ext4)
- Mounting and chroot
- Bootloader setup (systemd-boot)
- Basic system configuration (fstab, locale, timezone, hostname, users)

**Assumptions (Greenfield):**
- UEFI only, no legacy BIOS
- Modern hardware (high minimum specs - we run local LLMs)
- GPT partitioning only
- systemd-boot (not GRUB)
- No encryption initially

**Examples:**
```
"partition disk for uefi" → gdisk or parted commands
"format root as btrfs" → mkfs.btrfs /dev/sdX
"generate fstab" → genfstab -U /mnt >> /mnt/etc/fstab
"install bootloader" → bootctl install
```

### 2. Package Recipe Generation

Generate download/build/install/update scripts for the local package manager.

**Scope:**
- Download source (git clone, curl, wget)
- Build commands (make, cmake, meson, cargo, etc.)
- Install commands
- Update scripts

**Recipe Format: RHAI (Deferred)**

We will use RHAI scripting for recipes. Design details TBD.

```
// Example concept (not final)
fn download() {
    git_clone("https://github.com/foo/bar", "bar");
}

fn build() {
    cd("bar");
    run("cargo build --release");
}

fn install() {
    copy("target/release/bar", "/usr/local/bin/bar");
}
```

## Interaction Model

1. User provides natural language request
2. FunctionGemma generates command(s)
3. **Show and confirm** before execution
4. Execute only after user approval

## Error Handling

If uncertain or out of scope: **Say "unsupported"** - don't guess.

This is narrow by design. We don't try to handle everything.

## Fine-Tuning Strategy

FunctionGemma 270M requires fine-tuning for reliable performance (58% → 85% accuracy).

**Training Dataset:**
- ~500-1000 examples focused on installation + recipe generation
- Narrow scope = smaller, higher-quality dataset
- All examples follow show-and-confirm pattern

## TODO

- [ ] Design RHAI recipe format and traits
- [ ] Create installation command training dataset
- [ ] Create recipe generation training dataset
- [ ] Fine-tune FunctionGemma
- [ ] Integrate with installer UI
- [ ] Integrate with package manager

## Non-Goals

- General shell assistance ("show me files", "what's running")
- System administration beyond installation
- Conversational AI / chatbot behavior
- Multi-step reasoning or complex workflows
