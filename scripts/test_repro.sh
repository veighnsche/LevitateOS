#!/bin/bash
set -e

# Compile
aarch64-linux-gnu-gcc -c kernel/src/repro_boot.s -o repro_boot.o

# Link
aarch64-linux-gnu-ld -T linker_repro.ld repro_boot.o -o repro.elf

# Convert to bin
aarch64-linux-gnu-objcopy -O binary repro.elf repro.bin

echo "Running QEMU..."
qemu-system-aarch64 \
    -M virt \
    -cpu cortex-a53 \
    -m 512M \
    -kernel repro.bin \
    -display none \
    -serial stdio \
    -d int,mmu \
    -D qemu_repro.log \
    -no-reboot &

QEMU_PID=$!
sleep 2
kill $QEMU_PID || true

echo "QEMU finished. Checking log..."
if grep -i "exception" qemu_repro.log; then
    echo "FAULT DETECTED"
    grep -A 5 "Taking exception" qemu_repro.log | head -n 20
else
    echo "NO FAULT DETECTED"
fi
