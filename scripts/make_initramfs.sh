#!/bin/bash
set -e

# Ensure userspace is built (optional, but good practice)
# pushd userspace
# cargo build --release
# popd

# Create staging dir
mkdir -p initrd_root

# Copy shell binary (new location)
cp userspace/target/aarch64-unknown-none/release/shell initrd_root/

# Create dummy files
echo "Hello from initramfs!" > initrd_root/hello.txt
echo "This is a test file." > initrd_root/test.txt

# Create CPIO
cd initrd_root
find . | cpio -o -H newc > ../initramfs.cpio
cd ..

echo "Initramfs created at initramfs.cpio"
