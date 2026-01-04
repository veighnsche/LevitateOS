#!/bin/bash
mkdir -p initrd_root
echo "Hello from initramfs!" > initrd_root/hello.txt
echo "This is a test file." > initrd_root/test.txt
cd initrd_root
find . | cpio -o -H newc > ../initramfs.cpio
cd ..
echo "Initramfs created at initramfs.cpio"
