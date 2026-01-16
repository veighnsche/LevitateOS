#!/usr/bin/env python3
"""
Generate augmented training data for the installer LLM.
Creates variations of queries through paraphrasing, typos, and different phrasings.
"""

import json
import random
import re
from pathlib import Path

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

def generate_disk_queries():
    """Generate disk-related query/response pairs"""
    entries = []

    for query in DISK_QUERIES["list"]:
        entries.append({
            "query": lowercase_variation(query),
            "response": random.choice([
                "Here are your available disks:",
                "Let me show you the available disks.",
                "These are the connected storage devices:",
                "Available disks:",
            ]),
            "command": "lsblk"
        })
        # Add typo variant
        entries.append({
            "query": add_typos(lowercase_variation(query), 0.3),
            "response": "Showing disks:",
            "command": "lsblk"
        })

    for query in DISK_QUERIES["details"]:
        entries.append({
            "query": lowercase_variation(query),
            "response": "Detailed disk information:",
            "command": "lsblk -f"
        })

    for query in DISK_QUERIES["space"]:
        entries.append({
            "query": lowercase_variation(query),
            "response": "Disk space usage:",
            "command": "df -h"
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

def generate_format_queries():
    """Generate filesystem format queries"""
    entries = []

    partitions = ["/dev/sda2", "/dev/sdb2", "/dev/nvme0n1p2", "root", "the root partition"]

    for fs_type, templates in FORMAT_QUERIES.items():
        for template in templates:
            for part in partitions:
                query = template.format(part=part)

                if fs_type == "ext4":
                    cmd = f"mkfs.ext4 /dev/sda2"
                    resp = "Formatting with ext4."
                elif fs_type == "btrfs":
                    cmd = f"mkfs.btrfs /dev/sda2"
                    resp = "Formatting with Btrfs. You'll get snapshots and compression."
                elif fs_type == "xfs":
                    cmd = f"mkfs.xfs /dev/sda2"
                    resp = "Formatting with XFS."
                elif fs_type == "fat32":
                    cmd = f"mkfs.fat -F32 /dev/sda1"
                    resp = "Formatting with FAT32."

                entries.append({
                    "query": lowercase_variation(query),
                    "response": resp,
                    "command": cmd
                })

    return entries

def generate_partition_queries():
    """Generate partition queries"""
    entries = []

    disks = [
        ("/dev/sda", "the ssd", "500gb ssd", "the 500gb drive"),
        ("/dev/sdb", "the hdd", "1tb hdd", "the 1tb drive"),
        ("/dev/nvme0n1", "the nvme", "nvme drive", "the nvme ssd"),
    ]

    for disk, *descriptions in disks:
        for template in PARTITION_QUERIES["start"]:
            for desc in [disk] + descriptions:
                query = template.format(disk=disk, disk_desc=desc)
                entries.append({
                    "query": lowercase_variation(query),
                    "response": f"I'll partition {disk}. Use whole disk or custom layout?",
                    "context": "needs_clarification"
                })

        for template in PARTITION_QUERIES["whole_disk"]:
            for desc in [disk] + descriptions:
                query = template.format(disk=disk, disk_desc=desc)
                entries.append({
                    "query": lowercase_variation(query),
                    "response": "Using whole disk:\n- 512MB EFI\n- Rest for root\n\nWARNING: This erases all data. Proceed?",
                    "plan": {
                        "disk": disk,
                        "scheme": [
                            {"mount": "/boot/efi", "size": "512M", "fs": "vfat"},
                            {"mount": "/", "size": "rest", "fs": "ext4"}
                        ]
                    }
                })

    # Swap queries
    sizes = ["4G", "8G", "16G", "4gb", "8gb", "16gb"]
    for template in PARTITION_QUERIES["swap"]:
        for size in sizes:
            query = template.format(size=size)
            entries.append({
                "query": lowercase_variation(query),
                "response": f"Creating {size.upper()} swap partition.",
                "command": f"sgdisk -n 2:0:+{size.upper().replace('GB', 'G')} -t 2:8200 /dev/sda"
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

    print(f"\nGenerated {len(unique_entries)} unique training examples")
    print(f"Saved to: {output_path}")

if __name__ == "__main__":
    main()
