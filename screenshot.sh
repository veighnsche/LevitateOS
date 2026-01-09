#!/bin/bash
# screenshot.sh - QEMU VM Screenshot Tool
#
# Takes a screenshot of the LevitateOS VM display.
# TEAM_326: Consider using: cargo xtask vm screenshot

set -e

# === Defaults ===
ARCH="x86_64"
WAIT_SECS=10
OUTPUT=""
EXTRA_ARGS=""
SKIP_BUILD=false
VERBOSE=false

# === Colors ===
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# === Help ===
usage() {
    cat << EOF
📸 LevitateOS Screenshot Tool

Usage: $(basename "$0") [OPTIONS]

Options:
  -a, --arch ARCH       Target architecture (x86_64 or aarch64)
                        Default: x86_64
  -w, --wait SECONDS    Seconds to wait before screenshot
                        Default: 10
  -o, --output FILE     Output filename (without extension)
                        Default: screenshot_YYYYMMDD_HHMMSS
  -e, --extra ARGS      Extra QEMU arguments (quoted)
  -s, --skip-build      Skip the build step (use existing binaries)
  -v, --verbose         Show verbose output
  -h, --help            Show this help message

Examples:
  $(basename "$0")                          # x86_64, 10s wait
  $(basename "$0") -a aarch64               # aarch64, 10s wait
  $(basename "$0") -a aarch64 -w 5          # aarch64, 5s wait
  $(basename "$0") -w 20 -o myshot          # x86_64, 20s, custom name
  $(basename "$0") -s                       # Skip build, use existing
  $(basename "$0") -e "-m 1G"               # Extra memory
EOF
    exit 0
}

# === Argument Parsing ===
while [[ $# -gt 0 ]]; do
    case $1 in
        -a|--arch)
            ARCH="$2"
            shift 2
            ;;
        -w|--wait)
            WAIT_SECS="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT="$2"
            shift 2
            ;;
        -e|--extra)
            EXTRA_ARGS="$2"
            shift 2
            ;;
        -s|--skip-build)
            SKIP_BUILD=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# === Validation ===
if [[ "$ARCH" != "aarch64" && "$ARCH" != "x86_64" ]]; then
    echo -e "${RED}❌ Invalid architecture: $ARCH${NC}"
    echo "   Must be 'aarch64' or 'x86_64'"
    exit 1
fi

if ! [[ "$WAIT_SECS" =~ ^[0-9]+$ ]]; then
    echo -e "${RED}❌ Wait time must be a number: $WAIT_SECS${NC}"
    exit 1
fi

# === Setup ===
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
if [[ -z "$OUTPUT" ]]; then
    OUTPUT="screenshot_${ARCH}_${TIMESTAMP}"
fi
OUTPUT_FILE="${OUTPUT}.ppm"
QMP_SOCK="./qmp_screenshot_$$.sock"

# === Banner ===
echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║${NC}  📸 ${GREEN}LevitateOS Screenshot Tool${NC}                                   ${BLUE}║${NC}"
echo -e "${BLUE}╠══════════════════════════════════════════════════════════════════╣${NC}"
echo -e "${BLUE}║${NC}  Arch:   ${YELLOW}$ARCH${NC}"
echo -e "${BLUE}║${NC}  Wait:   ${YELLOW}${WAIT_SECS}s${NC}"
echo -e "${BLUE}║${NC}  Output: ${YELLOW}${OUTPUT}.png${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# === Cleanup Handler ===
cleanup() {
    echo ""
    echo -e "${BLUE}🧹 Cleaning up...${NC}"
    [[ -n "$QEMU_PID" ]] && kill "$QEMU_PID" 2>/dev/null || true
    rm -f "$QMP_SOCK"
}
trap cleanup EXIT

# === Build ===
if [[ "$SKIP_BUILD" == "false" ]]; then
    echo -e "${BLUE}🔨 Building for $ARCH...${NC}"
    if [[ "$ARCH" == "x86_64" ]]; then
        if [[ "$VERBOSE" == "true" ]]; then
            cargo xtask build iso --arch x86_64
        else
            cargo xtask build iso --arch x86_64 2>&1 | tail -3
        fi
    else
        if [[ "$VERBOSE" == "true" ]]; then
            cargo xtask build all --arch aarch64
        else
            cargo xtask build all --arch aarch64 2>&1 | tail -3
        fi
    fi
else
    echo -e "${YELLOW}⏭️  Skipping build (using existing binaries)${NC}"
fi

# === QEMU Setup ===
if [[ "$ARCH" == "aarch64" ]]; then
    QEMU_BIN="qemu-system-aarch64"
    MACHINE="virt"
    CPU="cortex-a53"
    KERNEL_ARGS="-kernel kernel64_rust.bin -initrd initramfs.cpio"
    DEVICE_SUFFIX="device"
else
    QEMU_BIN="qemu-system-x86_64"
    MACHINE="q35"
    CPU="qemu64"
    KERNEL_ARGS="-cdrom levitate.iso -boot d"
    DEVICE_SUFFIX="pci"
fi

# Clean previous socket
rm -f "$QMP_SOCK"

echo -e "${BLUE}🚀 Starting QEMU (headless)...${NC}"

# Start QEMU in background
$QEMU_BIN \
    -M "$MACHINE" \
    -cpu "$CPU" \
    -m 512M \
    $KERNEL_ARGS \
    -display none \
    -device virtio-gpu-pci,xres=1920,yres=1080 \
    -device "virtio-keyboard-${DEVICE_SUFFIX}" \
    -device "virtio-tablet-${DEVICE_SUFFIX}" \
    -serial null \
    -qmp "unix:${QMP_SOCK},server,nowait" \
    -no-reboot \
    $EXTRA_ARGS &
QEMU_PID=$!

# Wait for QMP socket
echo -e "${BLUE}⏳ Waiting for QEMU...${NC}"
for i in {1..20}; do
    if [[ -S "$QMP_SOCK" ]]; then
        break
    fi
    sleep 0.5
done

if [[ ! -S "$QMP_SOCK" ]]; then
    echo -e "${RED}❌ QMP socket not available${NC}"
    exit 1
fi

# Boot wait
sleep 1
echo -e "${BLUE}⏱️  Waiting ${WAIT_SECS}s for VM to boot...${NC}"
sleep "$WAIT_SECS"

# Take screenshot
echo -e "${BLUE}📸 Taking screenshot...${NC}"

ABS_OUTPUT="$(pwd)/$OUTPUT_FILE"
if command -v socat &>/dev/null; then
    {
        sleep 0.3
        echo '{"execute": "qmp_capabilities"}'
        sleep 0.3
        echo "{\"execute\": \"screendump\", \"arguments\": {\"filename\": \"$ABS_OUTPUT\"}}"
        sleep 0.5
    } | socat - UNIX-CONNECT:"$QMP_SOCK" > /dev/null 2>&1 || true
elif command -v nc &>/dev/null; then
    {
        sleep 0.3
        echo '{"execute": "qmp_capabilities"}'
        sleep 0.3
        echo "{\"execute\": \"screendump\", \"arguments\": {\"filename\": \"$ABS_OUTPUT\"}}"
        sleep 0.5
    } | nc -U "$QMP_SOCK" > /dev/null 2>&1 || true
else
    echo -e "${RED}❌ Neither socat nor nc available${NC}"
    exit 1
fi

# Wait for file write
sleep 1

# === Result ===
if [[ -f "$OUTPUT_FILE" ]]; then
    SIZE=$(stat -c%s "$OUTPUT_FILE" 2>/dev/null || stat -f%z "$OUTPUT_FILE" 2>/dev/null || echo "0")
    if [[ "$SIZE" -gt 100 ]]; then
        # Convert to PNG
        PNG_FILE="${OUTPUT}.png"
        if command -v convert &>/dev/null; then
            convert "$OUTPUT_FILE" "$PNG_FILE" 2>/dev/null
            rm -f "$OUTPUT_FILE"
            SIZE=$(stat -c%s "$PNG_FILE" 2>/dev/null || stat -f%z "$PNG_FILE" 2>/dev/null)
            echo ""
            echo -e "${GREEN}✅ Screenshot saved: ${YELLOW}$PNG_FILE${NC} (${SIZE} bytes)"
        else
            echo ""
            echo -e "${GREEN}✅ Screenshot saved: ${YELLOW}$OUTPUT_FILE${NC} (${SIZE} bytes)"
            echo -e "${YELLOW}   (Install ImageMagick to auto-convert to PNG)${NC}"
        fi
    else
        echo -e "${RED}❌ Screenshot file is empty${NC}"
        rm -f "$OUTPUT_FILE"
        exit 1
    fi
else
    echo -e "${RED}❌ Screenshot failed${NC}"
    exit 1
fi
