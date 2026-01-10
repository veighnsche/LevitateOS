#!/bin/bash
# Test script for libsyscall-tests on LevitateOS

set -e

cd /home/vince/Projects/LevitateOS

echo "=== Testing libsyscall-tests on LevitateOS (aarch64) ==="
echo ""

# Start QEMU with commands piped in
(
    sleep 5  # Wait for boot
    echo "ls"  # List files in initramfs
    sleep 1
    echo "libsyscall-tests"  # Run the tests
    sleep 5
    echo "poweroff"  # Shutdown
) | timeout 30 qemu-system-aarch64 \
  -M virt,gic-version=2 \
  -cpu cortex-a53 \
  -m 512M \
  -nographic \
  -kernel kernel64_rust.bin \
  -initrd initramfs_aarch64.cpio \
  -drive file=tinyos_disk.img,format=raw,if=none,id=hd0 \
  -device virtio-blk-device,drive=hd0 \
  -device virtio-keyboard-device \
  -device virtio-tablet-device \
  -device virtio-net-device,netdev=net0 \
  -netdev user,id=net0 \
  -device virtio-gpu-pci,xres=1280,yres=800 \
  -serial mon:stdio \
  -no-reboot 2>&1 | tee /tmp/libsyscall_test_output.log

echo ""
echo "=== Test output saved to /tmp/libsyscall_test_output.log ==="
echo ""

# Check for test results in the log
if grep -q "libsyscall-tests" /tmp/libsyscall_test_output.log; then
    echo "✅ Test binary was found and executed"
else
    echo "❌ Test binary was not found"
    exit 1
fi
