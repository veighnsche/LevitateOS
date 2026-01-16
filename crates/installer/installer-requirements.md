# LevitateOS Installer Requirements

## Overview

A Rust-based installer inspired by archinstall, but with **SmolLM3-3B LLM** for natural language understanding. Users describe what they want, the LLM translates to actions.

## Design Philosophy

1. **Conversational** - User types natural language, not menu selections
2. **Transparent** - Always show what will happen before doing it
3. **Recoverable** - Mistakes can be undone where possible
4. **Minimal** - Only install what's needed

---

## Installation Stages

### 1. Disk Configuration

**What it does:**
- List available disks and partitions
- Create partition tables (GPT preferred)
- Create partitions (EFI, swap, root, home)
- Format filesystems (ext4, btrfs, xfs)
- Setup encryption (LUKS) if requested

**Example prompts:**
```
"list disks"
"use the whole 500gb drive"
"encrypted root partition"
"dual boot with windows"
"separate home partition"
```

**LLM output:**
```json
{
  "action": "partition",
  "disk": "/dev/nvme0n1",
  "scheme": [
    {"mount": "/boot/efi", "size": "512M", "fs": "vfat"},
    {"mount": "swap", "size": "8G", "fs": "swap"},
    {"mount": "/", "size": "rest", "fs": "ext4"}
  ]
}
```

### 2. System Installation

**What it does:**
- Mount target partitions
- Copy live system to target (rsync)
- Generate /etc/fstab
- Install bootloader (GRUB)

**Example prompts:**
```
"install the system"
"copy to disk"
```

*Mostly automatic with progress bar.*

### 3. System Configuration

**What it does:**
- Set hostname
- Set timezone
- Set locale/language
- Set keyboard layout

**Example prompts:**
```
"hostname is my-laptop"
"timezone los angeles"
"keyboard us"
"language english"
```

### 4. User Setup

**What it does:**
- Create user account(s)
- Set passwords
- Configure sudo access

**Example prompts:**
```
"create user vince with sudo"
"set password"
"add user to wheel group"
```

### 5. Bootloader

**What it does:**
- Install GRUB to EFI partition or MBR
- Generate grub.cfg
- Set as default boot entry

**Example prompts:**
```
"install bootloader"
```

*Usually automatic after system installation.*

### 6. Finalize

**What it does:**
- Unmount partitions
- Offer to reboot

**Example prompts:**
```
"done"
"reboot now"
"exit without reboot"
```

---

## SmolLM3-3B Integration

### Model Requirements
- Base: SmolLM3-3B 270M (fits in ~1GB RAM)
- LoRA adapter: Trained on installation commands
- Inference: CPU-only (live ISO environment)

### Conversation Sequence (Multi-Turn)

SmolLM3-3B uses a **four-turn cycle**, NOT a single combined response:

```
Turn 1: User sends prompt
        "list disks"

Turn 2: Model outputs function call (with special tokens)
        <start_function_call>call:run_shell_command{command:<escape>lsblk<escape>}<end_function_call>

Turn 3: Developer executes command, sends result back as "tool" role
        {"role": "tool", "content": "sda  500G disk\nsdb  1T disk"}

Turn 4: Model outputs natural language summary
        "You have two disks: a 500GB SSD and a 1TB HDD."
```

**Key insight:** The model does NOT output both function call AND natural language in the same response. They are **separate turns**.

### Implementation Flow

```rust
loop {
    let response = llm.generate(messages);

    if let Some(function_call) = extract_function_call(&response) {
        // Execute the command
        let result = execute_command(&function_call.command);

        // Add tool response to messages
        messages.push(Message::tool(result));

        // Continue loop - model will generate natural language next
    } else {
        // Natural language response - show to user
        display_response(&response);
        break;
    }
}
```

### Reference: OpenCode Implementation

OpenCode (vendor/opencode) uses Vercel AI SDK with streaming tool calls:
- `streamText()` with `tools` parameter
- `while (true)` loop until all tool calls processed
- Tool call events: `tool-input-start` → `tool-call` → `tool-result`
- Text events: `text-start` → `text-delta` → `text-end`

However, SmolLM3-3B uses **custom tokens** (`<start_function_call>`, `<escape>`), not OpenAI-style tool calling. Manual parsing required.

### Input/Output Contract

**Input:** Natural language user request
**Output:** Either function call OR natural language (not both)

### Action Types

```rust
enum InstallerAction {
    ListDisks,
    Partition { disk: String, scheme: Vec<Partition> },
    Format { partition: String, filesystem: String },
    Mount { partition: String, mountpoint: String },
    CopySystem { source: String, target: String },
    SetHostname { name: String },
    SetTimezone { zone: String },
    CreateUser { name: String, sudo: bool },
    SetPassword { user: String, password: String },
    InstallBootloader { target: String },
    Reboot,
    Help,
    Clarify { question: String },  // LLM needs more info
}
```

### Conversation Flow

```
1. User types request
2. LLM parses → InstallerAction
3. If Clarify → ask user, goto 1
4. Show plan to user
5. User confirms (y/n)
6. Execute action
7. Show result
8. Loop
```

---

## Technical Requirements

### Rust Crates Needed
- `serde` / `serde_json` - Action serialization
- `nix` - Low-level system calls (mount, chroot)
- `gptman` - GPT partition table manipulation
- `mbrman` - MBR partition table manipulation
- `drives` - Disk enumeration (or read `/sys/block/` directly)
- `indicatif` - Progress bars
- `rustyline` - Line editing / history

### SmolLM3-3B Integration
- Python HTTP server (`crates/installer/python/llm_server.py`)
- HuggingFace Transformers for inference
- Rust calls server via HTTP on localhost:8765
- Server auto-starts when installer launches, auto-stops on exit
- Model loaded once at startup, stays in memory for fast responses

### Privilege Handling
- Run as root (installer context)
- Or use capability-based access for specific operations

---

## Non-Goals (for v1.0)

- Network configuration (use NetworkManager post-install)
- Package selection (minimal base only)
- Desktop environment choice (headless focus)
- RAID/LVM (future)
- Secure boot signing (future)

---

## Open Questions

1. **How to handle LLM failures?** Fall back to explicit prompts?
2. **Offline model loading** - Bundle model in ISO? Separate download?
3. **Disk operations library** - Use `parted` CLI or pure Rust?
4. **Copy method** - rsync vs unsquashfs vs cp?

---

## TUI Design

Using `ratatui` for the terminal interface.

```
┌─ Installation Steps ─────────────┬─ Chat ──────────────────────────────┐
│                                  │                                     │
│ [ ] Disk Configuration           │ LevitateOS Installer                │
│     "partition the disk"         │                                     │
│     "use whole disk encrypted"   │ > what disks do I have              │
│                                  │                                     │
│ [ ] System Installation          │ Found:                              │
│     "install the system"         │   /dev/sda - 500GB SSD              │
│                                  │   /dev/sdb - 1TB HDD                │
│ [ ] Configuration                │                                     │
│     "set timezone to..."         │ > use the whole ssd                 │
│     "hostname is..."             │                                     │
│                                  │ Plan:                               │
│ [ ] User Setup                   │   /dev/sda1: 512MB EFI              │
│     "create user X with sudo"    │   /dev/sda2: 499.5GB root           │
│                                  │                                     │
│ [ ] Bootloader                   │ Proceed? [y/n]                      │
│     "install bootloader"         │                                     │
│                                  │ > _                                 │
│ [ ] Finalize                     │                                     │
│     "done" or "reboot"           │                                     │
│                                  │                                     │
└──────────────────────────────────┴─────────────────────────────────────┘
```

### Layout

- **Left panel (35%)**: Installation checklist with example prompts for each stage
- **Right panel (65%)**: Scrolling chat history + input line at bottom

### Interaction

- User types in the input area (bottom of right panel)
- LLM response appears in chat history
- Checkboxes auto-update as stages complete
- Current stage is highlighted

### Rust Crates for TUI

- `ratatui` - TUI framework
- `crossterm` - Terminal backend

---

## Example Full Session

```
LevitateOS Installer v1.0
Type what you want to do. I'll help you install.

> what disks do I have

Found:
  /dev/sda - 500GB SSD (Samsung 870 EVO)
  /dev/sdb - 1TB HDD (WD Blue)

> install on the ssd, whole disk, encrypted

Plan:
  Disk: /dev/sda (500GB)
  - /dev/sda1: 512MB EFI (vfat)
  - /dev/sda2: 499.5GB LUKS → ext4 root

  WARNING: This will ERASE all data on /dev/sda

Proceed? [y/n] > y

Creating partition table... done
Creating EFI partition... done
Creating LUKS container...
  Enter encryption passphrase: ********
  Confirm: ********
Formatting root... done
Mounting... done
Copying system... [████████████████████] 100%
Generating fstab... done

> timezone los angeles

Set timezone to America/Los_Angeles

> create user vince with sudo

Created user 'vince' with sudo access
Set password for vince: ********

> install bootloader

Installing GRUB to /dev/sda... done
Generating grub.cfg... done

> done

Unmounting partitions... done

Installation complete! Remove the USB and reboot.
Reboot now? [y/n] > y
```
