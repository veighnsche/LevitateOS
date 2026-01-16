# LevitateOS Installer LLM Training Requirements

This document defines the capabilities the SmolLM3-3B model must learn to be an effective installation assistant.

---

## CRITICAL: THIS IS A CONVERSATION, NOT SINGLE QUERIES

**THE #1 MISTAKE TO AVOID:** Training on isolated query→response pairs.

The installer is a **multi-turn conversation**. Every single training example MUST include conversation history. The user doesn't restart from scratch each message - they build on what was said before.

### Why This Matters

```
# WRONG - Isolated queries (what failed before)
{"query": "format it", "command": "???"}  # Format WHAT?

# RIGHT - Conversation with history
{
  "history": [
    {"role": "user", "content": "list disks"},
    {"role": "assistant", "content": "[ran lsblk, showed /dev/sda and /dev/nvme0n1]"},
    {"role": "user", "content": "use the nvme"},
    {"role": "assistant", "content": "[ran sgdisk on /dev/nvme0n1]"}
  ],
  "query": "format it",
  "command": "mkfs.ext4 /dev/nvme0n1p2"  # Knows "it" = nvme0n1p2 from context
}
```

### Conversational References the LLM Must Understand

| User says | Requires history to resolve |
|-----------|----------------------------|
| "format it" | Which partition? |
| "the bigger one" | Which disk was bigger? |
| "yes" | Yes to what question? |
| "the second partition" | Of which disk? |
| "mount that" | Mount what where? |
| "do the same for boot" | What did we just do? |
| "now the other disk" | Which other disk? |

### Every Training Example = A Conversation Snapshot

Think of each training example as a snapshot mid-conversation:
- What has been said so far (history)
- What the user just said (query)
- What the LLM should respond (command or text)

**100% of training data must have conversation history, even if it's just 1 previous exchange.**

---

## 1. System Context (Injected at Runtime)

The LLM receives real system state before each query. Training data must use realistic examples of this context.

### Context Format
```
## Current System State

- Boot mode: UEFI | Legacy BIOS
- Network: Connected | Not connected
- Hostname: <current hostname>
- Timezone: <timezone or "not set">

## Available Disks

- /dev/sda: 500G (Samsung SSD 870)
  - /dev/sda1: 512M [vfat] mounted at /boot/efi
  - /dev/sda2: 499.5G [ext4] mounted at /
- /dev/nvme0n1: 1T (WD Black SN850)
  - (no partitions)

## Current Mounts

Target partitions are mounted under /mnt

## Existing Users: vince, admin
```

### Key Facts Available
| Fact | Source | Example Values |
|------|--------|----------------|
| Boot mode | `/sys/firmware/efi/efivars` existence | UEFI, Legacy BIOS |
| Network | `ping archlinux.org` | Connected, Not connected |
| Hostname | `hostname` command | archiso, localhost |
| Timezone | `/etc/localtime` symlink | America/New_York, not set |
| Disks | `lsblk -J` | /dev/sda, /dev/nvme0n1, /dev/vda |
| Partitions | `lsblk -J` children | /dev/sda1, /dev/nvme0n1p1 |
| Filesystems | `lsblk -J` fstype | ext4, vfat, btrfs, xfs |
| Mount points | `lsblk -J` mountpoint | /mnt, /mnt/boot |
| Users | `/etc/passwd` (UID 1000-60000) | vince, admin |

---

## 2. Installation Stages

The installer guides users through these stages:

### Stage 1: Disk Configuration
**Goal:** Partition and format disks, mount filesystems

**User intents:**
- List available disks
- Show disk details
- Create partitions (GPT for UEFI, MBR for BIOS)
- Format partitions
- Mount partitions

**Commands the LLM should generate:**
```bash
# List disks
lsblk
lsblk -f                    # with filesystem info
fdisk -l                    # detailed

# Partition (UEFI/GPT)
sgdisk -Z /dev/sda                              # wipe partition table
sgdisk -n 1:0:+512M -t 1:ef00 /dev/sda          # EFI partition
sgdisk -n 2:0:0 -t 2:8300 /dev/sda              # root partition

# Partition (BIOS/MBR)
parted /dev/sda mklabel msdos
parted /dev/sda mkpart primary ext4 1MiB 100%

# Format
mkfs.fat -F32 /dev/sda1             # EFI
mkfs.ext4 /dev/sda2                 # root (ext4)
mkfs.btrfs /dev/sda2                # root (btrfs)
mkfs.xfs /dev/sda2                  # root (xfs)

# Mount
mount /dev/sda2 /mnt                # root first
mkdir -p /mnt/boot/efi
mount /dev/sda1 /mnt/boot/efi       # then EFI
```

**CRITICAL:** LLM must ONLY reference disks that exist in the system context. Never hallucinate disk names.

### Stage 2: System Installation
**Goal:** Install base system to mounted partitions

**Commands:**
```bash
# Arch-style
pacstrap /mnt base linux linux-firmware

# Or copy from live system
rsync -aAXv --exclude={"/dev/*","/proc/*","/sys/*","/tmp/*","/run/*","/mnt/*"} / /mnt/
```

### Stage 3: System Configuration
**Goal:** Configure hostname, timezone, locale, keyboard

**User intents & commands:**
```bash
# Hostname
echo "my-laptop" > /mnt/etc/hostname
hostnamectl set-hostname my-laptop              # if systemd available

# Timezone
ln -sf /usr/share/zoneinfo/America/New_York /mnt/etc/localtime
timedatectl set-timezone America/New_York       # if systemd available

# Locale
echo "en_US.UTF-8 UTF-8" >> /mnt/etc/locale.gen
arch-chroot /mnt locale-gen
echo "LANG=en_US.UTF-8" > /mnt/etc/locale.conf

# Keyboard
echo "KEYMAP=us" > /mnt/etc/vconsole.conf
```

### Stage 4: User Setup
**Goal:** Create user accounts with proper permissions

**Commands:**
```bash
# Create user with home directory
useradd -m -G wheel vince

# Set password (interactive)
passwd vince

# Add to sudo/wheel group
usermod -aG wheel vince

# Enable sudo for wheel group
echo "%wheel ALL=(ALL) ALL" >> /mnt/etc/sudoers
```

### Stage 5: Bootloader
**Goal:** Install bootloader appropriate for boot mode

**UEFI systems:**
```bash
# systemd-boot
bootctl --path=/mnt/boot install

# GRUB (UEFI)
grub-install --target=x86_64-efi --efi-directory=/mnt/boot/efi --bootloader-id=GRUB
grub-mkconfig -o /mnt/boot/grub/grub.cfg
```

**BIOS systems:**
```bash
# GRUB (BIOS)
grub-install --target=i386-pc /dev/sda
grub-mkconfig -o /mnt/boot/grub/grub.cfg
```

### Stage 6: Finalize
**Goal:** Generate fstab, unmount, reboot

**Commands:**
```bash
# Generate fstab
genfstab -U /mnt >> /mnt/etc/fstab

# Unmount
umount -R /mnt

# Reboot
reboot
```

---

## 3. Response Types

The LLM must learn when to:

### A. Execute a Command (Function Call)
Use when user requests an ACTION:
- "list disks" → `lsblk`
- "format sda2 as ext4" → `mkfs.ext4 /dev/sda2`
- "mount root partition" → `mount /dev/sda2 /mnt`

**Output format:**
```
call:run_shell_command{command:<escape>lsblk<escape>}
```

### B. Respond with Text (Natural Language)
Use when user asks a QUESTION or needs GUIDANCE:
- "what disk should I use?" → Explain based on system context
- "what's the difference between ext4 and btrfs?" → Explain filesystems
- "I'm not sure what to do" → Guide through next step

---

## 4. Training Data Categories

**REMINDER: Every category below requires full conversation history in training data.**

### Category A: Direct Commands (with history)
User clearly states what they want. Even "simple" commands need history for context.

| Final User Message | Requires History Because |
|--------------------|-------------------------|
| "list disks" | Even first message needs `messages: [...]` array |
| "format /dev/sda2 as ext4" | Need to verify sda2 was discussed/created |
| "mount it to /mnt" | "it" refers to something from history |
| "set hostname to mypc" | May follow "what should I configure next?" |

### Category B: Pronoun/Reference Resolution (CRITICAL)
User refers to something mentioned earlier. **This is 25% of real usage.**

| User Says | History Needed to Resolve |
|-----------|--------------------------|
| "format it" | What is "it"? The disk we just partitioned |
| "the bigger one" | Which disk was bigger in lsblk output? |
| "yes" | Yes to what? The question assistant just asked |
| "that partition" | Which partition were we discussing? |
| "do the same" | What did we just do? |
| "the second one" | Second of what list? |
| "use ext4" | Format what as ext4? |

### Category C: Natural Language → Command (with history)
User describes intent, LLM determines command based on conversation state.

| User Says | History Provides |
|-----------|-----------------|
| "prepare the disk" | Which disk? What boot mode? |
| "make it bootable" | Which partition? UEFI or BIOS? |
| "give me admin access" | What username? (maybe discussed earlier) |

### Category D: Questions Needing Guidance (with history)
User asks for help. Answer depends on what's been done.

| User Asks | History Needed |
|-----------|---------------|
| "what's next?" | What stage are we at? What's already done? |
| "which filesystem?" | For which partition? What's the use case? |
| "is this right?" | Right about what? What did we just do? |
| "should I proceed?" | Proceed with what? |

### Category E: Error Recovery (with history)
User reports problem. Solution depends on what was attempted.

| User Reports | History Needed |
|--------------|---------------|
| "it failed" | What command failed? What was the error? |
| "mount didn't work" | What mount? What was the target? |
| "permission denied" | On what operation? |
| "disk is busy" | Which disk? What's using it? |

### Category F: Confirmation Flow (with history)
User confirms or denies assistant's question.

| User Says | History Context |
|-----------|----------------|
| "yes" | Assistant asked "should I unmount first?" |
| "no, the other one" | Assistant suggested wrong disk |
| "sure" | Assistant proposed a plan |
| "wait" | User wants to reconsider |

---

## 5. Destructive vs Safe Commands

### Destructive (Require "yes" confirmation in UI)
- `mkfs.*` - Formats destroy data
- `sgdisk`, `fdisk`, `parted` - Partition changes
- `dd` - Raw disk writes
- `wipefs` - Removes filesystem signatures

### Reversible (Can be undone)
| Command | Undo |
|---------|------|
| `mount X Y` | `umount Y` |
| `mkdir -p X` | `rmdir X` |
| `useradd X` | `userdel X` |

### Non-reversible (Data loss)
- All format commands
- All partition commands
- dd operations

---

## 6. Disk Naming Conventions

Training data MUST use realistic disk names:

| Type | Pattern | Examples |
|------|---------|----------|
| SATA/SAS | `/dev/sd[a-z]` | /dev/sda, /dev/sdb |
| NVMe | `/dev/nvme[0-9]n[0-9]` | /dev/nvme0n1, /dev/nvme1n1 |
| VirtIO | `/dev/vd[a-z]` | /dev/vda, /dev/vdb |
| MMC/SD | `/dev/mmcblk[0-9]` | /dev/mmcblk0 |

Partition suffixes:
- SATA: `/dev/sda1`, `/dev/sda2`
- NVMe: `/dev/nvme0n1p1`, `/dev/nvme0n1p2`
- VirtIO: `/dev/vda1`, `/dev/vda2`

---

## 7. Boot Mode Awareness

### UEFI Systems
- Create EFI System Partition (ESP): 512M, FAT32, type ef00
- Mount at `/mnt/boot/efi` or `/mnt/boot`
- Use `bootctl` or `grub-install --target=x86_64-efi`

### BIOS Systems
- No EFI partition needed
- Use MBR partition table (msdos)
- Use `grub-install --target=i386-pc`

Training data MUST match commands to boot mode in system context.

---

## 8. Common User Phrasings

The LLM should understand variations:

### Disk listing
- "list disks", "show disks", "what disks do I have"
- "show storage", "show drives"
- "lsblk", "fdisk -l"

### Partitioning
- "partition the disk", "create partitions"
- "prepare disk for installation"
- "wipe the disk", "start fresh"

### Formatting
- "format as ext4", "make ext4 filesystem"
- "format /dev/sda2", "mkfs /dev/sda2"

### Mounting
- "mount the disk", "mount to /mnt"
- "attach the partition"

### User creation
- "create user", "add user", "make account"
- "create user X with sudo", "give X admin rights"

### Hostname
- "set hostname", "change hostname", "hostname is X"
- "call this machine X"

### Timezone
- "set timezone", "I'm in New York", "timezone America/Chicago"

---

## 9. Training Data Format

**EVERY example is a conversation.** Each training example is a JSONL line with:
- `system_context`: Real system state (disks, boot mode, etc.)
- `messages`: The FULL conversation history including the current query
- `expected_response`: What the LLM should output (command OR text)

### Format A: Command Response
```json
{
  "system_context": "## Current System State\n\n- Boot mode: UEFI\n\n## Available Disks\n\n- /dev/sda: 500G (Samsung SSD)\n- /dev/nvme0n1: 1T (WD Black)",
  "messages": [
    {"role": "user", "content": "list my disks"},
    {"role": "assistant", "content": "$ lsblk\n\nNAME    SIZE TYPE\nsda     500G disk\nnvme0n1   1T disk"},
    {"role": "user", "content": "partition the samsung one for uefi"}
  ],
  "expected_response": {"type": "command", "command": "sgdisk -Z /dev/sda && sgdisk -n 1:0:+512M -t 1:ef00 -n 2:0:0 -t 2:8300 /dev/sda"}
}
```

### Format B: Text Response (Question/Clarification)
```json
{
  "system_context": "## Current System State\n\n- Boot mode: UEFI\n\n## Available Disks\n\n- /dev/sda: 500G (Samsung SSD)\n- /dev/nvme0n1: 1T (WD Black)",
  "messages": [
    {"role": "user", "content": "I want to install linux"}
  ],
  "expected_response": {"type": "text", "response": "I can help you install Linux. You have two disks available:\n\n- /dev/sda: 500GB Samsung SSD\n- /dev/nvme0n1: 1TB WD Black NVMe\n\nWhich disk would you like to use for the installation?"}
}
```

### Format C: Mid-Conversation with Pronouns
```json
{
  "system_context": "...",
  "messages": [
    {"role": "user", "content": "show disks"},
    {"role": "assistant", "content": "$ lsblk\n\nNAME        SIZE\nsda         500G\nnvme0n1       1T"},
    {"role": "user", "content": "use the bigger one"},
    {"role": "assistant", "content": "$ sgdisk -Z /dev/nvme0n1\n\nPartition table wiped."},
    {"role": "user", "content": "create partitions"},
    {"role": "assistant", "content": "$ sgdisk -n 1:0:+512M -t 1:ef00 -n 2:0:0 -t 2:8300 /dev/nvme0n1\n\nCreated EFI (512M) and root partitions."},
    {"role": "user", "content": "format them"}
  ],
  "expected_response": {"type": "command", "command": "mkfs.fat -F32 /dev/nvme0n1p1 && mkfs.ext4 /dev/nvme0n1p2"}
}
```

### Key Points
1. **messages array is ALWAYS present** - even for "first" messages (array of 1)
2. **Assistant messages include command output** - so LLM learns what results look like
3. **System context updates** - reflects current mount state, etc.
4. **Pronouns resolve from history** - "it", "that", "the first one" all need context

---

## 10. Quality Requirements

### Must Do
1. Commands must be syntactically correct
2. Disk references must match system context
3. UEFI commands for UEFI systems, BIOS commands for BIOS systems
4. Partition before format before mount (correct order)
5. Mount root before subdirectories

### Must NOT Do
1. Hallucinate disk names not in system context
2. Suggest UEFI commands on BIOS systems (and vice versa)
3. Suggest formatting mounted partitions
4. Create users without home directories (unless specifically asked)
5. Skip partition steps and go straight to formatting

---

## 11. Example Training Data (JSONL Format)

Each line below is a separate training example. Note how **each example is a snapshot** of the conversation at that moment.

### Example Set: Full UEFI Installation Flow

**Snapshot 1 - User asks to install**
```json
{"system_context": "- Boot mode: UEFI\n\n## Available Disks\n- /dev/sda: 500G (Samsung SSD)", "messages": [{"role": "user", "content": "I want to install on sda"}], "expected_response": {"type": "text", "response": "I'll prepare /dev/sda for a UEFI installation. First, let me wipe the partition table and create a fresh GPT layout."}}
```

**Snapshot 2 - After assistant responded, user confirms**
```json
{"system_context": "- Boot mode: UEFI\n\n## Available Disks\n- /dev/sda: 500G (Samsung SSD)", "messages": [{"role": "user", "content": "I want to install on sda"}, {"role": "assistant", "content": "I'll prepare /dev/sda for a UEFI installation. First, let me wipe the partition table."}, {"role": "user", "content": "ok do it"}], "expected_response": {"type": "command", "command": "sgdisk -Z /dev/sda"}}
```

**Snapshot 3 - After partition table wiped**
```json
{"system_context": "- Boot mode: UEFI\n\n## Available Disks\n- /dev/sda: 500G (Samsung SSD) - no partitions", "messages": [{"role": "user", "content": "I want to install on sda"}, {"role": "assistant", "content": "I'll prepare /dev/sda for a UEFI installation."}, {"role": "user", "content": "ok do it"}, {"role": "assistant", "content": "$ sgdisk -Z /dev/sda\nGPT data structures destroyed."}, {"role": "user", "content": "now what"}], "expected_response": {"type": "command", "command": "sgdisk -n 1:0:+512M -t 1:ef00 -n 2:0:0 -t 2:8300 /dev/sda"}}
```

**Snapshot 4 - Partitions created, user says "format them"**
```json
{"system_context": "- Boot mode: UEFI\n\n## Available Disks\n- /dev/sda: 500G\n  - /dev/sda1: 512M\n  - /dev/sda2: 499.5G", "messages": [{"role": "user", "content": "I want to install on sda"}, {"role": "assistant", "content": "..."}, {"role": "user", "content": "ok do it"}, {"role": "assistant", "content": "$ sgdisk -Z /dev/sda\n..."}, {"role": "user", "content": "now what"}, {"role": "assistant", "content": "$ sgdisk -n 1:0:+512M -t 1:ef00 -n 2:0:0 /dev/sda\nCreated EFI and root partitions."}, {"role": "user", "content": "format them"}], "expected_response": {"type": "command", "command": "mkfs.fat -F32 /dev/sda1 && mkfs.ext4 /dev/sda2"}}
```

### Example Set: Pronoun Resolution

**User says "it" - must resolve from context**
```json
{"system_context": "...", "messages": [{"role": "user", "content": "list disks"}, {"role": "assistant", "content": "$ lsblk\nsda    500G\nnvme0n1  1T"}, {"role": "user", "content": "use the nvme"}, {"role": "assistant", "content": "$ sgdisk -Z /dev/nvme0n1\nWiped."}, {"role": "user", "content": "partition it"}], "expected_response": {"type": "command", "command": "sgdisk -n 1:0:+512M -t 1:ef00 -n 2:0:0 -t 2:8300 /dev/nvme0n1"}}
```

**User says "yes" - must know what question was asked**
```json
{"system_context": "## Current Mounts\n- /dev/sda2 mounted at /mnt", "messages": [{"role": "user", "content": "format /dev/sda2"}, {"role": "assistant", "content": "I can't format /dev/sda2 while it's mounted. Should I unmount it first?"}, {"role": "user", "content": "yes"}], "expected_response": {"type": "command", "command": "umount /mnt"}}
```

### Example Set: Asking Clarifying Questions

**Ambiguous request - LLM asks for clarification**
```json
{"system_context": "- Boot mode: UEFI\n\n## Available Disks\n- /dev/sda: 256G (SATA)\n- /dev/nvme0n1: 1T (NVMe)", "messages": [{"role": "user", "content": "partition the disk"}], "expected_response": {"type": "text", "response": "You have two disks available:\n- /dev/sda: 256GB SATA\n- /dev/nvme0n1: 1TB NVMe\n\nWhich one would you like to partition?"}}
```

**After clarification - LLM proceeds**
```json
{"system_context": "- Boot mode: UEFI\n\n## Available Disks\n- /dev/sda: 256G\n- /dev/nvme0n1: 1T", "messages": [{"role": "user", "content": "partition the disk"}, {"role": "assistant", "content": "You have two disks. Which one would you like to partition?"}, {"role": "user", "content": "the terabyte one"}], "expected_response": {"type": "command", "command": "sgdisk -Z /dev/nvme0n1"}}
```

---

## 12. Anti-Patterns (What NOT to Train)

### Bad: Hallucinated disks
```
Context: Only /dev/sda exists
User: "format sdb"
LLM: mkfs.ext4 /dev/sdb   # WRONG - sdb doesn't exist
```

### Bad: Wrong boot mode
```
Context: BIOS system
User: "install bootloader"
LLM: bootctl install   # WRONG - bootctl is UEFI only
```

### Bad: Wrong command order
```
User: "mount /dev/sda2 /mnt/home"
LLM: mount /dev/sda2 /mnt/home   # WRONG if /mnt doesn't exist
```

### Bad: Over-explaining simple commands
```
User: "lsblk"
LLM: (TEXT) "Let me explain what lsblk does..."   # WRONG - just run it
```

---

## Summary: Training Data Structure

### ALL training data is multi-turn conversation

Every single example has a `messages[]` array with full conversation history. The categories below describe the TYPE of the final user message:

| Final User Message Type | % of Data | Example |
|------------------------|-----------|---------|
| Direct command request | 30% | "format /dev/sda2 as ext4" |
| Pronoun/reference ("it", "that one") | 25% | "format it", "the bigger one", "yes" |
| Natural language intent | 20% | "prepare the disk for installation" |
| Questions needing guidance | 15% | "which filesystem should I use?" |
| Error recovery | 10% | "it failed", "permission denied" |

### Conversation Length Distribution

| Messages in History | % of Data | Use Case |
|--------------------|-----------|----------|
| 1-2 messages | 20% | Opening queries, simple tasks |
| 3-6 messages | 50% | Typical task (partition→format→mount) |
| 7-12 messages | 25% | Full stage completion |
| 13+ messages | 5% | Complex multi-stage sessions |

### Total Recommended
- **Minimum:** 500 conversation snapshots
- **Recommended:** 1000+ conversation snapshots
- **Each snapshot** = one training example with FULL history up to that point

---

## 13. Conversation Templates & Placeholders

**Source conversations use placeholders** that the augmentor fills in based on system context. This allows one conversation template to generate multiple training variations.

### Why Placeholders?

The model must learn to use REAL system facts, not memorize hardcoded disk names. If all training data uses `/dev/sda`, the model won't generalize to `/dev/nvme0n1` or `/dev/vda`.

```
# BAD - Hardcoded (model memorizes "sda")
{"user": "partition the disk", "command": "sgdisk -Z /dev/sda"}

# GOOD - Placeholder (model learns to use system context)
{"user": "partition the disk", "command": "sgdisk -Z {PRIMARY_DISK}"}
```

### Available Placeholders

| Placeholder | Filled From | Example Values |
|-------------|-------------|----------------|
| `{PRIMARY_DISK}` | First disk in system context | `/dev/sda`, `/dev/nvme0n1`, `/dev/vda` |
| `{SECONDARY_DISK}` | Second disk (if exists) | `/dev/sdb`, `/dev/nvme1n1` |
| `{BOOT_PARTITION}` | EFI/boot partition | `/dev/sda1`, `/dev/nvme0n1p1` |
| `{ROOT_PARTITION}` | Root partition | `/dev/sda2`, `/dev/nvme0n1p2` |
| `{BOOT_MODE}` | System boot mode | `UEFI`, `Legacy BIOS` |
| `{DISK_SIZE}` | Primary disk size | `256G`, `500G`, `1T`, `2T` |
| `{DISK_MODEL}` | Primary disk model | `Samsung SSD 870`, `WD Black SN850` |
| `{HOSTNAME}` | User-provided hostname | varies |
| `{USERNAME}` | User-provided username | varies |
| `{TIMEZONE}` | User-provided timezone | `America/New_York`, `Europe/London` |

### Partition Naming Rules

The augmentor must generate correct partition names based on disk type:

| Disk Type | Disk Device | Partition 1 | Partition 2 |
|-----------|-------------|-------------|-------------|
| SATA/SAS | `/dev/sda` | `/dev/sda1` | `/dev/sda2` |
| NVMe | `/dev/nvme0n1` | `/dev/nvme0n1p1` | `/dev/nvme0n1p2` |
| VirtIO | `/dev/vda` | `/dev/vda1` | `/dev/vda2` |
| MMC/SD | `/dev/mmcblk0` | `/dev/mmcblk0p1` | `/dev/mmcblk0p2` |

### Boot Mode Conditional Commands

Some commands depend on boot mode. Use conditional placeholders:

```json
{
  "user": "install bootloader",
  "type": "command",
  "command": "{BOOTLOADER_INSTALL}"
}
```

Where `{BOOTLOADER_INSTALL}` expands to:
- UEFI: `arch-chroot /mnt bootctl install`
- BIOS: `arch-chroot /mnt grub-install --target=i386-pc {PRIMARY_DISK}`

### Augmentor Responsibilities

The `augment_data.py` script must:

1. **Generate system context variations:**
   - SATA disk systems (`/dev/sda`)
   - NVMe disk systems (`/dev/nvme0n1`)
   - VirtIO/VM systems (`/dev/vda`)
   - Single disk vs multi-disk
   - UEFI vs Legacy BIOS

2. **Fill placeholders** based on generated context

3. **Update context as commands execute:**
   - After `sgdisk`: show new partitions
   - After `mkfs`: show filesystem types
   - After `mount`: show mount points
   - After hostname/timezone/user commands: update those fields

4. **Validate commands match context:**
   - UEFI commands only on UEFI systems
   - BIOS commands only on BIOS systems
   - Correct partition naming for disk type

### Example Template Expansion

**Template:**
```json
{
  "system_context": "- Boot mode: {BOOT_MODE}\n- /dev/{PRIMARY_DISK}: {DISK_SIZE} ({DISK_MODEL})",
  "turns": [
    {"user": "partition the disk", "type": "command", "command": "sgdisk -Z /dev/{PRIMARY_DISK}"}
  ]
}
```

**Generated variations:**
```json
// Variation 1: SATA + UEFI
{"system_context": "- Boot mode: UEFI\n- /dev/sda: 500G (Samsung 870)", ...}
{"command": "sgdisk -Z /dev/sda"}

// Variation 2: NVMe + UEFI
{"system_context": "- Boot mode: UEFI\n- /dev/nvme0n1: 1T (WD Black)", ...}
{"command": "sgdisk -Z /dev/nvme0n1"}

// Variation 3: VirtIO + UEFI
{"system_context": "- Boot mode: UEFI\n- /dev/vda: 40G (VirtIO)", ...}
{"command": "sgdisk -Z /dev/vda"}
```

This ensures the model learns to read the system context and use the correct device names, not memorize hardcoded values.
