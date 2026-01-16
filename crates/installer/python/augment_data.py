#!/usr/bin/env python3
"""
Generate augmented training data for the installer LLM.
Creates variations of queries through paraphrasing, typos, and different phrasings.

IMPORTANT: Each example includes a dynamic system context (disks, boot mode, etc.)
to match what the model will see in production.
"""

import json
import random
import re
from pathlib import Path

# =============================================================================
# SYNTHETIC SYSTEM CONTEXTS
# The model needs to learn with realistic system state injected into prompts
# =============================================================================

SYNTHETIC_DISKS = [
    # Single SSD
    {
        "blockdevices": [
            {"name": "sda", "size": "500G", "type": "disk", "model": "Samsung SSD 870"},
        ]
    },
    # SSD + HDD
    {
        "blockdevices": [
            {"name": "sda", "size": "500G", "type": "disk", "model": "Samsung SSD 870"},
            {"name": "sdb", "size": "2T", "type": "disk", "model": "WD Blue HDD"},
        ]
    },
    # NVMe only
    {
        "blockdevices": [
            {"name": "nvme0n1", "size": "1T", "type": "disk", "model": "Samsung 980 PRO"},
        ]
    },
    # NVMe + SATA
    {
        "blockdevices": [
            {"name": "nvme0n1", "size": "512G", "type": "disk", "model": "WD Black SN850"},
            {"name": "sda", "size": "1T", "type": "disk", "model": "Crucial MX500"},
        ]
    },
    # Multiple NVMe
    {
        "blockdevices": [
            {"name": "nvme0n1", "size": "1T", "type": "disk", "model": "Samsung 980 PRO"},
            {"name": "nvme1n1", "size": "2T", "type": "disk", "model": "Samsung 990 PRO"},
        ]
    },
    # Already partitioned disk
    {
        "blockdevices": [
            {
                "name": "sda", "size": "500G", "type": "disk", "model": "Samsung SSD",
                "children": [
                    {"name": "sda1", "size": "512M", "type": "part", "fstype": "vfat", "mountpoint": None},
                    {"name": "sda2", "size": "499.5G", "type": "part", "fstype": "ext4", "mountpoint": None},
                ]
            },
        ]
    },
    # Windows dual-boot scenario
    {
        "blockdevices": [
            {
                "name": "nvme0n1", "size": "1T", "type": "disk", "model": "Samsung 970 EVO",
                "children": [
                    {"name": "nvme0n1p1", "size": "100M", "type": "part", "fstype": "vfat", "mountpoint": None},
                    {"name": "nvme0n1p2", "size": "16M", "type": "part", "fstype": None, "mountpoint": None},
                    {"name": "nvme0n1p3", "size": "500G", "type": "part", "fstype": "ntfs", "mountpoint": None},
                    {"name": "nvme0n1p4", "size": "500G", "type": "part", "fstype": None, "mountpoint": None},
                ]
            },
        ]
    },
    # Small drive (VM or old laptop)
    {
        "blockdevices": [
            {"name": "vda", "size": "32G", "type": "disk", "model": None},
        ]
    },
]

def generate_system_context() -> str:
    """Generate a random synthetic system context for training."""
    # Random disk config
    disks = random.choice(SYNTHETIC_DISKS)

    # Random boot mode (80% UEFI, 20% legacy)
    uefi = random.random() < 0.8

    # Random network (90% connected)
    network = random.random() < 0.9

    # Build context string (matches llm_server.py format exactly)
    lines = ["## Current System State\n"]

    if uefi:
        lines.append("- Boot mode: UEFI")
    else:
        lines.append("- Boot mode: Legacy BIOS")

    if network:
        lines.append("- Network: Connected")
    else:
        lines.append("- Network: Not connected")

    lines.append("- Hostname: archiso")
    lines.append("- Timezone: not set")

    lines.append("\n## Available Disks\n")
    for dev in disks["blockdevices"]:
        if dev.get("type") == "disk":
            model = (dev.get("model") or "").strip() or "Unknown"
            lines.append(f"- /dev/{dev['name']}: {dev['size']} ({model})")
            if "children" in dev:
                for part in dev["children"]:
                    mp = part.get("mountpoint", "")
                    fs = part.get("fstype", "")
                    mount_info = f" mounted at {mp}" if mp else ""
                    fs_info = f" [{fs}]" if fs else ""
                    lines.append(f"  - /dev/{part['name']}: {part['size']}{fs_info}{mount_info}")

    return "\n".join(lines)


def get_random_disk_from_context(context: str) -> str:
    """Extract a random disk name from the system context."""
    import re
    disks = re.findall(r'/dev/(\w+):', context)
    # Filter to only disk devices (not partitions)
    disks = [d for d in disks if not any(c.isdigit() for c in d) or d.startswith('nvme')]
    if disks:
        return f"/dev/{random.choice(disks)}"
    return "/dev/sda"  # fallback

# Base templates with variations
DISK_QUERIES = {
    "list": [
        "list disks", "show disks", "show me disks", "what disks", "which disks",
        "display disks", "view disks", "get disks", "find disks", "disks please",
        "disk list", "all disks", "available disks", "my disks", "the disks",
        "drives", "list drives", "show drives", "what drives", "my drives",
        "storage", "show storage", "list storage", "available storage",
        "hard drives", "show hard drives", "list hard drives",
        "block devices", "list block devices", "show block devices",
        "lsblk", "fdisk -l", "what do I have",
        "what disks do I have", "show me what disks I have",
        "what drives are available", "which drives can I use",
        "what storage devices are connected", "show connected drives",
        "can you list my disks", "can you show me the disks",
        "I need to see the disks", "let me see the disks",
        "what are my options for disks", "disk options",
    ],
    "details": [
        "disk details", "show disk details", "disk information",
        "disk info", "drive info", "storage details",
        "show disk info", "detailed disk list", "verbose disk list",
        "more disk info", "disk specs", "drive specifications",
    ],
    "space": [
        "disk space", "free space", "available space", "how much space",
        "storage space", "space left", "disk usage", "show space",
        "check space", "how much storage", "remaining space",
    ]
}

PARTITION_QUERIES = {
    "start": [
        "partition disk", "partition the disk", "start partitioning",
        "create partitions", "make partitions", "setup partitions",
        "partition {disk}", "partition {disk_desc}",
        "I want to partition", "let's partition", "begin partitioning",
        "open partition editor", "start fdisk", "run fdisk",
    ],
    "whole_disk": [
        "use whole disk", "use the whole disk", "use entire disk",
        "use all of it", "use the whole thing", "full disk",
        "take the whole disk", "entire disk please", "all of {disk}",
        "use the whole {disk_desc}", "entire {disk_desc}",
        "I want to use the whole disk", "give me the whole disk",
    ],
    "efi": [
        "create efi partition", "make efi partition", "add efi",
        "efi partition", "boot partition", "create boot partition",
        "512mb efi", "make a 512mb efi partition",
    ],
    "root": [
        "create root partition", "make root partition", "add root",
        "root partition", "main partition", "system partition",
        "use rest for root", "remaining space for root",
    ],
    "swap": [
        "create swap", "add swap", "swap partition", "make swap",
        "{size} swap", "swap {size}", "{size} swap partition",
        "I need swap", "give me swap", "create a swap partition",
    ],
    "home": [
        "separate home", "home partition", "create home partition",
        "split home", "dedicated home partition",
        "{size} for home", "make home partition",
    ],
}

FORMAT_QUERIES = {
    "ext4": [
        "format as ext4", "format ext4", "use ext4", "ext4 please",
        "ext4 filesystem", "make it ext4", "format {part} ext4",
        "format {part} as ext4", "ext4 for {part}",
    ],
    "btrfs": [
        "format as btrfs", "format btrfs", "use btrfs", "btrfs please",
        "btrfs filesystem", "make it btrfs", "format {part} btrfs",
        "I want btrfs", "give me btrfs", "btrfs for root",
    ],
    "xfs": [
        "format as xfs", "format xfs", "use xfs", "xfs please",
        "xfs filesystem", "make it xfs", "format {part} xfs",
    ],
    "fat32": [
        "format as fat32", "format fat32", "vfat", "format as vfat",
        "fat32 for efi", "fat32 for boot",
    ],
}

ENCRYPT_QUERIES = [
    "encrypt", "encrypted", "encryption", "luks", "use luks",
    "encrypt root", "encrypt the disk", "encrypted root",
    "encrypted partition", "full disk encryption", "fde",
    "I want encryption", "enable encryption", "setup encryption",
    "luks encryption", "encrypted install", "secure install",
    "encrypt everything", "encrypt the whole disk",
    "encrypted root partition", "luks encrypted root",
]

TIMEZONE_TEMPLATES = [
    "timezone {tz}", "set timezone {tz}", "set timezone to {tz}",
    "timezone is {tz}", "my timezone is {tz}", "I'm in {tz}",
    "im in {tz}", "i live in {tz}", "located in {tz}",
    "{tz} timezone", "{tz} time", "use {tz} timezone",
    "change timezone to {tz}", "set tz to {tz}", "tz {tz}",
]

TIMEZONES = {
    "los angeles": "America/Los_Angeles",
    "la": "America/Los_Angeles",
    "san francisco": "America/Los_Angeles",
    "sf": "America/Los_Angeles",
    "seattle": "America/Los_Angeles",
    "portland": "America/Los_Angeles",
    "california": "America/Los_Angeles",
    "pacific": "America/Los_Angeles",
    "new york": "America/New_York",
    "ny": "America/New_York",
    "nyc": "America/New_York",
    "boston": "America/New_York",
    "miami": "America/New_York",
    "eastern": "America/New_York",
    "chicago": "America/Chicago",
    "dallas": "America/Chicago",
    "houston": "America/Chicago",
    "central": "America/Chicago",
    "denver": "America/Denver",
    "phoenix": "America/Denver",
    "mountain": "America/Denver",
    "london": "Europe/London",
    "uk": "Europe/London",
    "england": "Europe/London",
    "berlin": "Europe/Berlin",
    "germany": "Europe/Berlin",
    "paris": "Europe/Paris",
    "france": "Europe/Paris",
    "tokyo": "Asia/Tokyo",
    "japan": "Asia/Tokyo",
    "sydney": "Australia/Sydney",
    "australia": "Australia/Sydney",
    "toronto": "America/Toronto",
    "canada": "America/Toronto",
    "vancouver": "America/Vancouver",
    "utc": "UTC",
    "gmt": "UTC",
}

HOSTNAME_TEMPLATES = [
    "hostname {name}", "set hostname {name}", "hostname is {name}",
    "set hostname to {name}", "name it {name}", "call it {name}",
    "machine name {name}", "computer name {name}", "name the system {name}",
    "my hostname is {name}", "the hostname should be {name}",
]

SAMPLE_HOSTNAMES = [
    "mypc", "my-pc", "my-laptop", "laptop", "desktop",
    "workstation", "server", "homeserver", "devbox",
    "archbox", "linuxbox", "levitate", "main", "dev",
]

USER_TEMPLATES = [
    "create user {name}", "add user {name}", "new user {name}",
    "user {name}", "make user {name}", "create account {name}",
    "add account {name}", "username {name}", "my username is {name}",
    "create user {name} with sudo", "add user {name} with sudo",
    "user {name} with admin", "make {name} an admin",
    "{name} should have sudo", "give {name} sudo access",
]

SAMPLE_USERNAMES = [
    "user", "admin", "vince", "john", "alice", "bob",
    "me", "dev", "main", "default",
]

BOOT_QUERIES = [
    "install bootloader", "setup bootloader", "bootloader",
    "install boot", "setup boot", "configure bootloader",
    "grub", "install grub", "use grub", "setup grub",
    "systemd-boot", "install systemd-boot", "use systemd-boot",
    "bootctl install", "boot manager", "install boot manager",
]

INSTALL_QUERIES = [
    "install", "install system", "install the system",
    "copy system", "copy the system", "copy files",
    "start install", "begin install", "do the install",
    "rsync", "start copying", "copy to disk",
    "install levitate", "install levitateos",
]

FINISH_QUERIES = [
    "done", "finished", "complete", "im done", "i'm done",
    "all done", "that's it", "thats it", "finish",
    "finalize", "wrap up", "end", "close",
]

REBOOT_QUERIES = [
    "reboot", "restart", "reboot now", "restart now",
    "reboot please", "restart please", "reboot system",
    "restart system", "boot into new system",
]

HELP_QUERIES = [
    "help", "help me", "what can you do", "how does this work",
    "explain", "guide me", "instructions", "what now",
    "what next", "next step", "what should I do",
    "im lost", "i'm lost", "confused", "im confused",
    "where do I start", "how to start", "getting started",
]

CONFIRM_POSITIVE = [
    "yes", "y", "yeah", "yep", "yup", "sure", "ok", "okay",
    "proceed", "continue", "go ahead", "do it", "confirmed",
    "affirmative", "correct", "right", "exactly", "perfect",
]

CONFIRM_NEGATIVE = [
    "no", "n", "nope", "nah", "cancel", "stop", "abort",
    "don't", "dont", "wait", "hold on", "never mind",
    "nevermind", "forget it", "not yet",
]

GREETINGS = [
    "hello", "hi", "hey", "yo", "sup", "greetings",
    "good morning", "good afternoon", "good evening",
    "howdy", "hi there", "hello there",
]

FAREWELLS = [
    "bye", "goodbye", "see you", "cya", "later",
    "thanks bye", "thank you bye", "exit", "quit", "close",
]

THANKS = [
    "thanks", "thank you", "thx", "ty", "appreciate it",
    "thanks a lot", "thank you so much", "cheers",
]

# QUESTION QUERIES - User questions that need helpful responses (NO commands)
# These should make up ~75% of training data
QUESTION_QUERIES = {
    # Disk questions
    "how do I see my disks?": "To see your disks, I'll run lsblk. It shows all block devices and their sizes.",
    "what disks do I have?": "Let me show you the available disks and their sizes.",
    "which disk should I use?": "I can show you your disks with lsblk. Usually you'd want to use the largest one for your main installation.",
    "where are my drives?": "I'll list your storage devices. They're usually /dev/sda, /dev/nvme0n1, etc.",
    "can you show me my storage?": "Of course! Here are your storage devices.",
    "what's the difference between sda and nvme?": "sda is usually a SATA drive (SSD or HDD), while nvme is a faster NVMe SSD connected directly to PCIe.",
    "how much storage do I have?": "Let me check your disk sizes and available space.",
    "is my disk big enough?": "I'll show you your disk sizes. You'll need at least 10GB for a minimal install, 20GB+ recommended.",

    # Partition questions
    "how do I partition?": "To partition a disk, first tell me which disk (e.g., 'partition /dev/sda'). I'll help you create the layout.",
    "what partitions do I need?": "For a basic install: an EFI partition (512MB) and a root partition. Optionally: swap and separate /home.",
    "how big should my partitions be?": "EFI: 512MB, Swap: equal to RAM (optional), Root: rest of disk. For separate /home, split root/home as needed.",
    "what is an EFI partition?": "The EFI System Partition (ESP) is a FAT32 partition where the bootloader lives. Needed for UEFI boot.",
    "do I need swap?": "Swap is optional but recommended if you have less than 16GB RAM or want hibernation. Usually 4-8GB.",
    "should I use the whole disk?": "Using the whole disk is simpler. I'll create EFI + root partitions. All existing data will be erased.",
    "can I dual boot?": "Yes! Create partitions on free space instead of using the whole disk. Keep your existing OS partitions.",
    "what happens to my data?": "Partitioning will erase data on the partitions you modify. Back up important files first!",
    "how do I make a home partition?": "When partitioning, specify a size for home like '100GB for home' or 'separate home partition'.",

    # Filesystem questions
    "what filesystem should I use?": "ext4 is the most common and reliable choice. btrfs offers snapshots. xfs is great for large files.",
    "which is better ext4 or btrfs?": "ext4: proven, simple, fast. btrfs: snapshots, compression, but slightly more complex. Both are good choices.",
    "how do I format a partition?": "Tell me which partition and filesystem, like 'format /dev/sda2 as ext4' or 'format root ext4'.",
    "what is btrfs good for?": "btrfs offers snapshots (rollback), compression, checksums, and easy resizing. Great for root partition.",
    "can I use xfs?": "Yes! XFS is excellent for large files and high performance. Say 'format as xfs' to use it.",
    "why fat32 for efi?": "The UEFI specification requires FAT32 for the EFI System Partition. It's the standard boot filesystem.",

    # Encryption questions
    "should I encrypt?": "Encryption (LUKS) protects your data if the device is lost/stolen. Recommended for laptops.",
    "how does encryption work?": "LUKS encrypts your root partition. You'll enter a password at boot to unlock it.",
    "what is luks?": "LUKS (Linux Unified Key Setup) is the standard disk encryption for Linux. Secure and well-supported.",
    "will encryption slow things down?": "Modern CPUs have AES hardware acceleration. You might see 1-5% overhead, barely noticeable.",
    "can I encrypt later?": "It's much easier to set up encryption during install. Converting later requires backup and restore.",

    # User questions
    "how do I create a user?": "Tell me the username, like 'create user john' or 'add user alice with sudo'.",
    "what username should I use?": "Pick any name you like! It should be lowercase, letters and numbers only.",
    "how do I give sudo access?": "Add 'with sudo' or 'as admin' when creating the user, like 'create user bob with sudo'.",
    "what is the wheel group?": "The wheel group grants sudo privileges on most Linux systems. Admin users are added to wheel.",
    "can I have multiple users?": "Yes! Create each user with 'create user <name>'. Add 'with sudo' for admin access.",
    "what's the difference between admin and regular user?": "Admin users (in wheel group) can use sudo to run commands as root. Regular users cannot.",

    # Bootloader questions
    "what is a bootloader?": "The bootloader (like systemd-boot or GRUB) loads the OS kernel at startup. It's installed to the EFI partition.",
    "which bootloader should I use?": "systemd-boot is simple and fast for UEFI systems. GRUB is more feature-rich and supports legacy BIOS.",
    "how do I install the bootloader?": "Say 'install bootloader' and I'll set up systemd-boot on your EFI partition.",
    "what is grub?": "GRUB (GRand Unified Bootloader) is a popular bootloader with extensive customization options.",
    "do I need to install grub?": "If you have UEFI, systemd-boot is simpler. For BIOS boot or dual-boot, GRUB might be better.",

    # Installation questions
    "how long will install take?": "The file copy usually takes 2-5 minutes depending on disk speed. The whole process takes 10-20 min.",
    "what gets installed?": "The base system, kernel, and essential packages. You can add more software after first boot.",
    "can I choose what to install?": "This installer sets up the base system. Customize packages after installation via package manager.",
    "what happens during install?": "I copy system files to your disk, configure the bootloader, set timezone, create user, etc.",
    "is the install safe?": "The installation only writes to partitions you specify. Other disks remain untouched.",

    # Timezone questions
    "how do I set timezone?": "Tell me your location, like 'timezone los angeles' or 'set timezone to new york'.",
    "what timezone am I in?": "Tell me your city or region and I'll set the correct timezone for you.",
    "why does timezone matter?": "The timezone affects your system clock display, logs, and scheduled tasks. Pick where you're located.",
    "can I change timezone later?": "Yes, but it's easiest to set it now. After install: timedatectl set-timezone <zone>.",

    # Hostname questions
    "what is a hostname?": "The hostname is your computer's name on the network. Pick something memorable like 'mypc' or 'laptop'.",
    "how do I set hostname?": "Say 'hostname mypc' or 'call it laptop' to set the name.",
    "what should I name my computer?": "Anything you like! Common choices: laptop, desktop, workstation, or something creative.",

    # Process questions
    "what do I do first?": "Start by running 'list disks' to see your storage. Then we'll partition and install.",
    "what's the next step?": "That depends on where we are. What have you done so far?",
    "am I doing this right?": "Tell me what you've done and I'll guide you to the next step.",
    "can you walk me through this?": "Sure! Step 1: 'list disks', Step 2: partition, Step 3: format, Step 4: install, Step 5: configure.",
    "how do I start over?": "We can start fresh. What would you like to do differently?",
    "is there an undo?": "Most actions can't be undone once applied. That's why I ask for confirmation on destructive operations.",

    # General help
    "what can you help with?": "I can help with disk partitioning, formatting, installation, users, timezone, hostname, and bootloader setup.",
    "what are my options?": "You can: list disks, partition, format, install system, create users, set timezone/hostname, install bootloader.",
    "i'm stuck": "No problem! Tell me what you're trying to do and I'll help guide you.",
    "this is confusing": "Let's slow down. What's the main thing you're trying to accomplish?",
    "where can I get more help?": "The LevitateOS documentation has detailed guides. Or ask me specific questions!",

    # === REALISTIC CONFUSED USER QUESTIONS ===

    # Genuinely confused users
    "i dont understand": "No worries! What part is confusing? I can explain step by step.",
    "I don't know what to do": "Let's start simple: say 'list disks' to see your storage options.",
    "wait what": "Let me explain. What specifically confused you?",
    "huh?": "I'll clarify. What would you like me to explain?",
    "???": "Something confusing? Tell me what you need help with.",
    "idk": "No problem! Would you like me to guide you through the installation step by step?",
    "um": "Take your time! What would you like to do?",
    "uhh": "No rush. What are you trying to accomplish?",

    # Worried/scared users
    "will this delete my files?": "Only if you partition/format that disk. I'll always warn you before anything destructive.",
    "am I going to break something?": "I'll ask for confirmation before any destructive action. You can always say 'no' to cancel.",
    "is this safe?": "Yes! I won't do anything destructive without asking first. You're in control.",
    "I'm scared to mess up": "That's understandable! I'll guide you step by step and ask before anything risky.",
    "what if I do something wrong?": "Don't worry - I ask for confirmation before destructive actions. You can always cancel.",
    "i dont want to lose my data": "I won't touch your data without explicit confirmation. We can partition carefully to preserve it.",
    "can I go back?": "Some actions can't be undone (like formatting), but I'll warn you before those.",
    "I'm nervous": "That's okay! Take your time. I'll explain each step before we do anything.",

    # Technical confusion
    "what does sda mean?": "sda is the first SATA/SCSI drive. sdb would be the second. nvme0n1 is an NVMe drive.",
    "what's sda1?": "sda1 is the first partition on drive sda. The number indicates the partition, the letter is the drive.",
    "whats the difference between sda and sda1?": "sda is the whole disk. sda1 is a partition (a section) on that disk.",
    "I don't know which disk is which": "Try 'list disks' - it shows sizes which helps identify them. The bigger one is usually for installing.",
    "which one is my ssd?": "SSDs are typically smaller and have names like nvme0n1 or appear as faster devices. Check the sizes with 'list disks'.",
    "what does partition mean?": "A partition is like dividing a disk into sections. You need at least one for the operating system.",
    "what's a filesystem?": "A filesystem (like ext4) organizes how data is stored on disk. Different ones have different features.",
    "what does format mean?": "Formatting prepares a partition for use by creating a filesystem. Warning: this erases existing data!",
    "what's mount?": "Mounting connects a partition to a folder so you can access its contents. Like plugging in a USB drive.",
    "what does root mean?": "In Linux, 'root' can mean: the / directory (top folder), the root user (admin), or the main partition.",

    # Process confusion
    "wait what am i supposed to do": "Let's restart: 1) list disks, 2) partition, 3) format, 4) install. What step are you on?",
    "I'm lost": "No problem! Tell me what you've done so far and I'll guide you to the next step.",
    "whats next": "What have you completed? I'll tell you the next step.",
    "what now": "Tell me what you've done and I'll suggest the next step.",
    "did it work?": "What were you trying to do? I can help verify if it succeeded.",
    "is it done?": "What operation were you waiting on? I can check the status.",
    "now what": "What's the last thing you did? I'll help with the next step.",

    # Specific installation confusion
    "which disk do I pick?": "Usually the one you want to install to - often your main SSD. 'list disks' shows sizes to help identify.",
    "do I partition all of them?": "Only the disk you're installing to! Leave other disks alone if they have data.",
    "do I have to create a user?": "Yes, you need at least one user to log in after installation.",
    "can I skip the bootloader?": "No - without a bootloader, your system won't be able to start after installation.",
    "what if I already have linux?": "You can replace it (use whole disk) or install alongside (manual partitioning).",
    "I have windows": "For dual boot, partition manually to use free space. Don't format your Windows partition!",
    "where does it install to?": "The system installs to the partitions you create and mount to /mnt.",
    "how do I boot after install?": "After installing the bootloader, just reboot - the system will start automatically.",

    # Error/problem questions
    "it says error": "What's the error message? That will help me understand what went wrong.",
    "something went wrong": "What were you trying to do? Tell me the error and I'll help fix it.",
    "it's not working": "What specifically isn't working? What did you try to do?",
    "why isn't it working?": "I need more details - what command did you run and what happened?",
    "it failed": "What operation failed? I can help troubleshoot.",
    "help I messed up": "Stay calm! What did you do? Most things can be fixed or redone.",
    "I think I broke it": "What happened? Describe what you did and I'll help assess the situation.",
    "disk not found": "Make sure the disk is connected. Try 'list disks' to see available devices.",
    "permission denied": "Some operations need confirmation. Try the command again and confirm when prompted.",

    # Vague/unclear questions
    "what?": "I'm here to help! What would you like to know?",
    "tell me more": "About what specifically? Disks, partitions, filesystems, installation...?",
    "explain": "Sure! What would you like me to explain?",
    "how": "How to do what? Tell me what you're trying to accomplish.",
    "why": "Why what? I'm happy to explain the reasoning behind any step.",
    "when": "When do you mean? Tell me what you're asking about.",
    "ok so": "Go ahead, I'm listening!",
    "alright": "What would you like to do next?",
    "so basically": "Yes? What's your question?",
}

# More question variations to expand the dataset
QUESTION_PATTERNS = [
    ("how do I {action}?", "How do I {action}?"),
    ("what is {topic}?", "What is {topic}?"),
    ("can I {action}?", "Can I {action}?"),
    ("should I {action}?", "Should I {action}?"),
    ("where is {thing}?", "Where is {thing}?"),
    ("which {thing} should I choose?", "Which {thing} should I choose?"),
    ("why {reason}?", "Why {reason}?"),
    ("is it safe to {action}?", "Is it safe to {action}?"),
    ("what happens if I {action}?", "What happens if I {action}?"),
    ("do I need to {action}?", "Do I need to {action}?"),
]

# SAFETY: Dangerous commands that should be REFUSED
DANGEROUS_QUERIES = [
    # Direct dangerous commands
    "rm -rf", "rm -rf /", "rm -rf /*", "sudo rm -rf",
    "delete everything", "delete all", "erase everything",
    "wipe everything", "destroy the disk", "format everything",
    "dd if=/dev/zero", "dd if=/dev/urandom",
    "mkfs on all disks", "format all drives",
    # Dangerous without context
    "just delete it", "remove it all", "clear everything",
    "nuke the system", "wipe it", "erase it",
    # Shell injection attempts
    "; rm -rf /", "| rm -rf", "&& rm -rf",
    "$(rm -rf /)", "`rm -rf /`",
]

SAFETY_REFUSAL_RESPONSES = [
    "I can't execute that command - it could damage your system. What would you like to do instead?",
    "That's a dangerous operation. Please be more specific about what you want to accomplish.",
    "I won't run destructive commands without explicit confirmation. What are you trying to do?",
    "For safety, I can't execute that. Can you tell me what you're trying to achieve?",
    "That command could cause data loss. Please specify exactly what you want to do.",
]

# AMBIGUOUS: Queries that need clarification
AMBIGUOUS_QUERIES = {
    "partition": [
        "Which disk would you like to partition? Try 'list disks' to see available options.",
        "I need to know which disk to partition. Say 'list disks' to see your options.",
    ],
    "format": [
        "Which partition should I format, and with what filesystem? (e.g., 'format /dev/sda2 as ext4')",
        "Please specify the partition and filesystem. Example: 'format root ext4'",
    ],
    "mount": [
        "What would you like to mount and where? Example: 'mount /dev/sda2 to /mnt'",
        "Please specify the partition and mount point.",
    ],
    "create user": [
        "What username would you like? Example: 'create user john with sudo'",
        "Please provide a username. Example: 'add user alice'",
    ],
    "add user": [
        "What username would you like to add? Example: 'add user bob with sudo'",
        "Please specify the username. Example: 'create user admin'",
    ],
    "make user": [
        "What username would you like? Example: 'make user john'",
        "Please provide a username. Example: 'make user admin with sudo'",
    ],
    "new user": [
        "What username would you like? Example: 'new user alice'",
        "Please specify the username. Example: 'new user bob with sudo'",
    ],
    "user": [
        "Do you want to create a user? Please specify: 'create user <name>'",
        "What would you like to do with users? Try 'create user <name> with sudo'",
    ],
    "install": [
        "Do you want to install the system or the bootloader? Try 'install system' or 'install bootloader'",
        "Please specify: 'install system' to copy files, or 'install bootloader' for boot setup.",
    ],
    "install levitate": [
        "Ready to install LevitateOS. This will copy files to /mnt. Proceed? Say 'install system' to start.",
        "I'll install the system. This copies files from the live environment. Say 'install system' to begin.",
    ],
    "encrypt": [
        "Which partition should be encrypted? The root partition? Example: 'encrypt root'",
        "Please specify what to encrypt. Example: 'encrypted root partition'",
    ],
    "set": [
        "What would you like to set? Try 'set timezone', 'set hostname', or 'set password'.",
        "Please be more specific. Examples: 'set timezone los angeles', 'set hostname mypc'",
    ],
    "configure": [
        "What would you like to configure? Try 'hostname', 'timezone', 'keyboard', or 'language'.",
        "Please specify what to configure. Example: 'configure timezone' or 'hostname mypc'",
    ],
    "setup": [
        "What would you like to set up? Boot, users, or something else?",
        "Please be more specific. Try 'setup boot' or 'create user <name>'",
    ],
    "delete": [
        "What would you like to delete? Please be specific.",
        "I need more details. What exactly do you want to remove?",
    ],
    "remove": [
        "What would you like to remove? Please specify.",
        "Please be more specific about what to remove.",
    ],
}

# Additional variations for cancel/stop
CANCEL_VARIATIONS = [
    "cancel", "stop", "abort", "nevermind", "never mind",
    "forget it", "nvm", "scratch that", "hold on", "wait",
    "stop that", "cancel that", "dont do that", "don't",
]

# More user creation patterns
USER_CREATION_PATTERNS = [
    ("create user {name}", "useradd -m -s /bin/bash {name}"),
    ("add user {name}", "useradd -m -s /bin/bash {name}"),
    ("new user {name}", "useradd -m -s /bin/bash {name}"),
    ("make user {name}", "useradd -m -s /bin/bash {name}"),
    ("user {name}", "useradd -m -s /bin/bash {name}"),
    ("create account {name}", "useradd -m -s /bin/bash {name}"),
    ("add account {name}", "useradd -m -s /bin/bash {name}"),
    ("create user {name} with sudo", "useradd -m -G wheel -s /bin/bash {name}"),
    ("add user {name} with sudo", "useradd -m -G wheel -s /bin/bash {name}"),
    ("create user {name} as admin", "useradd -m -G wheel -s /bin/bash {name}"),
    ("add {name} as admin", "useradd -m -G wheel -s /bin/bash {name}"),
    ("make {name} admin", "useradd -m -G wheel -s /bin/bash {name}"),
    ("give {name} sudo", "useradd -m -G wheel -s /bin/bash {name}"),
    ("{name} should be admin", "useradd -m -G wheel -s /bin/bash {name}"),
    ("{name} with sudo access", "useradd -m -G wheel -s /bin/bash {name}"),
    ("i want user {name}", "useradd -m -s /bin/bash {name}"),
    ("my username is {name}", "useradd -m -s /bin/bash {name}"),
    ("username {name}", "useradd -m -s /bin/bash {name}"),
    ("call me {name}", "useradd -m -s /bin/bash {name}"),
]

EXTENDED_USERNAMES = [
    "user", "admin", "vince", "john", "alice", "bob", "root",
    "me", "dev", "main", "default", "test", "guest", "operator",
    "sysadmin", "developer", "owner", "primary",
]


def add_typos(text: str, prob: float = 0.1) -> str:
    """Randomly introduce typos"""
    if random.random() > prob:
        return text

    chars = list(text)
    idx = random.randint(0, len(chars) - 1)

    typo_type = random.choice(['swap', 'delete', 'double', 'replace'])

    if typo_type == 'swap' and idx < len(chars) - 1:
        chars[idx], chars[idx+1] = chars[idx+1], chars[idx]
    elif typo_type == 'delete' and len(chars) > 3:
        del chars[idx]
    elif typo_type == 'double':
        chars.insert(idx, chars[idx])
    elif typo_type == 'replace':
        nearby = {
            'a': 'sq', 'b': 'vn', 'c': 'xv', 'd': 'sf', 'e': 'wr',
            'f': 'dg', 'g': 'fh', 'h': 'gj', 'i': 'uo', 'j': 'hk',
            'k': 'jl', 'l': 'k;', 'm': 'n,', 'n': 'bm', 'o': 'ip',
            'p': 'o[', 'q': 'wa', 'r': 'et', 's': 'ad', 't': 'ry',
            'u': 'yi', 'v': 'cb', 'w': 'qe', 'x': 'zc', 'y': 'tu',
            'z': 'xs',
        }
        if chars[idx].lower() in nearby:
            chars[idx] = random.choice(nearby[chars[idx].lower()])

    return ''.join(chars)

def lowercase_variation(text: str) -> str:
    """Random case variations"""
    r = random.random()
    if r < 0.7:
        return text.lower()
    elif r < 0.9:
        return text
    else:
        return text.upper()

def format_disk_list_response(disks: dict) -> str:
    """Format a response that lists disks from the context."""
    lines = ["Found the following disks:"]
    for dev in disks.get("blockdevices", []):
        if dev.get("type") == "disk":
            name = f"/dev/{dev['name']}"
            size = dev.get("size", "?")
            model = (dev.get("model") or "").strip() or "Unknown"
            lines.append(f"  - {name}: {size} ({model})")
    return "\n".join(lines)


def generate_disk_queries():
    """Generate disk-related query/response pairs WITH SYSTEM CONTEXT."""
    entries = []

    for query in DISK_QUERIES["list"]:
        # Generate fresh context for each example
        disks = random.choice(SYNTHETIC_DISKS)
        context = generate_system_context()
        disk_response = format_disk_list_response(disks)

        entries.append({
            "query": lowercase_variation(query),
            "response": disk_response,
            "command": "lsblk",
            "system_context": context
        })
        # Add typo variant with same context
        entries.append({
            "query": add_typos(lowercase_variation(query), 0.3),
            "response": disk_response,
            "command": "lsblk",
            "system_context": context
        })

    for query in DISK_QUERIES["details"]:
        context = generate_system_context()
        entries.append({
            "query": lowercase_variation(query),
            "response": "Detailed disk information:",
            "command": "lsblk -f",
            "system_context": context
        })

    for query in DISK_QUERIES["space"]:
        context = generate_system_context()
        entries.append({
            "query": lowercase_variation(query),
            "response": "Disk space usage:",
            "command": "df -h",
            "system_context": context
        })

    return entries

def generate_timezone_queries():
    """Generate timezone query/response pairs"""
    entries = []

    for tz_name, tz_path in TIMEZONES.items():
        for template in TIMEZONE_TEMPLATES:
            query = template.format(tz=tz_name)
            entries.append({
                "query": lowercase_variation(query),
                "response": f"Setting timezone to {tz_path}.",
                "command": f"ln -sf /usr/share/zoneinfo/{tz_path} /etc/localtime && hwclock --systohc"
            })

    return entries

def generate_hostname_queries():
    """Generate hostname query/response pairs"""
    entries = []

    for hostname in SAMPLE_HOSTNAMES:
        for template in HOSTNAME_TEMPLATES:
            query = template.format(name=hostname)
            entries.append({
                "query": lowercase_variation(query),
                "response": f"Setting hostname to '{hostname}'.",
                "command": f"echo '{hostname}' > /etc/hostname"
            })

    return entries

def generate_user_queries():
    """Generate user creation query/response pairs"""
    entries = []

    for username in SAMPLE_USERNAMES:
        for template in USER_TEMPLATES:
            query = template.format(name=username)
            has_sudo = "sudo" in query.lower() or "admin" in query.lower()

            if has_sudo:
                entries.append({
                    "query": lowercase_variation(query),
                    "response": f"Creating user '{username}' with sudo privileges.",
                    "command": f"useradd -m -G wheel -s /bin/bash {username} && echo '%wheel ALL=(ALL:ALL) ALL' > /etc/sudoers.d/wheel"
                })
            else:
                entries.append({
                    "query": lowercase_variation(query),
                    "response": f"Creating user '{username}'. Should this user have sudo access?",
                    "context": "needs_clarification"
                })

    return entries

def generate_simple_queries():
    """Generate simple command queries"""
    entries = []

    # Boot queries
    for query in BOOT_QUERIES:
        entries.append({
            "query": lowercase_variation(query),
            "response": "Installing bootloader.",
            "command": "bootctl install"
        })

    # Install queries
    for query in INSTALL_QUERIES:
        entries.append({
            "query": lowercase_variation(query),
            "response": "Installing system. This may take a few minutes.",
            "command": "rsync -aAXv --exclude='/dev/*' --exclude='/proc/*' --exclude='/sys/*' --exclude='/tmp/*' --exclude='/run/*' --exclude='/mnt/*' / /mnt/"
        })

    # Finish queries
    for query in FINISH_QUERIES:
        entries.append({
            "query": lowercase_variation(query),
            "response": "Unmounting partitions. You can now reboot.",
            "command": "umount -R /mnt"
        })

    # Reboot queries
    for query in REBOOT_QUERIES:
        entries.append({
            "query": lowercase_variation(query),
            "response": "Rebooting now.",
            "command": "reboot"
        })

    # Help queries
    for query in HELP_QUERIES:
        entries.append({
            "query": lowercase_variation(query),
            "response": "I can help you with:\n- Disk configuration (partition, format, encrypt)\n- System installation\n- Configuration (hostname, timezone, locale)\n- User setup\n\nJust describe what you want!"
        })

    # Greetings
    for query in GREETINGS:
        entries.append({
            "query": lowercase_variation(query),
            "response": random.choice([
                "Hello! Ready to install LevitateOS. Try 'list disks' to start.",
                "Hi there! What would you like to do?",
                "Hey! Let's get LevitateOS installed. Say 'help' for guidance.",
            ])
        })

    # Farewells
    for query in FAREWELLS:
        entries.append({
            "query": lowercase_variation(query),
            "response": "Goodbye! Remove the USB and reboot when ready."
        })

    # Thanks
    for query in THANKS:
        entries.append({
            "query": lowercase_variation(query),
            "response": random.choice([
                "You're welcome!",
                "Happy to help!",
                "No problem! What's next?",
            ])
        })

    # Confirmations
    for query in CONFIRM_POSITIVE:
        entries.append({
            "query": lowercase_variation(query),
            "response": "Proceeding."
        })

    for query in CONFIRM_NEGATIVE:
        entries.append({
            "query": lowercase_variation(query),
            "response": "Cancelled. What would you like to do instead?"
        })

    return entries

def generate_encrypt_queries():
    """Generate encryption queries"""
    entries = []

    for query in ENCRYPT_QUERIES:
        entries.append({
            "query": lowercase_variation(query),
            "response": "Setting up LUKS encryption. Only /boot/efi will remain unencrypted.",
            "plan": {"encrypted": True}
        })
        entries.append({
            "query": add_typos(lowercase_variation(query), 0.3),
            "response": "Enabling disk encryption.",
            "plan": {"encrypted": True}
        })

    return entries

def get_disk_from_context(disks: dict) -> tuple[str, str]:
    """Get first disk name and size from disk config."""
    for dev in disks.get("blockdevices", []):
        if dev.get("type") == "disk":
            return f"/dev/{dev['name']}", dev.get("size", "")
    return "/dev/sda", "500G"


def generate_format_queries():
    """Generate filesystem format queries WITH SYSTEM CONTEXT."""
    entries = []

    for fs_type, templates in FORMAT_QUERIES.items():
        for template in templates:
            # Generate context and get disk from it
            disks = random.choice(SYNTHETIC_DISKS)
            context = generate_system_context()
            disk, _ = get_disk_from_context(disks)

            # Partition name based on disk type
            if "nvme" in disk:
                part1, part2 = f"{disk}p1", f"{disk}p2"
            else:
                part1, part2 = f"{disk}1", f"{disk}2"

            # Create variations with actual disk partition and generic refs
            for part_ref in [part2, "root", "the root partition"]:
                query = template.format(part=part_ref)

                if fs_type == "ext4":
                    cmd = f"mkfs.ext4 {part2}"
                    resp = f"Formatting {part2} with ext4."
                elif fs_type == "btrfs":
                    cmd = f"mkfs.btrfs {part2}"
                    resp = f"Formatting {part2} with Btrfs."
                elif fs_type == "xfs":
                    cmd = f"mkfs.xfs {part2}"
                    resp = f"Formatting {part2} with XFS."
                elif fs_type == "fat32":
                    cmd = f"mkfs.fat -F32 {part1}"
                    resp = f"Formatting {part1} with FAT32."
                else:
                    continue

                entries.append({
                    "query": lowercase_variation(query),
                    "response": resp,
                    "command": cmd,
                    "system_context": context
                })

    return entries

def get_disk_descriptions(disk: str, size: str) -> list[str]:
    """Generate natural descriptions for a disk."""
    descs = [disk]
    if "nvme" in disk:
        descs.extend(["the nvme", "nvme drive", "the nvme ssd"])
    elif size:
        size_lower = size.lower()
        if "500" in size_lower or "512" in size_lower:
            descs.extend(["the 500gb drive", "500gb ssd", "the ssd"])
        elif "1t" in size_lower or "1000" in size_lower:
            descs.extend(["the 1tb drive", "1tb hdd", "the hdd"])
        elif "2t" in size_lower:
            descs.extend(["the 2tb drive", "the big drive"])
        elif "32" in size_lower:
            descs.extend(["the small drive", "the 32gb drive"])
    return descs


def generate_partition_queries():
    """Generate partition queries WITH SYSTEM CONTEXT."""
    entries = []

    for _ in range(50):  # Generate many context-aware examples
        disks_config = random.choice(SYNTHETIC_DISKS)
        context = generate_system_context()
        disk, size = get_disk_from_context(disks_config)
        descriptions = get_disk_descriptions(disk, size)

        for template in PARTITION_QUERIES["start"]:
            for desc in descriptions[:3]:  # Limit to avoid explosion
                query = template.format(disk=disk, disk_desc=desc)
                entries.append({
                    "query": lowercase_variation(query),
                    "response": f"I'll partition {disk}. Use whole disk or custom layout?",
                    "context": "needs_clarification",
                    "system_context": context
                })

        for template in PARTITION_QUERIES["whole_disk"]:
            for desc in descriptions[:2]:
                query = template.format(disk=disk, disk_desc=desc)
                entries.append({
                    "query": lowercase_variation(query),
                    "response": f"Using whole disk ({disk}):\n- 512MB EFI\n- Rest for root\n\nWARNING: This erases all data. Proceed?",
                    "plan": {
                        "disk": disk,
                        "scheme": [
                            {"mount": "/boot/efi", "size": "512M", "fs": "vfat"},
                            {"mount": "/", "size": "rest", "fs": "ext4"}
                        ]
                    },
                    "system_context": context
                })

        # Swap queries with context
        sizes = ["4G", "8G", "16G"]
        for template in PARTITION_QUERIES["swap"]:
            for swap_size in random.sample(sizes, 2):
                query = template.format(size=swap_size)
                entries.append({
                    "query": lowercase_variation(query),
                    "response": f"Creating {swap_size} swap partition on {disk}.",
                    "command": f"sgdisk -n 2:0:+{swap_size} -t 2:8200 {disk}",
                    "system_context": context
                })

    return entries


def generate_safety_queries():
    """Generate safety examples - dangerous queries should be REFUSED."""
    entries = []

    for query in DANGEROUS_QUERIES:
        # Multiple refusal responses per dangerous query
        for _ in range(3):  # Generate multiple examples per query
            entries.append({
                "query": lowercase_variation(query),
                "response": random.choice(SAFETY_REFUSAL_RESPONSES)
            })
        # Also add with typos
        entries.append({
            "query": add_typos(lowercase_variation(query), 0.3),
            "response": random.choice(SAFETY_REFUSAL_RESPONSES)
        })

    return entries


def generate_ambiguous_queries():
    """Generate ambiguous queries that need clarification.

    Critical queries like 'create user' and 'install' need MANY more examples
    to outweigh the specific command examples.
    """
    entries = []

    # Critical ambiguous queries that need heavy weighting
    critical_queries = {
        "create user", "add user", "make user", "new user", "user",
        "install", "install levitate", "partition"
    }

    for query, responses in AMBIGUOUS_QUERIES.items():
        # Critical queries get 50x weight to balance against specific examples
        multiplier = 50 if query in critical_queries else 5

        for _ in range(multiplier):
            entries.append({
                "query": lowercase_variation(query),
                "response": random.choice(responses)
            })
            # With typos
            if random.random() < 0.3:
                entries.append({
                    "query": add_typos(lowercase_variation(query), 0.4),
                    "response": random.choice(responses)
                })

        # Variations
        variations = [
            f"{query}?",
            f"just {query}",
            f"i want to {query}",
            f"can you {query}",
            f"please {query}",
            f"{query} please",
            f"help me {query}",
            f"let's {query}",
        ]

        for var in variations:
            count = 10 if query in critical_queries else 2
            for _ in range(count):
                entries.append({
                    "query": lowercase_variation(var),
                    "response": random.choice(responses)
                })

    return entries


def generate_cancel_queries():
    """Generate cancel/stop queries - should produce text, not commands."""
    entries = []

    for query in CANCEL_VARIATIONS:
        entries.append({
            "query": lowercase_variation(query),
            "response": "Cancelled. What would you like to do instead?"
        })
        entries.append({
            "query": lowercase_variation(query),
            "response": "Operation cancelled. How can I help?"
        })

    return entries


def generate_extended_user_queries():
    """Generate comprehensive user creation examples."""
    entries = []

    for pattern, cmd_template in USER_CREATION_PATTERNS:
        for name in EXTENDED_USERNAMES:
            query = pattern.format(name=name)
            cmd = cmd_template.format(name=name)

            # Determine if sudo/admin
            has_sudo = "wheel" in cmd

            if has_sudo:
                resp = f"Creating user '{name}' with administrator privileges."
            else:
                resp = f"Creating user '{name}'."

            entries.append({
                "query": lowercase_variation(query),
                "response": resp,
                "command": cmd
            })

            # Add typo variant
            if random.random() < 0.3:
                entries.append({
                    "query": add_typos(lowercase_variation(query), 0.5),
                    "response": resp,
                    "command": cmd
                })

    return entries


def generate_question_queries():
    """Generate question-based queries that should get helpful text responses.

    These are the bulk of real user interactions - people asking questions
    about what to do, how things work, etc. These should NOT produce commands.
    """
    entries = []

    # Generate many variations of each question
    # User wants ~75% questions - need heavy weight to balance commands
    multiplier = 150  # Very heavy weight on questions

    for query, response in QUESTION_QUERIES.items():
        for _ in range(multiplier):
            entries.append({
                "query": lowercase_variation(query),
                "response": response
            })
            # Add typo versions
            if random.random() < 0.3:
                entries.append({
                    "query": add_typos(lowercase_variation(query), 0.4),
                    "response": response
                })

        # Without question mark
        if query.endswith("?"):
            no_qmark = query[:-1]
            for _ in range(multiplier // 2):
                entries.append({
                    "query": lowercase_variation(no_qmark),
                    "response": response
                })

        # With "please"
        entries.append({
            "query": f"{query} please",
            "response": response
        })
        entries.append({
            "query": f"please {query}",
            "response": response
        })

    return entries


def main():
    all_entries = []

    print("Generating disk queries...")
    all_entries.extend(generate_disk_queries())

    print("Generating timezone queries...")
    all_entries.extend(generate_timezone_queries())

    print("Generating hostname queries...")
    all_entries.extend(generate_hostname_queries())

    print("Generating user queries...")
    all_entries.extend(generate_user_queries())

    print("Generating simple queries...")
    all_entries.extend(generate_simple_queries())

    print("Generating encrypt queries...")
    all_entries.extend(generate_encrypt_queries())

    print("Generating format queries...")
    all_entries.extend(generate_format_queries())

    print("Generating partition queries...")
    all_entries.extend(generate_partition_queries())

    print("Generating safety queries...")
    all_entries.extend(generate_safety_queries())

    print("Generating ambiguous queries...")
    all_entries.extend(generate_ambiguous_queries())

    print("Generating cancel queries...")
    all_entries.extend(generate_cancel_queries())

    print("Generating extended user queries...")
    all_entries.extend(generate_extended_user_queries())

    print("Generating question queries...")
    all_entries.extend(generate_question_queries())

    # Ensure ALL entries have system_context
    print("\nAdding system context to entries that don't have it...")
    for entry in all_entries:
        if "system_context" not in entry:
            entry["system_context"] = generate_system_context()

    # Remove duplicates based on query
    seen = set()
    unique_entries = []
    for entry in all_entries:
        q = entry["query"].lower().strip()
        if q not in seen:
            seen.add(q)
            unique_entries.append(entry)

    # Shuffle
    random.shuffle(unique_entries)

    # Write output
    output_path = Path(__file__).parent / "training" / "augmented_dataset.jsonl"
    output_path.parent.mkdir(exist_ok=True)

    with open(output_path, 'w') as f:
        for entry in unique_entries:
            f.write(json.dumps(entry) + '\n')

    # Stats
    with_context = sum(1 for e in unique_entries if "system_context" in e)
    print(f"\nGenerated {len(unique_entries)} unique training examples")
    print(f"  - With system context: {with_context} ({100*with_context//len(unique_entries)}%)")
    print(f"Saved to: {output_path}")

if __name__ == "__main__":
    main()
