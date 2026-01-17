#!/bin/bash
# Automated Arch Linux installation for LevitateOS testing
# Run this inside the Arch live environment

set -euo pipefail

DISK="/dev/vda"
HOSTNAME="levitate-test"
USERNAME="live"
PASSWORD="live"
ROOT_PASSWORD="live"

echo "=== LevitateOS Arch Installation ==="
echo "Disk: $DISK"
echo "User: $USERNAME / $PASSWORD"
echo ""

# Wait for disk to be available
while [ ! -b "$DISK" ]; do
    echo "Waiting for $DISK..."
    sleep 1
done

echo "[1/8] Partitioning disk..."
# Create GPT partition table with:
# - 512M EFI partition
# - Rest as root
parted -s "$DISK" mklabel gpt
parted -s "$DISK" mkpart ESP fat32 1MiB 513MiB
parted -s "$DISK" set 1 esp on
parted -s "$DISK" mkpart root ext4 513MiB 100%

# Format partitions
echo "[2/8] Formatting partitions..."
mkfs.fat -F32 "${DISK}1"
mkfs.ext4 -F "${DISK}2"

# Mount
echo "[3/8] Mounting filesystems..."
mount "${DISK}2" /mnt
mkdir -p /mnt/boot
mount "${DISK}1" /mnt/boot

# Install base system
echo "[4/8] Installing base system (this takes a while)..."
pacstrap -K /mnt \
    base linux linux-firmware \
    base-devel git \
    meson ninja cmake pkg-config \
    networkmanager openssh sudo \
    mesa libdrm \
    wayland wayland-protocols \
    libxkbcommon libinput \
    cairo pango gdk-pixbuf2 \
    json-c pcre2 \
    scdoc \
    vim nano

# Generate fstab
echo "[5/8] Generating fstab..."
genfstab -U /mnt >> /mnt/etc/fstab

# Configure system
echo "[6/8] Configuring system..."
arch-chroot /mnt /bin/bash <<CHROOT
# Timezone and locale
ln -sf /usr/share/zoneinfo/UTC /etc/localtime
hwclock --systohc
echo "en_US.UTF-8 UTF-8" >> /etc/locale.gen
locale-gen
echo "LANG=en_US.UTF-8" > /etc/locale.conf

# Hostname
echo "$HOSTNAME" > /etc/hostname
cat > /etc/hosts <<EOF
127.0.0.1   localhost
::1         localhost
127.0.1.1   $HOSTNAME.localdomain $HOSTNAME
EOF

# Root password
echo "root:$ROOT_PASSWORD" | chpasswd

# Create user
useradd -m -G wheel -s /bin/bash $USERNAME
echo "$USERNAME:$PASSWORD" | chpasswd
echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" >> /etc/sudoers

# Enable services
systemctl enable NetworkManager
systemctl enable sshd

# Bootloader (systemd-boot)
bootctl install
cat > /boot/loader/loader.conf <<EOF
default arch.conf
timeout 3
editor no
EOF

cat > /boot/loader/entries/arch.conf <<EOF
title   Arch Linux (LevitateOS Test)
linux   /vmlinuz-linux
initrd  /initramfs-linux.img
options root=PARTUUID=$(blkid -s PARTUUID -o value ${DISK}2) rw
EOF

# Create recipe directories
mkdir -p /usr/share/recipe/recipes
mkdir -p /usr/local/bin
CHROOT

# Copy recipe binary (if available)
echo "[7/8] Installing recipe package manager..."
if [ -f /tmp/recipe ]; then
    cp /tmp/recipe /mnt/usr/local/bin/recipe
    chmod 755 /mnt/usr/local/bin/recipe
    echo "  Installed recipe binary"
fi

# Copy example recipes (OPTIONAL - for testing package manager only)
if [ -d /tmp/recipes ]; then
    cp /tmp/recipes/*.recipe /mnt/usr/share/recipe/recipes/ 2>/dev/null || true
    echo "  Installed $(ls /mnt/usr/share/recipe/recipes/*.recipe 2>/dev/null | wc -l) example recipes (for testing)"
fi

# Cleanup
echo "[8/8] Finishing up..."
umount -R /mnt

echo ""
echo "=== Installation Complete ==="
echo "You can now reboot and log in as:"
echo "  Username: $USERNAME"
echo "  Password: $PASSWORD"
echo ""
echo "To install packages, use the recipe command:"
echo "  recipe install <package>"
echo ""
