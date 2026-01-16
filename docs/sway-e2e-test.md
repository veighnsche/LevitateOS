# Sway Desktop E2E Test Instructions

End-to-end test for installing and running Sway desktop using the levitate package manager.

## Prerequisites

- QEMU with KVM support (`qemu-system-x86_64`)
- ~25GB free disk space
- Internet connection (for downloading Arch ISO and packages)

Check QEMU is installed:
```bash
qemu-system-x86_64 --version
```

## Phase 1: Setup (One-time)

### 1.1 Create disk image and download Arch ISO

```bash
cargo xtask vm setup
```

This creates:
- `.vm/levitate-test.qcow2` (20GB disk image)
- `.vm/arch.iso` (Arch Linux installer, ~1GB download)

### 1.2 Build levitate and prepare recipes

```bash
cargo xtask vm prepare
```

This creates:
- `.vm/levitate` (compiled binary)
- `.vm/recipes/` (22 recipe files)
- `.vm/install-arch.sh` (installation script)

## Phase 2: Install Arch Linux (~15 minutes)

### 2.1 Boot the Arch installer

```bash
cargo xtask vm start --gui --cdrom arch --uefi
```

A QEMU window opens with the Arch live environment.

### 2.2 Install Arch (in the VM)

Once you see the `root@archiso` prompt, run these commands:

```bash
# Partition the disk (UEFI layout)
parted -s /dev/vda mklabel gpt
parted -s /dev/vda mkpart ESP fat32 1MiB 513MiB
parted -s /dev/vda set 1 esp on
parted -s /dev/vda mkpart root ext4 513MiB 100%

# Format partitions
mkfs.fat -F32 /dev/vda1
mkfs.ext4 -F /dev/vda2

# Mount
mount /dev/vda2 /mnt
mkdir -p /mnt/boot
mount /dev/vda1 /mnt/boot

# Install base system + build tools + Wayland deps
pacstrap -K /mnt base linux linux-firmware base-devel git \
  meson ninja cmake pkg-config \
  networkmanager openssh sudo \
  mesa libdrm pixman \
  wayland wayland-protocols libxkbcommon libinput \
  cairo pango gdk-pixbuf2 json-c pcre2 \
  scdoc libevdev mtdev libdisplay-info hwdata

# Generate fstab
genfstab -U /mnt >> /mnt/etc/fstab

# Configure system
arch-chroot /mnt /bin/bash
```

Inside the chroot:

```bash
# Timezone and locale
ln -sf /usr/share/zoneinfo/UTC /etc/localtime
hwclock --systohc
echo "en_US.UTF-8 UTF-8" >> /etc/locale.gen
locale-gen
echo "LANG=en_US.UTF-8" > /etc/locale.conf

# Hostname
echo "levitate-test" > /etc/hostname

# Set passwords
echo "root:live" | chpasswd

# Create user live:live
useradd -m -G wheel,seat -s /bin/bash live
echo "live:live" | chpasswd
echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" >> /etc/sudoers

# Enable services
systemctl enable NetworkManager
systemctl enable sshd

# Install bootloader
bootctl install

# Create boot entry
cat > /boot/loader/loader.conf << 'EOF'
default arch.conf
timeout 3
EOF

# Get the PARTUUID
PARTUUID=$(blkid -s PARTUUID -o value /dev/vda2)

cat > /boot/loader/entries/arch.conf << EOF
title   Arch Linux
linux   /vmlinuz-linux
initrd  /initramfs-linux.img
options root=PARTUUID=$PARTUUID rw
EOF

# Exit chroot
exit
```

Back in the live environment:

```bash
# Unmount and reboot
umount -R /mnt
reboot
```

### 2.3 Remove the ISO

Close the QEMU window (or let it reboot). The VM will fail to boot from the ISO again. Stop it:

```bash
cargo xtask vm stop
```

## Phase 3: First Boot and Setup

### 3.1 Boot the installed system

```bash
cargo xtask vm start --gui --uefi
```

You should see the bootloader, then a login prompt.

### 3.2 Login

```
Username: live
Password: live
```

### 3.3 Copy levitate and recipes to VM

From another terminal on the **host**:

```bash
cargo xtask vm copy
```

This SSHs into the VM and copies:
- `/usr/local/bin/levitate`
- `/usr/share/levitate/recipes/*.recipe`

### 3.4 Verify levitate works

In the VM:

```bash
levitate list
```

You should see 22 packages listed.

## Phase 4: Install Sway Desktop

### 4.1 Install the desktop (in VM)

```bash
levitate desktop
```

This installs (in order):
1. wayland, wayland-protocols
2. libxkbcommon, libinput
3. seatd
4. wlroots
5. sway, swaybg, swaylock, swayidle
6. gtk-layer-shell
7. foot, waybar, wofi, mako
8. grim, slurp, wl-clipboard

**Note:** This will take a while as it downloads and compiles each package.

### 4.2 Start seatd (required for Sway)

```bash
sudo systemctl enable --now seatd
sudo usermod -aG seat live
# Log out and back in for group to take effect
exit
```

Log back in as `live`.

### 4.3 Start Sway

```bash
sway
```

**Expected result:** Sway compositor starts, you see a gray desktop with a status bar (waybar).

## Phase 5: Verify Desktop Works

### 5.1 Open a terminal

Press `Super + Enter` (Windows key + Enter)

A foot terminal should open.

### 5.2 Open the launcher

Press `Super + d`

Wofi launcher should appear.

### 5.3 Take a screenshot

```bash
grim ~/screenshot.png
```

### 5.4 Exit Sway

Press `Super + Shift + e`

Or type `swaymsg exit` in a terminal.

## Troubleshooting

### "Failed to connect to socket" when starting sway

seatd is not running:
```bash
sudo systemctl start seatd
```

### Black screen after starting sway

virtio-gpu might not be working. Check:
```bash
ls /dev/dri/
```

Should show `card0` and `renderD128`.

### SSH connection refused

SSH might not be running:
```bash
sudo systemctl start sshd
```

### levitate command not found

Copy didn't work. Manually copy:
```bash
# On host, start a web server
cd .vm && python3 -m http.server 8080

# In VM
curl http://10.0.2.2:8080/levitate -o /tmp/levitate
sudo install -m755 /tmp/levitate /usr/local/bin/levitate
```

### Package build fails

Check if build dependencies are installed:
```bash
pacman -S --needed base-devel meson ninja cmake
```

## Quick Reference

| Action | Command |
|--------|---------|
| Start VM (GUI) | `cargo xtask vm start --gui --uefi` |
| Stop VM | `cargo xtask vm stop` |
| SSH into VM | `cargo xtask vm ssh` |
| Copy files to VM | `cargo xtask vm copy` |
| VM status | `cargo xtask vm status` |
| List packages | `levitate list` |
| Install desktop | `levitate desktop` |
| Start Sway | `sway` |

## Credentials

| User | Password |
|------|----------|
| live | live |
| root | live |

## Success Criteria

- [ ] Arch Linux boots successfully
- [ ] Can login as live:live
- [ ] `levitate list` shows 22 packages
- [ ] `levitate desktop` completes without errors
- [ ] `sway` starts and shows desktop
- [ ] Can open terminal with Super+Enter
- [ ] Can open launcher with Super+d
- [ ] Can take screenshot with `grim`
