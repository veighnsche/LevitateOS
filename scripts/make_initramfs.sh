#!/bin/bash
# TEAM_198: Updated to include all levbox utilities
set -e

RELEASE_DIR="userspace/target/aarch64-unknown-none/release"

# Create staging dir
mkdir -p initrd_root

# Copy init binary
cp "$RELEASE_DIR/init" initrd_root/

# Copy shell binary
cp "$RELEASE_DIR/shell" initrd_root/

# TEAM_198: Copy levbox utilities
LEVBOX_UTILS="cat ls pwd mkdir rmdir rm mv cp"
for util in $LEVBOX_UTILS; do
    if [ -f "$RELEASE_DIR/$util" ]; then
        cp "$RELEASE_DIR/$util" initrd_root/
        echo "  Added: $util"
    else
        echo "  Warning: $util not found"
    fi
done

# Create dummy files
echo "Hello from initramfs!" > initrd_root/hello.txt
echo "This is a test file." > initrd_root/test.txt

# Create CPIO
cd initrd_root
find . | cpio -o -H newc > ../initramfs.cpio
cd ..

echo "Initramfs created at initramfs.cpio"
