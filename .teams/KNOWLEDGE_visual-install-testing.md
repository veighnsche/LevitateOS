# Visual Installation Testing via QEMU + noVNC + Puppeteer

## CRITICAL: Resolution Requirements

**ALL screenshots MUST be 1920x1080. NO EXCEPTIONS.**

LevitateOS is a daily-driver desktop OS targeting modern hardware. Users have 1080p+ displays.
Never use 1024x768 or other legacy resolutions - this is 2026, not 1999.

```
mcp__puppeteer__puppeteer_screenshot  name="example" width=1920 height=1080
```

## Purpose

This document provides step-by-step instructions for testing LevitateOS installation in a headless QEMU VM using Puppeteer MCP tools and noVNC. This simulates a real user installing LevitateOS to disk.

**Target Audience**: Small language models and AI agents that need to perform visual installation testing.

---

## Prerequisites

### Required Software

```bash
# Check these are installed
which qemu-system-x86_64  # QEMU
which websockify          # WebSocket proxy (for noVNC)
recqemu ovmf              # Verify OVMF detection
```

### Required Files

| File | Location | How to Build |
|------|----------|--------------|
| LevitateOS ISO | `leviso/output/levitateos-x86_64.iso` | `cd leviso && cargo run -- build iso` |
| OVMF UEFI firmware | Auto-detected | System package (edk2-ovmf) |

---

## Step 1: Start QEMU with VNC + Websockify

Use `recqemu vnc` with `--websockify` to start everything in one command:

```bash
recqemu vnc leviso/output/levitateos-x86_64.iso --websockify &
```

This automatically:
- Creates a 20GB qcow2 disk (next to the ISO)
- Finds OVMF firmware
- Starts QEMU with VNC on :5900
- Starts websockify on port 6080
- Serves noVNC web interface

**Options**:
```bash
recqemu vnc --help

# Custom disk size
recqemu vnc levitateos-x86_64.iso --disk-size 50G --websockify &

# Use existing disk
recqemu vnc levitateos-x86_64.iso --disk /path/to/disk.qcow2 --websockify &

# Custom ports
recqemu vnc levitateos-x86_64.iso --display 1 --websockify-port 8080 --websockify &
```

**Alternative: Manual startup** (if you need more control):
```bash
# Start QEMU with VNC only
recqemu vnc levitateos-x86_64.iso &

# Start websockify separately
websockify 6080 localhost:5900 --web /usr/share/novnc &
```

noVNC is served at `http://localhost:6080`.

---

## Step 3: Connect via Puppeteer

Use the Puppeteer MCP tools to connect to noVNC:

```
mcp__puppeteer__puppeteer_navigate
  url: "http://localhost:6080/vnc.html?autoconnect=true"
```

**Wait 3-5 seconds** for the VNC connection to establish before taking screenshots or typing.

---

## Step 4: Take Screenshots

**CRITICAL: ALWAYS USE 1920x1080 FORMAT**

All screenshots MUST be 1920x1080 to capture the full TUI and avoid missing critical UI elements:

```
mcp__puppeteer__puppeteer_screenshot
  name: "current-state"
  width: 1920
  height: 1080
```

**Naming convention**: Use descriptive names like `boot-complete`, `after-fdisk`, `recstrap-done`.

**Why 1920x1080**: The LevitateOS live ISO should display a TUI on boot. Smaller formats (1024x768) may crop or hide UI elements, masking failures.

---

## Step 5: Type Commands

**CRITICAL**: All typing goes through the noVNC keyboard input element.

```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "your command here\n"
```

**IMPORTANT RULES**:

1. **Always end with `\n`** to press Enter
2. **One command at a time** - Don't use `&&` chains (VNC input breaks them)
3. **Wait between commands** - Take a screenshot to verify completion before next command
4. **Special characters may fail** - `>>`, `|`, `&&` often don't work reliably

**Example - Run `lsblk`**:
```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "lsblk\n"
```

---

## Step 6: Installation Process

### 6.1 Verify Boot Complete

After connecting, take a screenshot to verify the system booted to a shell prompt:

```
mcp__puppeteer__puppeteer_screenshot  name: "boot-check"
```

You should see a root shell prompt (`#` or `root@levitate`).

### 6.2 Check Available Disks

```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "lsblk\n"
```

Expected output shows:
- `sr0` - CD-ROM (the ISO)
- `vda` - Virtual disk (20GB, for installation)

### 6.3 Partition the Disk

Use `fdisk` (NOT `gdisk` - it's not included):

```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "fdisk /dev/vda\n"
```

Then send these commands one at a time, taking screenshots between each:

| Command | Purpose |
|---------|---------|
| `g\n` | Create GPT partition table |
| `n\n` | New partition |
| `1\n` | Partition number 1 |
| `\n` | Default first sector |
| `+512M\n` | 512MB size (for EFI) |
| `t\n` | Change partition type |
| `1\n` | Type 1 = EFI System |
| `n\n` | New partition |
| `2\n` | Partition number 2 |
| `\n` | Default first sector |
| `\n` | Default last sector (rest of disk) |
| `w\n` | Write and exit |

### 6.4 Format Partitions

**EFI partition (FAT32)**:
```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "mkfs.vfat /dev/vda1\n"
```

**Root partition (ext4)**:
```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "mkfs.ext4 /dev/vda2\n"
```

### 6.5 Mount Partitions

```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "mount /dev/vda2 /mnt\n"
```

```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "mkdir -p /mnt/boot/efi\n"
```

```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "mount /dev/vda1 /mnt/boot/efi\n"
```

### 6.6 Extract Root Filesystem

```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "recstrap --force /mnt\n"
```

**This takes 1-3 minutes**. Take screenshots periodically to monitor progress.

### 6.7 Generate fstab

```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "recfstab /mnt\n"
```

Then manually append to fstab (since `>>` is unreliable):

```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "cat /mnt/etc/fstab\n"
```

Take screenshot to see current fstab, then use `tee -a` if needed.

### 6.8 Enter Chroot

```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "recchroot /mnt\n"
```

### 6.9 Install Bootloader

```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "bootctl install\n"
```

### 6.10 Exit and Reboot

```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "exit\n"
```

```
mcp__puppeteer__puppeteer_fill
  selector: "#noVNC_keyboardinput"
  value: "reboot\n"
```

---

## Troubleshooting

### Problem: VNC shows black screen

**Cause**: QEMU hasn't booted yet or display not initialized.

**Fix**: Wait 10-15 seconds and take another screenshot.

### Problem: Commands don't execute

**Cause**: The `\n` was missing or VNC input focus was lost.

**Fix**:
1. Click on the noVNC canvas first
2. Ensure command ends with `\n`
3. Try clicking the input field directly

### Problem: `&&` chains fail

**Cause**: noVNC keyboard input doesn't handle `&` reliably.

**Fix**: Run commands separately, one at a time.

### Problem: `>>` redirection fails

**Cause**: Same VNC input issue.

**Fix**: Use `tee -a` instead:
```bash
echo "content" | tee -a /path/to/file
```

### Problem: gdisk not found

**Cause**: gdisk is not included in LevitateOS. Use fdisk instead.

**Fix**: fdisk supports GPT with the `g` command.

### Problem: recstrap fails with "not a mount point"

**Cause**: Target directory isn't mounted.

**Fix**: Use `--force` flag or ensure `/mnt` is a mount point.

---

## Complete Test Script (Sequential)

Here's the full sequence of Puppeteer calls for a complete installation test:

```
1.  puppeteer_navigate      url="http://localhost:6080/vnc.html?autoconnect=true"
2.  [wait 5 seconds]
3.  puppeteer_screenshot    name="01-boot-check"
4.  puppeteer_fill          selector="#noVNC_keyboardinput" value="lsblk\n"
5.  puppeteer_screenshot    name="02-lsblk"
6.  puppeteer_fill          selector="#noVNC_keyboardinput" value="fdisk /dev/vda\n"
7.  puppeteer_fill          selector="#noVNC_keyboardinput" value="g\n"
8.  puppeteer_fill          selector="#noVNC_keyboardinput" value="n\n"
9.  puppeteer_fill          selector="#noVNC_keyboardinput" value="1\n"
10. puppeteer_fill          selector="#noVNC_keyboardinput" value="\n"
11. puppeteer_fill          selector="#noVNC_keyboardinput" value="+512M\n"
12. puppeteer_fill          selector="#noVNC_keyboardinput" value="t\n"
13. puppeteer_fill          selector="#noVNC_keyboardinput" value="1\n"
14. puppeteer_fill          selector="#noVNC_keyboardinput" value="n\n"
15. puppeteer_fill          selector="#noVNC_keyboardinput" value="2\n"
16. puppeteer_fill          selector="#noVNC_keyboardinput" value="\n"
17. puppeteer_fill          selector="#noVNC_keyboardinput" value="\n"
18. puppeteer_fill          selector="#noVNC_keyboardinput" value="w\n"
19. puppeteer_screenshot    name="03-partitioned"
20. puppeteer_fill          selector="#noVNC_keyboardinput" value="mkfs.vfat /dev/vda1\n"
21. puppeteer_screenshot    name="04-efi-formatted"
22. puppeteer_fill          selector="#noVNC_keyboardinput" value="mkfs.ext4 /dev/vda2\n"
23. puppeteer_screenshot    name="05-root-formatted"
24. puppeteer_fill          selector="#noVNC_keyboardinput" value="mount /dev/vda2 /mnt\n"
25. puppeteer_fill          selector="#noVNC_keyboardinput" value="mkdir -p /mnt/boot/efi\n"
26. puppeteer_fill          selector="#noVNC_keyboardinput" value="mount /dev/vda1 /mnt/boot/efi\n"
27. puppeteer_screenshot    name="06-mounted"
28. puppeteer_fill          selector="#noVNC_keyboardinput" value="recstrap --force /mnt\n"
29. [wait 60-180 seconds]
30. puppeteer_screenshot    name="07-recstrap-done"
31. puppeteer_fill          selector="#noVNC_keyboardinput" value="recfstab /mnt\n"
32. puppeteer_screenshot    name="08-fstab-generated"
33. puppeteer_fill          selector="#noVNC_keyboardinput" value="recchroot /mnt\n"
34. puppeteer_fill          selector="#noVNC_keyboardinput" value="bootctl install\n"
35. puppeteer_screenshot    name="09-bootloader-installed"
36. puppeteer_fill          selector="#noVNC_keyboardinput" value="exit\n"
37. puppeteer_fill          selector="#noVNC_keyboardinput" value="reboot\n"
38. [wait 30 seconds]
39. puppeteer_screenshot    name="10-reboot-to-installed"
```

---

## Cleanup

After testing, kill the background processes:

```bash
pkill -f "qemu-system-x86_64.*levitate"
pkill -f "websockify"
```

Remove the test disk if no longer needed:

```bash
rm /tmp/levitate-test.qcow2
```

---

## Available Tools in Live Environment

These tools are available for installation (from `distro-spec::LEVITATE_TOOLS`):

| Tool | Purpose |
|------|---------|
| `recstrap` | Extract rootfs to target (like pacstrap) |
| `recfstab` | Generate fstab entries (like genfstab) |
| `recchroot` | Enter chroot with bind mounts (like arch-chroot) |
| `recipe` | Package manager |
| `levitate-docs` | Interactive documentation TUI |

## Available Disk Tools

| Tool | Purpose |
|------|---------|
| `fdisk` | Partition disks (supports GPT with `g` command) |
| `parted` | Alternative partitioner |
| `mkfs.ext4` | Format ext4 |
| `mkfs.vfat` | Format FAT32 (for EFI) |
| `mkfs.btrfs` | Format Btrfs |
| `mkfs.xfs` | Format XFS |
| `lsblk` | List block devices |
| `blkid` | Show UUIDs |
| `mount` / `umount` | Mount/unmount filesystems |

---

## Success Criteria

A successful installation test should:

1. Boot the live ISO to a shell prompt
2. Create GPT partition table with EFI + root partitions
3. Format partitions (FAT32 for EFI, ext4/btrfs/xfs for root)
4. Mount partitions correctly
5. Run `recstrap` without errors
6. Generate valid fstab with `recfstab`
7. Enter chroot with `recchroot`
8. Install bootloader with `bootctl install`
9. Reboot into the installed system

If any step fails, take a screenshot and analyze the error output.
