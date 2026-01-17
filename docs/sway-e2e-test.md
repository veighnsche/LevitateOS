# Sway Desktop E2E Test Instructions

> **NOTE: This is for TESTING the `levitate` package manager only.**
> Sway/Wayland is NOT part of default LevitateOS. LevitateOS boots to terminal.
> This test uses Sway because it has a complex dependency chain - good for testing.

Test the complete Sway desktop installation using the levitate package manager.

## Prerequisites

- QEMU with KVM support (`qemu-system-x86_64`)
- ~5GB free disk space
- Internet connection

## Quick Start (5 commands)

```bash
# 1. Download Arch cloud image (~500MB)
cargo xtask vm setup

# 2. Build levitate binary
cargo xtask vm prepare

# 3. Boot the VM
cargo xtask vm start --gui

# 4. In VM: login as arch/arch, then from host:
cargo xtask vm copy

# 5. In VM: install desktop and run
levitate desktop
sway
```

## Detailed Steps

### Step 1: Setup

```bash
cargo xtask vm setup
```

Downloads the official Arch Linux cloud image (~500MB) and resizes it to 20GB.

### Step 2: Prepare

```bash
cargo xtask vm prepare
```

Builds the `levitate` binary and stages it with all 22 recipes.

### Step 3: Boot VM

```bash
cargo xtask vm start --gui
```

Starts QEMU with:
- virtio-gpu (OpenGL for Wayland)
- virtio-keyboard/mouse
- SSH forwarding on port 2222

### Step 4: Login

Wait for the VM to boot. Login at the console:

```
Username: arch
Password: arch
```

### Step 5: Copy levitate to VM

From the **host** (another terminal):

```bash
cargo xtask vm copy
```

This copies:
- `/usr/local/bin/levitate` (package manager)
- `/usr/share/levitate/recipes/*.recipe` (22 recipes)

### Step 6: Install build dependencies

In the **VM**:

```bash
sudo pacman -Syu --noconfirm
sudo pacman -S --noconfirm --needed \
    base-devel meson ninja cmake pkg-config scdoc \
    wayland wayland-protocols libxkbcommon libinput \
    mesa libdrm pixman cairo pango gdk-pixbuf2 \
    json-c pcre2 libevdev mtdev seatd
```

### Step 7: Install Sway desktop

```bash
levitate desktop
```

This installs (in order):
- wayland, wayland-protocols, libxkbcommon, libinput
- seatd, wlroots
- sway, swaybg, swaylock, swayidle
- foot, waybar, wofi, mako
- grim, slurp, wl-clipboard

### Step 8: Start seatd and Sway

```bash
sudo systemctl enable --now seatd
sudo usermod -aG seat arch
# Logout and login again for group to take effect
exit
```

Login again, then:

```bash
sway
```

## Expected Result

Sway compositor starts with:
- Gray background
- Status bar (waybar) at top
- Working keyboard/mouse

### Test the desktop

| Action | Keys |
|--------|------|
| Open terminal | `Super + Enter` |
| Open launcher | `Super + d` |
| Exit Sway | `Super + Shift + e` |

### Take a screenshot

```bash
grim ~/screenshot.png
```

## Troubleshooting

### VM won't start

Check KVM is available:
```bash
ls /dev/kvm
```

### Can't SSH/copy to VM

Make sure VM is booted and SSH is running:
```bash
# In VM
sudo systemctl start sshd
```

### sway fails: "Failed to connect to socket"

seatd not running:
```bash
sudo systemctl start seatd
```

### Black screen in sway

Check virtio-gpu:
```bash
ls /dev/dri/
# Should show: card0  renderD128
```

## Credentials

| User | Password |
|------|----------|
| arch | arch |

## Command Reference

| Command | Description |
|---------|-------------|
| `cargo xtask vm setup` | Download Arch cloud image |
| `cargo xtask vm prepare` | Build levitate binary |
| `cargo xtask vm start --gui` | Start VM with display |
| `cargo xtask vm stop` | Stop VM |
| `cargo xtask vm ssh` | SSH into VM |
| `cargo xtask vm copy` | Copy levitate + recipes |
| `cargo xtask vm status` | Check VM status |

## Success Checklist

- [ ] VM boots successfully
- [ ] Can login as arch:arch
- [ ] `cargo xtask vm copy` succeeds
- [ ] `levitate list` shows 22 packages
- [ ] `levitate desktop` completes
- [ ] `sway` starts
- [ ] Can open terminal (Super+Enter)
- [ ] Can open launcher (Super+d)
