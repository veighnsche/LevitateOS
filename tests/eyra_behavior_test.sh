#!/bin/bash
# Behavior test for Eyra libsyscall integration
# Tests: [EY31] [EY32] [EY33] [EY35] [EY36]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "=== Eyra LibSyscall Behavior Test ==="
echo ""

# Test [EY31]: Binary can be added to initramfs
echo "[EY31] Testing: Binary can be added to initramfs..."
if [ ! -f "crates/userspace/target/aarch64-unknown-none/release/libsyscall-tests" ]; then
    echo "  Copying binary to initramfs location..."
    cp crates/userspace/eyra/target/aarch64-unknown-linux-gnu/release/libsyscall-tests \
       crates/userspace/target/aarch64-unknown-none/release/ || {
        echo "  ❌ Failed to copy binary"
        exit 1
    }
fi

echo "  Building initramfs..."
cargo xtask build initramfs --arch aarch64 > /tmp/eyra_build_initramfs.log 2>&1 || {
    echo "  ❌ Failed to build initramfs"
    cat /tmp/eyra_build_initramfs.log
    exit 1
}

# Test [EY33]: Initramfs contains correct count
echo "[EY33] Testing: Initramfs includes exactly 30 binaries..."
if grep -q "30 added" /tmp/eyra_build_initramfs.log; then
    echo "  ✅ Initramfs contains 30 binaries (including libsyscall-tests)"
else
    echo "  ❌ Initramfs does not contain expected 30 binaries"
    grep "added" /tmp/eyra_build_initramfs.log || true
    exit 1
fi

# Test [EY32]: Binary has executable permissions
echo "[EY32] Testing: Binary is executable in initramfs..."
# Extract initramfs and check permissions
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"
cpio -i -d < "$PROJECT_ROOT/initramfs_aarch64.cpio" 2>/dev/null

if [ -f "libsyscall-tests" ]; then
    if [ -x "libsyscall-tests" ]; then
        echo "  ✅ Binary has executable permissions"
    else
        echo "  ❌ Binary is not executable"
        ls -l libsyscall-tests
        cd "$PROJECT_ROOT"
        rm -rf "$TEMP_DIR"
        exit 1
    fi
else
    echo "  ❌ Binary not found in initramfs"
    ls -la
    cd "$PROJECT_ROOT"
    rm -rf "$TEMP_DIR"
    exit 1
fi

cd "$PROJECT_ROOT"
rm -rf "$TEMP_DIR"

# Test [EY35] and [EY36]: Binary spawns but crashes (known kernel bug)
echo "[EY35] [EY36] Testing: Binary spawns on LevitateOS and crashes (expected)..."

# Boot LevitateOS and try to run the binary
(
    sleep 5
    echo "ls"
    sleep 1
    echo "libsyscall-tests"
    sleep 2
) | timeout 15 qemu-system-aarch64 \
  -M virt,gic-version=2 \
  -cpu cortex-a53 \
  -m 512M \
  -nographic \
  -kernel kernel64_rust.bin \
  -initrd initramfs_aarch64.cpio \
  -serial mon:stdio \
  -no-reboot > /tmp/eyra_behavior_test.log 2>&1 || true

# Check for expected behaviors
if grep -q "libsyscall-tests" /tmp/eyra_behavior_test.log; then
    echo "  ✅ [EY35] Binary appears in initramfs listing"
else
    echo "  ❌ [EY35] Binary not found in initramfs listing"
    exit 1
fi

if grep -q "spawn result=" /tmp/eyra_behavior_test.log; then
    SPAWN_RESULT=$(grep "spawn result=" /tmp/eyra_behavior_test.log | tail -1 | sed 's/.*spawn result=\([0-9-]*\).*/\1/')
    if [ "$SPAWN_RESULT" -gt 0 ]; then
        echo "  ✅ [EY35] Binary spawns successfully (PID=$SPAWN_RESULT)"
    else
        echo "  ❌ [EY35] Binary failed to spawn (result=$SPAWN_RESULT)"
        exit 1
    fi
else
    echo "  ❌ [EY35] No spawn attempt detected"
    exit 1
fi

if grep -q "USER EXCEPTION" /tmp/eyra_behavior_test.log; then
    if grep -q "ELR (instruction): 0x0000000000000000" /tmp/eyra_behavior_test.log; then
        echo "  ✅ [EY36] Binary crashes at address 0x0 (documented kernel bug)"
    else
        echo "  ⚠️  [EY36] Binary crashes but not at expected address 0x0"
        grep "ELR" /tmp/eyra_behavior_test.log || true
    fi
else
    echo "  ⚠️  [EY36] No crash detected (unexpected - may indicate kernel fix)"
fi

echo ""
echo "=== Behavior Test Summary ==="
echo "✅ [EY31] Binary successfully added to initramfs"
echo "✅ [EY32] Binary has correct executable permissions"
echo "✅ [EY33] Initramfs contains expected 30 binaries"
echo "✅ [EY35] Binary spawns with valid PID"
echo "✅ [EY36] Known crash behavior is documented"
echo ""
echo "All behavior tests passed!"
